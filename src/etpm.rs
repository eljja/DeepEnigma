use pyo3::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::constant_time::ct_select_i32;

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum UpdateRule {
    Hebbian,
    AntiHebbian,
    RandomWalk,
}

impl UpdateRule {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hebbian" => Some(Self::Hebbian),
            "antihebbian" | "anti-hebbian" => Some(Self::AntiHebbian),
            "randomwalk" | "random-walk" => Some(Self::RandomWalk),
            _ => None,
        }
    }
}

#[pyclass(eq, eq_int)]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActivationType {
    /// Standard sign-based activation: σ(h) = sign(h).
    Standard,
    /// Pure chaotic activation: σ(h) = sign(sin(π·h/(2L))).
    /// WARNING: Impedes synchronization convergence. Use Hybrid for production.
    Chaotic,
    /// Hybrid mode: uses Standard activation during synchronization for reliable
    /// convergence, then applies chaotic weight transformation for key hardening.
    /// This is the recommended production mode.
    Hybrid,
}

impl ActivationType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "standard" => Some(Self::Standard),
            "chaotic" => Some(Self::Chaotic),
            "hybrid" => Some(Self::Hybrid),
            _ => None,
        }
    }
}

#[pyclass]
pub struct ETPM {
    #[pyo3(get)]
    pub k: usize,
    #[pyo3(get)]
    pub n: usize,
    #[pyo3(get)]
    pub l: i32,
    pub weights: Vec<Vec<i32>>,
    pub outputs: Vec<i32>,
    pub last_input: Vec<Vec<i32>>,
    #[pyo3(get)]
    pub activation_type: ActivationType,
}

/// Securely wipe all weight and state data when ETPM is dropped.
/// This prevents secrets from lingering in memory after use (Cold Boot defense).
impl Drop for ETPM {
    fn drop(&mut self) {
        for row in &mut self.weights {
            row.zeroize();
        }
        self.outputs.zeroize();
        for row in &mut self.last_input {
            row.zeroize();
        }
    }
}

#[pymethods]
impl ETPM {
    #[new]
    #[pyo3(signature = (k, n, l, activation_type = "hybrid"))]
    pub fn new(k: usize, n: usize, l: i32, activation_type: &str) -> PyResult<Self> {
        // Parameter validation to prevent degenerate or dangerous configurations.
        if k == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "K (hidden units) must be >= 1",
            ));
        }
        if n == 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "N (inputs per unit) must be >= 1",
            ));
        }
        if l <= 0 {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "L (synaptic depth) must be >= 1",
            ));
        }

        let act_type = ActivationType::from_str(activation_type)
            .ok_or_else(|| pyo3::exceptions::PyValueError::new_err("Invalid activation type. Choose 'standard', 'chaotic', or 'hybrid'."))?;

        // Initialize weights to 0 first, will be randomized in initialize_weights
        let weights = vec![vec![0; n]; k];
        let outputs = vec![0; k];
        let last_input = vec![vec![0; n]; k];

        let mut etpm = Self {
            k,
            n,
            l,
            weights,
            outputs,
            last_input,
            activation_type: act_type,
        };

        // Randomly initialize weights using default thread rng
        etpm.initialize_weights(None)?;

        Ok(etpm)
    }

    /// Initializes or randomizes weights. If a seed is provided, a deterministic RNG (ChaCha8) is used.
    #[pyo3(signature = (seed = None))]
    pub fn initialize_weights(&mut self, seed: Option<u64>) -> PyResult<()> {
        let mut rng: Box<dyn RngCore> = match seed {
            Some(s) => Box::new(ChaCha8Rng::seed_from_u64(s)),
            None => Box::new(crate::rng::secure_rng()),
        };

        for i in 0..self.k {
            for j in 0..self.n {
                self.weights[i][j] = rng.gen_range(-self.l..=self.l);
            }
        }
        Ok(())
    }

    /// Computes the output of the E-TPM for a given input matrix.
    /// Inputs should be shape K x N with values in {-1, 1}.
    pub fn calculate_output(&mut self, inputs: Vec<Vec<i32>>) -> PyResult<i32> {
        if inputs.len() != self.k {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Input row count ({}) must match K ({})",
                inputs.len(),
                self.k
            )));
        }

        for (i, row) in inputs.iter().enumerate() {
            if row.len() != self.n {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Input column count at row {} ({}) must match N ({})",
                    i,
                    row.len(),
                    self.n
                )));
            }
            for &val in row.iter() {
                if val != 1 && val != -1 {
                    return Err(pyo3::exceptions::PyValueError::new_err(
                        "Input values must be either -1 or 1",
                    ));
                }
            }
        }

        self.last_input = inputs.clone();
        let mut tau = 1;

        for i in 0..self.k {
            // Compute inner product h_i = sum(w_ij * x_ij)
            let mut h: i32 = 0;
            for j in 0..self.n {
                h += self.weights[i][j] * inputs[i][j];
            }

            // Apply activation function
            let sigma = match self.activation_type {
                ActivationType::Standard | ActivationType::Hybrid => {
                    // Standard sign activation — used during synchronization.
                    // Hybrid mode delegates non-linearity to post-sync key hardening.
                    if h > 0 {
                        1
                    } else if h < 0 {
                        -1
                    } else {
                        1
                    }
                }
                ActivationType::Chaotic => {
                    // Pure chaotic activation: sin(π·h/(2L)).
                    // Note: this mode disrupts synchronization convergence.
                    let freq = std::f64::consts::PI / (2.0 * self.l as f64);
                    let val = (h as f64 * freq).sin();
                    if val >= 0.0 {
                        1
                    } else {
                        -1
                    }
                }
            };

            self.outputs[i] = sigma;
            tau *= sigma;
        }

        Ok(tau)
    }

    /// Updates weights based on the specified rule and parity output tau.
    ///
    /// Uses **constant-time conditional masking** to prevent timing side-channel
    /// attacks: the update delta is always computed for every hidden unit, then
    /// multiplied by a 0-or-1 mask derived from the output-match condition.
    /// This ensures execution time is independent of which units match tau.
    #[pyo3(signature = (tau, rule = "hebbian"))]
    pub fn update_weights(&mut self, tau: i32, rule: &str) -> PyResult<()> {
        let rule_enum = UpdateRule::from_str(rule).ok_or_else(|| {
            pyo3::exceptions::PyValueError::new_err(
                "Invalid update rule. Choose 'hebbian', 'antihebbian', or 'randomwalk'.",
            )
        })?;

        for i in 0..self.k {
            // Constant-time: always compute delta, select via mask.
            let match_condition = self.outputs[i] == tau;

            for j in 0..self.n {
                let w_ij = self.weights[i][j];
                let x_ij = self.last_input[i][j];

                let delta = match rule_enum {
                    UpdateRule::Hebbian => x_ij * tau,
                    UpdateRule::AntiHebbian => -x_ij * tau,
                    UpdateRule::RandomWalk => x_ij,
                };

                // ct_select_i32: returns delta if match, 0 otherwise (no branch)
                let applied_delta = ct_select_i32(match_condition, delta, 0);
                let new_w = w_ij + applied_delta;

                // Clamp weights to [-L, L]
                self.weights[i][j] = new_w.clamp(-self.l, self.l);
            }
        }
        Ok(())
    }

    /// Dynamically scales synaptic depth L, mapping current weights into the new bounds.
    pub fn scale_synaptic_depth(&mut self, new_l: i32) -> PyResult<()> {
        if new_l <= self.l {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "New synaptic depth L must be greater than current L",
            ));
        }

        let scale = new_l as f64 / self.l as f64;
        for i in 0..self.k {
            for j in 0..self.n {
                let scaled_w = (self.weights[i][j] as f64 * scale).round() as i32;
                self.weights[i][j] = scaled_w.clamp(-new_l, new_l);
            }
        }
        self.l = new_l;
        Ok(())
    }

    pub fn get_weights(&self) -> Vec<Vec<i32>> {
        self.weights.clone()
    }

    pub fn set_weights(&mut self, weights: Vec<Vec<i32>>) -> PyResult<()> {
        if weights.len() != self.k {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Weight row count ({}) must match K ({})",
                weights.len(),
                self.k
            )));
        }
        for (i, row) in weights.iter().enumerate() {
            if row.len() != self.n {
                return Err(pyo3::exceptions::PyValueError::new_err(format!(
                    "Weight column count at row {} ({}) must match N ({})",
                    i,
                    row.len(),
                    self.n
                )));
            }
            for &val in row.iter() {
                if val.abs() > self.l {
                    return Err(pyo3::exceptions::PyValueError::new_err(format!(
                        "Weight value {} exceeds synaptic depth L ({})",
                        val,
                        self.l
                    )));
                }
            }
        }
        self.weights = weights;
        Ok(())
    }

    pub fn get_hidden_outputs(&self) -> Vec<i32> {
        self.outputs.clone()
    }

    /// Applies a chaotic integer-only transformation to the weight matrix
    /// for key hardening before SHA-256 derivation.
    ///
    /// Uses a combination of integer Tent Map and SipHash-inspired non-linear
    /// mixing to guarantee both platform-independent determinism (integer-only)
    /// and strong Avalanche Effect (SAC-compliant diffusion).
    ///
    /// Returns the chaotically-transformed weight matrix without modifying
    /// the original ETPM state.
    pub fn chaotic_transform(&self, iterations: u32) -> Vec<Vec<i32>> {
        let mut result = self.weights.clone();
        let m = (2 * self.l) as i64;

        for i in 0..self.k {
            for j in 0..self.n {
                let w = self.weights[i][j];
                // Shift to unsigned range [0, 2L]
                let mut x = (w + self.l) as i64;

                for round in 0..iterations {
                    // Integer chaotic Tent map:
                    // x_{n+1} = 2*x_n if x_n < M/2 else 2*(M - x_n)
                    let half = m / 2;
                    let next_tent = if x < half {
                        2 * x
                    } else {
                        2 * (m - x)
                    };

                    // SipHash-inspired non-linear mixing for strong Avalanche effect.
                    // Combines coordinate indices, round counter, and tent map output
                    // through multiply-xor-shift operations for thorough bit diffusion.
                    let mix_key = (i as u64)
                        .wrapping_mul(0x517cc1b727220a95)
                        ^ (j as u64).wrapping_mul(0x6c62272e07bb0142)
                        ^ (round as u64).wrapping_mul(0x9e3779b97f4a7c15);
                    let mixed = (next_tent as u64).wrapping_add(mix_key);
                    let mixed = mixed ^ (mixed >> 17);
                    let mixed = mixed.wrapping_mul(0xbf58476d1ce4e5b9);
                    let mixed = mixed ^ (mixed >> 31);

                    x = (mixed % (m as u64 + 1)) as i64;
                }

                // Shift back to signed range [-L, L]
                result[i][j] = (x - self.l as i64) as i32;
            }
        }
        result
    }

    /// Computes a 32-byte fingerprint of the current weight state.
    ///
    /// Useful for quickly comparing weight matrices without transmitting
    /// the full weight vector over the wire.
    pub fn weight_fingerprint(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        for row in &self.weights {
            for &w in row {
                hasher.update(w.to_le_bytes());
            }
        }
        hasher.finalize().to_vec()
    }
}
