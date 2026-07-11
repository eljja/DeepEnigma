//! Security Analysis module for E-TPM key exchange.
//!
//! Provides attack simulations (passive eavesdropper, geometric attack) and
//! information-theoretic metrics (Shannon entropy, weight overlap) to evaluate
//! the resilience of the neural key exchange protocol.

use pyo3::prelude::*;
use rand::Rng;

use crate::etpm::ETPM;
use crate::protocol::compute_overlap;

/// Result of a simulated attack against the key exchange protocol.
#[pyclass]
#[derive(Clone, Debug)]
pub struct AttackResult {
    /// Name of the attack strategy used.
    #[pyo3(get)]
    pub attack_type: String,
    /// Whether the attacker achieved full synchronization with one of the parties.
    #[pyo3(get)]
    pub success: bool,
    /// Number of rounds the attack ran.
    #[pyo3(get)]
    pub rounds: u32,
    /// Final weight overlap between the attacker and Alice (0.0 to 1.0).
    #[pyo3(get)]
    pub final_overlap: f64,
    /// Sum of absolute element-wise weight differences between attacker and Alice.
    #[pyo3(get)]
    pub weight_difference: i64,
}

#[pymethods]
impl AttackResult {
    fn __repr__(&self) -> String {
        format!(
            "AttackResult(attack_type=\"{}\", success={}, rounds={}, final_overlap={:.4}, weight_difference={})",
            self.attack_type, self.success, self.rounds, self.final_overlap, self.weight_difference
        )
    }
}

/// Provides security analysis tools for evaluating E-TPM key exchange resilience.
#[pyclass]
pub struct SecurityAnalyzer {
    k: usize,
    n: usize,
    l: i32,
}

#[pymethods]
impl SecurityAnalyzer {
    #[new]
    pub fn new(k: usize, n: usize, l: i32) -> Self {
        Self { k, n, l }
    }

    /// Simulates a **passive eavesdropper** attack.
    ///
    /// The attacker (Eve) maintains her own E-TPM and observes the public inputs
    /// and outputs of Alice and Bob. Whenever Alice and Bob agree (τ_A == τ_B)
    /// and Eve's output also matches, Eve updates her weights using the same
    /// rule. This models an eavesdropper who can only listen on the public channel.
    pub fn run_passive_attack(&mut self, max_rounds: u32) -> PyResult<AttackResult> {
        let mut rng = rand::thread_rng();
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
    ///
    /// This is a stronger attack where Eve, after observing matching outputs,
    /// identifies the hidden unit with the smallest absolute local field |h_i|
    /// (most uncertain unit) and flips its output before updating weights.
    /// This strategy attempts to accelerate Eve's synchronization with Alice.
    pub fn run_geometric_attack(&mut self, max_rounds: u32) -> PyResult<AttackResult> {
        let mut rng = rand::thread_rng();
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
    ///
    /// Returns entropy in bits per byte (maximum 8.0 for a uniformly random key).
    /// Higher values indicate better key quality.
    pub fn measure_key_entropy(&self, key: Vec<u8>) -> f64 {
        shannon_entropy(&key)
    }

    /// Computes the fraction of element-wise matching weights between two weight matrices.
    ///
    /// Returns a value in [0.0, 1.0] where 1.0 means all weights are identical.
    #[staticmethod]
    pub fn compute_weight_overlap(w1: Vec<Vec<i32>>, w2: Vec<Vec<i32>>) -> f64 {
        compute_overlap(&w1, &w2)
    }
}

// ---------------------------------------------------------------------------
// Internal helpers (not exposed to Python)
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
    counts
        .iter()
        .filter(|&&c| c > 0)
        .map(|&c| {
            let p = c as f64 / len;
            -p * p.log2()
        })
        .sum()
}
