use pyo3::prelude::*;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

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

#[pymethods]
impl ETPM {
    #[new]
    #[pyo3(signature = (k, n, l, activation_type = "hybrid"))]
    pub fn new(k: usize, n: usize, l: i32, activation_type: &str) -> PyResult<Self> {
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
            None => Box::new(thread_rng()),
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
    /// Only updates weights of hidden units whose output matches the overall parity output tau.
    #[pyo3(signature = (tau, rule = "chaotic"))]
    pub fn update_weights(&mut self, tau: i32, rule: &str) -> PyResult<()> {
        let rule_enum = match rule.to_lowercase().as_str() {
            "chaotic" => {
                // If "chaotic" is passed, we select a rule deterministically using the hash of weights,
                // or just fallback. Let's make it select a rule based on weight sum parity to make it dynamic.
                let sum: i32 = self.weights.iter().map(|row| row.iter().sum::<i32>()).sum();
                match sum.abs() % 3 {
                    0 => UpdateRule::Hebbian,
                    1 => UpdateRule::AntiHebbian,
                    _ => UpdateRule::RandomWalk,
                }
            }
            s => UpdateRule::from_str(s).ok_or_else(|| {
                pyo3::exceptions::PyValueError::new_err(
                    "Invalid update rule. Choose 'hebbian', 'antihebbian', 'randomwalk', or 'chaotic'.",
                )
            })?,
        };

        for i in 0..self.k {
            if self.outputs[i] == tau {
                for j in 0..self.n {
                    let w_ij = self.weights[i][j];
                    let x_ij = self.last_input[i][j];

                    let new_w = match rule_enum {
                        UpdateRule::Hebbian => w_ij + x_ij * tau,
                        UpdateRule::AntiHebbian => w_ij - x_ij * tau,
                        UpdateRule::RandomWalk => w_ij + x_ij,
                    };

                    // Clip weights to [-L, L]
                    self.weights[i][j] = new_w.clamp(-self.l, self.l);
                }
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

    /// Applies a chaotic integer-only Tent Map transformation to the weight matrix.
    ///
    /// This is used in Hybrid mode after synchronization to harden the key
    /// before SHA-256 derivation. Using integer-only arithmetic guarantees that
    /// the key exchange is 100% deterministic and yields identical keys on all
    /// CPU architectures (e.g. x86, ARM, RISC-V), preventing float-discrepancies.
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
                    let mut next_x = if x < half {
                        2 * x
                    } else {
                        2 * (m - x)
                    };

                    // Apply coordinate and round-based non-linear mixing/diffusion
                    next_x = (next_x ^ (i as i64) ^ (j as i64) ^ (round as i64)) % (m + 1);
                    x = next_x;
                }

                // Shift back to signed range [-L, L]
                result[i][j] = (x - self.l as i64) as i32;
            }
        }
        result
    }
}
