//! E-TPM (Enhanced Tree Parity Machine) Core Module.
//!
//! Implements a hardened artificial neural network structure for public key exchange.
//! Supports standard sign activation, chaotic sine activation, constant-time weight
//! updating, and secure memory zeroization.

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::boxed::Box;

use crate::constant_time::ct_select_i32;

/// Result type alias supporting both PyO3 and pure Rust environments.
#[cfg(feature = "extension-module")]
type ETPMResult<T> = PyResult<T>;

#[cfg(not(feature = "extension-module"))]
type ETPMResult<T> = Result<T, &'static str>;

#[cfg(feature = "extension-module")]
macro_rules! make_err {
    ($msg:expr) => {
        pyo3::exceptions::PyValueError::new_err($msg)
    };
}

#[cfg(not(feature = "extension-module"))]
macro_rules! make_err {
    ($msg:expr) => {
        $msg
    };
}

#[cfg_attr(feature = "extension-module", pyclass(eq, eq_int))]
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
            "antihebbian" => Some(Self::AntiHebbian),
            "randomwalk" => Some(Self::RandomWalk),
            _ => None,
        }
    }
}

#[cfg_attr(feature = "extension-module", pyclass(eq, eq_int))]
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ActivationType {
    /// Standard step sign activation: σ(h) = sign(h).
    Standard,
    /// Pure chaotic activation: σ(h) = sign(sin(π·h/(2L))).
    Chaotic,
    /// Hybrid mode: Standard sign activation is used for synchronization
    /// convergence, then applies chaotic weight transformation for key hardening.
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

/// Core E-TPM Neural Network Structure.
#[cfg_attr(feature = "extension-module", pyclass)]
#[derive(Clone, Debug, PartialEq)]
pub struct ETPM {
    /// Number of hidden units (K).
    pub k: usize,
    /// Number of input neurons per unit (N).
    pub n: usize,
    /// Synaptic depth limit (L).
    pub l: i32,
    /// Synaptic weight matrices: shape [K][N].
    pub weights: Vec<Vec<i32>>,
    /// Output states of the K hidden units.
    pub outputs: Vec<i32>,
    /// Last processed input matrix: shape [K][N].
    pub last_input: Vec<Vec<i32>>,
    /// Selected activation function.
    pub activation_type: ActivationType,
}

/// Securely wipe E-TPM weights from memory when dropped.
impl Drop for ETPM {
    fn drop(&mut self) {
        for row in &mut self.weights {
            row.zeroize();
        }
    }
}

impl ETPM {
    pub fn new(k: usize, n: usize, l: i32, activation_type: &str) -> ETPMResult<Self> {
        // Parameter validation to prevent degenerate or dangerous configurations.
        if k == 0 {
            return Err(make_err!("K (hidden units) must be >= 1"));
        }
        if n == 0 {
            return Err(make_err!("N (inputs per unit) must be >= 1"));
        }
        if l <= 0 {
            return Err(make_err!("L (synaptic depth) must be >= 1"));
        }

        let act_type = ActivationType::from_str(activation_type)
            .ok_or_else(|| make_err!("Invalid activation type. Choose 'standard', 'chaotic', or 'hybrid'."))?;

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

        // Randomly initialize weights using default secure OS RNG
        etpm.initialize_weights(None)?;

        Ok(etpm)
    }

    /// Initializes or randomizes weights. If a seed is provided, a deterministic RNG (ChaCha8) is used.
    pub fn initialize_weights(&mut self, seed: Option<u64>) -> ETPMResult<()> {
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
    pub fn calculate_output(&mut self, inputs: Vec<Vec<i32>>) -> ETPMResult<i32> {
        if inputs.len() != self.k {
            return Err(make_err!("Input row count must match K"));
        }

        for row in inputs.iter() {
            if row.len() != self.n {
                return Err(make_err!("Input column count must match N"));
            }
            for &val in row.iter() {
                if val != 1 && val != -1 {
                    return Err(make_err!("Input values must be either -1 or 1"));
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
                    // Pure chaotic activation: sign of sin(π·h/(2L)).
                    // Computed using pure integer modulo to support no_std without float dependencies.
                    let two_l = 2 * self.l;
                    let h_mod = h.rem_euclid(2 * two_l);
                    if h_mod < two_l {
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
    pub fn update_weights(&mut self, tau: i32, rule: &str) -> ETPMResult<()> {
        let rule_enum = UpdateRule::from_str(rule).ok_or_else(|| {
            make_err!("Invalid update rule. Choose 'hebbian', 'antihebbian', or 'randomwalk'.")
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

                let new_w = w_ij + delta;

                // Constant-time clamp:
                let clamped = new_w.clamp(-self.l, self.l);

                // Constant-time update select using bitwise selection
                self.weights[i][j] = ct_select_i32(match_condition, clamped, w_ij);
            }
        }
        Ok(())
    }

    pub fn scale_synaptic_depth(&mut self, new_l: i32) -> ETPMResult<()> {
        if new_l <= self.l {
            return Err(make_err!("New synaptic depth L must be greater than current L"));
        }

        let scale = new_l as f64 / self.l as f64;
        for i in 0..self.k {
            for j in 0..self.n {
                let val = self.weights[i][j] as f64 * scale;
                let scaled_w = if val >= 0.0 { (val + 0.5) as i32 } else { (val - 0.5) as i32 };
                self.weights[i][j] = scaled_w.clamp(-new_l, new_l);
            }
        }
        self.l = new_l;
        Ok(())
    }

    pub fn get_weights(&self) -> Vec<Vec<i32>> {
        self.weights.clone()
    }

    pub fn set_weights(&mut self, weights: Vec<Vec<i32>>) -> ETPMResult<()> {
        if weights.len() != self.k {
            return Err(make_err!("Weight row count must match K"));
        }
        for row in weights.iter() {
            if row.len() != self.n {
                return Err(make_err!("Weight column count must match N"));
            }
            for &val in row.iter() {
                if val.abs() > self.l {
                    return Err(make_err!("Weight value exceeds synaptic depth L"));
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
    pub fn weight_fingerprint(&self) -> Vec<u8> {
        let mut hasher = Sha256::new();
        for row in &self.weights {
            for &w in row {
                hasher.update(w.to_le_bytes());
            }
        }
        hasher.finalize().to_vec()
    }

    /// Computes the local field values h_i for each hidden unit.
    pub fn calculate_local_fields(&self, inputs: &Vec<Vec<i32>>) -> Vec<i32> {
        let mut fields = vec![0; self.k];
        for i in 0..self.k {
            let mut sum = 0;
            for j in 0..self.n {
                sum += self.weights[i][j] * inputs[i][j];
            }
            fields[i] = sum;
        }
        fields
    }
}

// ---------------------------------------------------------------------------
// Python bindings
// ---------------------------------------------------------------------------
#[cfg(feature = "extension-module")]
#[pymethods]
impl ETPM {
    #[new]
    #[pyo3(signature = (k, n, l, activation_type = "hybrid"))]
    pub fn py_new(k: usize, n: usize, l: i32, activation_type: &str) -> ETPMResult<Self> {
        Self::new(k, n, l, activation_type)
    }

    #[pyo3(name = "initialize_weights")]
    #[pyo3(signature = (seed = None))]
    pub fn py_initialize_weights(&mut self, seed: Option<u64>) -> ETPMResult<()> {
        self.initialize_weights(seed)
    }

    #[pyo3(name = "calculate_output")]
    pub fn py_calculate_output(&mut self, inputs: Vec<Vec<i32>>) -> ETPMResult<i32> {
        self.calculate_output(inputs)
    }

    #[pyo3(name = "update_weights")]
    #[pyo3(signature = (tau, rule = "hebbian"))]
    pub fn py_update_weights(&mut self, tau: i32, rule: &str) -> ETPMResult<()> {
        self.update_weights(tau, rule)
    }

    #[pyo3(name = "scale_synaptic_depth")]
    pub fn py_scale_synaptic_depth(&mut self, new_l: i32) -> ETPMResult<()> {
        self.scale_synaptic_depth(new_l)
    }

    #[pyo3(name = "get_weights")]
    pub fn py_get_weights(&self) -> Vec<Vec<i32>> {
        self.get_weights()
    }

    #[pyo3(name = "set_weights")]
    pub fn py_set_weights(&mut self, weights: Vec<Vec<i32>>) -> ETPMResult<()> {
        self.set_weights(weights)
    }

    #[pyo3(name = "get_hidden_outputs")]
    pub fn py_get_hidden_outputs(&self) -> Vec<i32> {
        self.get_hidden_outputs()
    }

    #[pyo3(name = "chaotic_transform")]
    pub fn py_chaotic_transform(&self, iterations: u32) -> Vec<Vec<i32>> {
        self.chaotic_transform(iterations)
    }

    #[pyo3(name = "weight_fingerprint")]
    pub fn py_weight_fingerprint(&self) -> Vec<u8> {
        self.weight_fingerprint()
    }

    #[pyo3(name = "calculate_local_fields")]
    pub fn py_calculate_local_fields(&self, inputs: Vec<Vec<i32>>) -> Vec<i32> {
        self.calculate_local_fields(&inputs)
    }

    #[getter]
    pub fn k(&self) -> usize {
        self.k
    }

    #[getter]
    pub fn n(&self) -> usize {
        self.n
    }

    #[getter]
    pub fn l(&self) -> i32 {
        self.l
    }

    #[getter]
    pub fn activation_type(&self) -> ActivationType {
        self.activation_type
    }
}
