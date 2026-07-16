//! Security Analysis module for E-TPM key exchange.
//!
//! Provides attack simulations (passive eavesdropper, geometric attack) and
//! information-theoretic metrics (Shannon entropy, weight overlap) to evaluate
//! the resilience of the neural key exchange protocol.

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;
use rand::Rng;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;
#[cfg(not(feature = "std"))]
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::etpm::ETPM;
use crate::protocol::compute_overlap;

/// Result type alias supporting both PyO3 and pure Rust environments.
#[cfg(feature = "extension-module")]
type SecurityResult<T> = PyResult<T>;

#[cfg(not(feature = "extension-module"))]
type SecurityResult<T> = Result<T, &'static str>;



/// Result of a simulated attack against the key exchange protocol.
#[cfg_attr(feature = "extension-module", pyclass)]
#[derive(Clone, Debug)]
pub struct AttackResult {
    /// Name of the attack strategy used.
    pub attack_type: String,
    /// Whether the attacker achieved full synchronization with one of the parties.
    pub success: bool,
    /// Number of rounds the attack ran.
    pub rounds: u32,
    /// Final weight overlap between the attacker and Alice (0.0 to 1.0).
    pub final_overlap: f64,
    /// Sum of absolute element-wise weight differences between attacker and Alice.
    pub weight_difference: i64,
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl AttackResult {
    fn __repr__(&self) -> String {
        format!(
            "AttackResult(attack_type=\"{}\", success={}, rounds={}, final_overlap={:.4}, weight_difference={})",
            self.attack_type, self.success, self.rounds, self.final_overlap, self.weight_difference
        )
    }

    #[getter]
    pub fn attack_type(&self) -> String {
        self.attack_type.clone()
    }

    #[getter]
    pub fn success(&self) -> bool {
        self.success
    }

    #[getter]
    pub fn rounds(&self) -> u32 {
        self.rounds
    }

    #[getter]
    pub fn final_overlap(&self) -> f64 {
        self.final_overlap
    }

    #[getter]
    pub fn weight_difference(&self) -> i64 {
        self.weight_difference
    }
}

/// Provides security analysis tools for evaluating E-TPM key exchange resilience.
#[cfg_attr(feature = "extension-module", pyclass)]
pub struct SecurityAnalyzer {
    k: usize,
    n: usize,
    l: i32,
}

impl SecurityAnalyzer {
    pub fn new(k: usize, n: usize, l: i32) -> Self {
        Self { k, n, l }
    }

    /// Simulates a **passive eavesdropper** attack.
    pub fn run_passive_attack(&mut self, max_rounds: u32) -> SecurityResult<AttackResult> {
        let mut rng = crate::rng::secure_rng();
        let update_rule = "hebbian";

        let mut alice = ETPM::new(self.k, self.n, self.l, "hybrid")?;
        let mut bob = ETPM::new(self.k, self.n, self.l, "hybrid")?;
        let mut eve = ETPM::new(self.k, self.n, self.l, "hybrid")?;

        for round in 1..=max_rounds {
            let inputs: Vec<Vec<i32>> = (0..self.k)
                .map(|_| {
                    (0..self.n)
                        .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                        .collect()
                })
                .collect();

            let tau_alice = alice.calculate_output(inputs.clone())?;
            let tau_bob = bob.calculate_output(inputs.clone())?;
            let tau_eve = eve.calculate_output(inputs)?;

            // Alice and Bob update when they agree.
            if tau_alice == tau_bob {
                alice.update_weights(tau_alice, update_rule)?;
                bob.update_weights(tau_bob, update_rule)?;

                // Eve can only update when her output also matches the public τ.
                if tau_eve == tau_alice {
                    eve.update_weights(tau_eve, update_rule)?;
                }

                // Check if Alice and Bob have synchronised.
                if alice.weights == bob.weights {
                    let overlap = compute_overlap(&eve.weights, &alice.weights);
                    let diff = weight_diff(&eve.weights, &alice.weights);
                    return Ok(AttackResult {
                        attack_type: "passive".to_string(),
                        success: eve.weights == alice.weights,
                        rounds: round,
                        final_overlap: overlap,
                        weight_difference: diff,
                    });
                }
            }
        }

        let overlap = compute_overlap(&eve.weights, &alice.weights);
        let diff = weight_diff(&eve.weights, &alice.weights);
        Ok(AttackResult {
            attack_type: "passive".to_string(),
            success: false,
            rounds: max_rounds,
            final_overlap: overlap,
            weight_difference: diff,
        })
    }

    /// Simulates a **geometric attack**.
    pub fn run_geometric_attack(&mut self, max_rounds: u32) -> SecurityResult<AttackResult> {
        let mut rng = crate::rng::secure_rng();
        let update_rule = "hebbian";

        let mut alice = ETPM::new(self.k, self.n, self.l, "hybrid")?;
        let mut bob = ETPM::new(self.k, self.n, self.l, "hybrid")?;
        let mut eve = ETPM::new(self.k, self.n, self.l, "hybrid")?;

        for round in 1..=max_rounds {
            let inputs: Vec<Vec<i32>> = (0..self.k)
                .map(|_| {
                    (0..self.n)
                        .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                        .collect()
                })
                .collect();

            let tau_alice = alice.calculate_output(inputs.clone())?;
            let tau_bob = bob.calculate_output(inputs.clone())?;
            let _tau_eve = eve.calculate_output(inputs.clone())?;

            if tau_alice == tau_bob {
                alice.update_weights(tau_alice, update_rule)?;
                bob.update_weights(tau_bob, update_rule)?;

                // Geometric attack: find the hidden unit with the smallest |h_i|
                // and flip its output to try to match Alice's τ, then update.
                let eve_outputs = eve.get_hidden_outputs();

                // Compute local fields h_i for Eve.
                let local_fields: Vec<i32> = (0..self.k)
                    .map(|i| {
                        eve.weights[i]
                            .iter()
                            .zip(inputs[i].iter())
                            .map(|(&w, &x)| w * x)
                            .sum()
                    })
                    .collect();

                // Product of Eve's hidden outputs.
                let tau_eve_actual: i32 = eve_outputs.iter().product();

                if tau_eve_actual != tau_alice {
                    // Find the unit with the smallest |h_i| and flip it.
                    if let Some(flip_idx) = local_fields
                        .iter()
                        .enumerate()
                        .min_by_key(|(_, h)| h.abs())
                        .map(|(i, _)| i)
                    {
                        eve.outputs[flip_idx] = -eve.outputs[flip_idx];
                    }
                }

                eve.update_weights(tau_alice, update_rule)?;

                if alice.weights == bob.weights {
                    let overlap = compute_overlap(&eve.weights, &alice.weights);
                    let diff = weight_diff(&eve.weights, &alice.weights);
                    return Ok(AttackResult {
                        attack_type: "geometric".to_string(),
                        success: eve.weights == alice.weights,
                        rounds: round,
                        final_overlap: overlap,
                        weight_difference: diff,
                    });
                }
            }
        }

        let overlap = compute_overlap(&eve.weights, &alice.weights);
        let diff = weight_diff(&eve.weights, &alice.weights);
        Ok(AttackResult {
            attack_type: "geometric".to_string(),
            success: false,
            rounds: max_rounds,
            final_overlap: overlap,
            weight_difference: diff,
        })
    }

    /// Measures the Shannon entropy of a key byte sequence.
    pub fn measure_key_entropy(&self, key: Vec<u8>) -> f64 {
        shannon_entropy(&key)
    }

    /// Computes the fraction of element-wise matching weights between two weight matrices.
    pub fn compute_weight_overlap(w1: Vec<Vec<i32>>, w2: Vec<Vec<i32>>) -> f64 {
        compute_overlap(&w1, &w2)
    }

    /// Computes LWE-based security metrics based on dimensions and quantization parameters (Part 2).
    pub fn estimate_lwe_security(&self, dimension: usize, modulus: usize, error_std_dev: f64) -> LweSecurityMetrics {
        let n = dimension as f64;
        let q = modulus as f64;
        let sigma = error_std_dev.max(0.1);

        let ratio = q / sigma;
        let log_ratio = safe_log2(ratio);

        // BKZ block size needed to solve this LWE instance (estimate using LWE hardness heuristics)
        let beta = (log_ratio * 1.85).max(10.0);
        
        // Classical BKZ security level: 0.265 * beta * sqrt(n / ref_dimension)
        let classical = 0.265 * beta * (n / 500.0).sqrt();
        // Quantum BKZ security level (core-SVP model): 0.229 * beta * sqrt(n / ref_dimension)
        let quantum = 0.229 * beta * (n / 500.0).sqrt();


        LweSecurityMetrics {
            dimension,
            modulus,
            error_std_dev,
            classical_security_bits: classical,
            quantum_security_bits: quantum,
        }
    }
}

/// LWE Security Evaluation Metrics for Neural Cryptography.
#[cfg_attr(feature = "extension-module", pyclass)]
#[derive(Clone, Debug)]
pub struct LweSecurityMetrics {
    pub dimension: usize,
    pub modulus: usize,
    pub error_std_dev: f64,
    pub classical_security_bits: f64,
    pub quantum_security_bits: f64,
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl LweSecurityMetrics {
    #[getter]
    pub fn dimension(&self) -> usize {
        self.dimension
    }

    #[getter]
    pub fn modulus(&self) -> usize {
        self.modulus
    }

    #[getter]
    pub fn error_std_dev(&self) -> f64 {
        self.error_std_dev
    }

    #[getter]
    pub fn classical_security_bits(&self) -> f64 {
        self.classical_security_bits
    }

    #[getter]
    pub fn quantum_security_bits(&self) -> f64 {
        self.quantum_security_bits
    }
}

// Python bindings for SecurityAnalyzer
#[cfg(feature = "extension-module")]
#[pymethods]
impl SecurityAnalyzer {
    #[new]
    pub fn py_new(k: usize, n: usize, l: i32) -> Self {
        Self::new(k, n, l)
    }

    #[pyo3(name = "run_passive_attack")]
    pub fn py_run_passive_attack(&mut self, max_rounds: u32) -> SecurityResult<AttackResult> {
        self.run_passive_attack(max_rounds)
    }

    #[pyo3(name = "run_geometric_attack")]
    pub fn py_run_geometric_attack(&mut self, max_rounds: u32) -> SecurityResult<AttackResult> {
        self.run_geometric_attack(max_rounds)
    }

    #[pyo3(name = "measure_key_entropy")]
    pub fn py_measure_key_entropy(&self, key: Vec<u8>) -> f64 {
        self.measure_key_entropy(key)
    }

    #[pyo3(name = "compute_weight_overlap")]
    #[staticmethod]
    pub fn py_compute_weight_overlap(w1: Vec<Vec<i32>>, w2: Vec<Vec<i32>>) -> f64 {
        Self::compute_weight_overlap(w1, w2)
    }

    #[pyo3(name = "estimate_lwe_security")]
    pub fn py_estimate_lwe_security(&self, dimension: usize, modulus: usize, error_std_dev: f64) -> LweSecurityMetrics {
        self.estimate_lwe_security(dimension, modulus, error_std_dev)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

/// Sum of absolute element-wise weight differences.
fn weight_diff(w1: &[Vec<i32>], w2: &[Vec<i32>]) -> i64 {
    w1.iter()
        .zip(w2.iter())
        .flat_map(|(r1, r2)| r1.iter().zip(r2.iter()))
        .map(|(&a, &b)| (a as i64 - b as i64).abs())
        .sum()
}

/// Computes Shannon entropy in bits per byte for a byte sequence.
fn shannon_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f64;
    
    // safe_log2 implementation supporting both std and no_std
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * safe_log2(p)
        })
        .sum()
}

/// Helper function to compute log2 in both std and no_std environments.
fn safe_log2(x: f64) -> f64 {
    #[cfg(feature = "std")]
    {
        x.log2()
    }
    #[cfg(not(feature = "std"))]
    {
        if x <= 0.0 {
            return 0.0;
        }
        // Integer-based log2 approximation using bit manipulation.
        // Convert to bits, extract exponent from IEEE 754 representation.
        let bits = x.to_bits();
        let exponent = ((bits >> 52) & 0x7FF) as i64 - 1023;
        let mantissa = (bits & 0x000FFFFFFFFFFFFF) as f64 / (1u64 << 52) as f64;
        // Linear approximation: log2(x) ≈ exponent + mantissa
        exponent as f64 + mantissa
    }
}

