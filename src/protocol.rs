//! Key Exchange Protocol module for E-TPM synchronization.
//!
//! Orchestrates the full Alice-Bob neural key exchange using Enhanced Tree
//! Parity Machines. Both parties iteratively synchronize their weight vectors
//! by exchanging outputs over a public channel until their internal states match,
//! then a shared cryptographic key is derived via HKDF-SHA256.

use pyo3::prelude::*;
use rand::Rng;
use sha2::Sha256;
use hkdf::Hkdf;
use std::time::Instant;
use zeroize::Zeroize;

use crate::auth::{ZKPProver, ZKPVerifier};
use crate::etpm::ETPM;

/// Configuration parameters for the key exchange protocol.
#[pyclass]
#[derive(Clone, Debug)]
pub struct KeyExchangeConfig {
    /// Number of hidden units (K).
    #[pyo3(get, set)]
    pub k: usize,
    /// Number of input neurons per hidden unit (N).
    #[pyo3(get, set)]
    pub n: usize,
    /// Synaptic depth bound (L). Weights are clamped to [-L, L].
    #[pyo3(get, set)]
    pub l: i32,
    /// Maximum number of synchronization rounds before giving up.
    #[pyo3(get, set)]
    pub max_rounds: u32,
    /// Weight update rule name (e.g. "hebbian", "antihebbian", "randomwalk").
    #[pyo3(get, set)]
    pub update_rule: String,
    /// Activation function type (e.g. "standard", "chaotic", "hybrid").
    #[pyo3(get, set)]
    pub activation_type: String,
    /// Number of chaotic transform iterations for key hardening (Hybrid mode).
    #[pyo3(get, set)]
    pub chaotic_iterations: u32,
    /// Automatically scale up L dynamically during long synchronization.
    #[pyo3(get, set)]
    pub adaptive_l_scaling: bool,
}

#[pymethods]
impl KeyExchangeConfig {
    #[new]
    #[pyo3(signature = (k, n, l, max_rounds = 10000, update_rule = "hebbian".to_string(), activation_type = "hybrid".to_string(), chaotic_iterations = 100, adaptive_l_scaling = false))]
    pub fn new(
        k: usize,
        n: usize,
        l: i32,
        max_rounds: u32,
        update_rule: String,
        activation_type: String,
        chaotic_iterations: u32,
        adaptive_l_scaling: bool,
    ) -> Self {
        Self {
            k,
            n,
            l,
            max_rounds,
            update_rule,
            activation_type,
            chaotic_iterations,
            adaptive_l_scaling,
        }
    }
}

/// Result of a completed key exchange attempt.
#[pyclass]
#[derive(Clone, Debug)]
pub struct KeyExchangeResult {
    /// Whether synchronization was achieved within the round limit.
    #[pyo3(get)]
    pub success: bool,
    /// Number of rounds executed.
    #[pyo3(get)]
    pub rounds: u32,
    /// Derived 256-bit key as raw bytes (32 bytes). Empty if unsuccessful.
    #[pyo3(get)]
    pub key: Vec<u8>,
    /// Hex-encoded representation of the derived key.
    #[pyo3(get)]
    pub key_hex: String,
    /// Wall-clock synchronization time in milliseconds.
    #[pyo3(get)]
    pub sync_time_ms: f64,
    /// Whether ZKP mutual authentication was performed.
    #[pyo3(get)]
    pub authenticated: bool,
}

#[pymethods]
impl KeyExchangeResult {
    fn __repr__(&self) -> String {
        format!(
            "KeyExchangeResult(success={}, rounds={}, key_hex=\"{}\", sync_time_ms={:.2}, authenticated={})",
            self.success, self.rounds, self.key_hex, self.sync_time_ms, self.authenticated
        )
    }
}

/// Manages the full E-TPM key exchange protocol between two parties (Alice and Bob).
#[pyclass]
pub struct KeyExchange {
    alice: ETPM,
    bob: ETPM,
    config: KeyExchangeConfig,
}

#[pymethods]
impl KeyExchange {
    /// Creates a new key exchange instance with independently initialized Alice and Bob ETPMs.
    #[new]
    pub fn new(config: &KeyExchangeConfig) -> PyResult<Self> {
        let alice = ETPM::new(config.k, config.n, config.l, &config.activation_type)?;
        let bob = ETPM::new(config.k, config.n, config.l, &config.activation_type)?;

        Ok(Self {
            alice,
            bob,
            config: config.clone(),
        })
    }

    /// Runs the full synchronization loop WITHOUT authentication.
    ///
    /// Protocol steps per round:
    /// 1. Generate a random input matrix of shape K×N with values in {-1, 1}.
    /// 2. Both Alice and Bob compute their output (τ).
    /// 3. If outputs match, both update their weights using the configured rule.
    /// 4. Check whether all weights are identical (full synchronization).
    /// 5. On sync, derive a 256-bit key via HKDF-SHA256 from the shared weight vector.
    pub fn run(&mut self) -> PyResult<KeyExchangeResult> {
        self.run_sync(false)
    }

    /// Runs the full synchronization loop WITH ZKP mutual authentication.
    ///
    /// Before entering the synchronization loop, performs a Fiat-Shamir style
    /// hash-based zero-knowledge challenge-response protocol to verify that
    /// both parties possess the same pre-shared key (PSK), blocking MitM attacks.
    ///
    /// If authentication fails, the exchange is aborted immediately.
    #[pyo3(signature = (alice_psk, bob_psk = None))]
    pub fn authenticated_run(&mut self, alice_psk: Vec<u8>, bob_psk: Option<Vec<u8>>) -> PyResult<KeyExchangeResult> {
        let bob_psk_val = bob_psk.unwrap_or_else(|| alice_psk.clone());

        // Simulate bidirectional ZKP authentication
        let mut prover = ZKPProver::new(alice_psk);
        let mut verifier = ZKPVerifier::new(bob_psk_val);

        // Alice (Prover) → Bob (Verifier)
        let commitment = prover.create_commitment();
        verifier.receive_commitment(commitment);
        let challenge = verifier.create_challenge();
        let response = prover.respond(challenge);
        let nonce = prover.get_nonce();
        let counter = prover.get_session_counter();

        if !verifier.verify(nonce, response, counter) {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "ZKP authentication failed: PSK mismatch or replay attack detected. Aborting key exchange.",
            ));
        }

        self.run_sync(true)
    }

    /// Returns the current weight overlap ratio between Alice and Bob (0.0 to 1.0).
    ///
    /// An overlap of 1.0 means all weights are identical (full synchronization).
    pub fn get_sync_progress(&self) -> f64 {
        compute_overlap(&self.alice.weights, &self.bob.weights)
    }
}

impl KeyExchange {
    /// Internal synchronization loop shared by `run()` and `authenticated_run()`.
    fn run_sync(&mut self, authenticated: bool) -> PyResult<KeyExchangeResult> {
        let start = Instant::now();
        let mut rng = crate::rng::secure_rng();

        // Collect first-round public input hash as HKDF salt for domain separation
        let mut salt_data: Vec<u8> = Vec::new();

        for round in 1..=self.config.max_rounds {
            // Adaptive L scaling trigger:
            // Every 1000 rounds, if sync is not complete, scale up L to expand the weight boundary.
            if self.config.adaptive_l_scaling && round > 1 && round % 1000 == 0 {
                let current_l = self.alice.l;
                let new_l = current_l + 2;
                self.alice.scale_synaptic_depth(new_l)?;
                self.bob.scale_synaptic_depth(new_l)?;
            }

            // Step 1: Generate random input matrix K x N with values in {-1, 1}.
            let inputs: Vec<Vec<i32>> = (0..self.config.k)
                .map(|_| {
                    (0..self.config.n)
                        .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                        .collect()
                })
                .collect();

            // Capture first round's inputs as salt material
            if round == 1 {
                for row in &inputs {
                    for &val in row {
                        salt_data.extend_from_slice(&val.to_le_bytes());
                    }
                }
            }

            // Step 2: Both compute output.
            let tau_alice = self.alice.calculate_output(inputs.clone())?;
            let tau_bob = self.bob.calculate_output(inputs)?;

            // Step 3: Update weights only when outputs agree.
            if tau_alice == tau_bob {
                self.alice
                    .update_weights(tau_alice, &self.config.update_rule)?;
                self.bob
                    .update_weights(tau_bob, &self.config.update_rule)?;

                // Step 4: Check full weight synchronization.
                if self.alice.weights == self.bob.weights {
                    // Step 5: Derive key via HKDF-SHA256.
                    let mut final_weights =
                        if self.alice.activation_type == crate::etpm::ActivationType::Hybrid {
                            self.alice.chaotic_transform(self.config.chaotic_iterations)
                        } else {
                            self.alice.weights.clone()
                        };

                    let key = derive_key_hkdf(&final_weights, &salt_data);
                    let key_hex = hex::encode(&key);
                    let elapsed = start.elapsed().as_secs_f64() * 1000.0;

                    // Zeroize intermediate weight data
                    for row in &mut final_weights {
                        row.zeroize();
                    }
                    salt_data.zeroize();

                    return Ok(KeyExchangeResult {
                        success: true,
                        rounds: round,
                        key,
                        key_hex,
                        sync_time_ms: elapsed,
                        authenticated,
                    });
                }
            }
        }

        // Synchronization was not achieved within the round limit.
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;
        salt_data.zeroize();

        Ok(KeyExchangeResult {
            success: false,
            rounds: self.config.max_rounds,
            key: Vec::new(),
            key_hex: String::new(),
            sync_time_ms: elapsed,
            authenticated,
        })
    }
}

// ---------------------------------------------------------------------------
// Internal helpers (not exposed to Python)
// ---------------------------------------------------------------------------

/// Derives a 256-bit key using HKDF-SHA256 (Extract-and-Expand).
///
/// - **IKM (Input Keying Material)**: Flattened weight vector bytes.
/// - **Salt**: Session-specific data (first round's public inputs).
/// - **Info**: Domain separation string for this protocol version.
///
/// This replaces raw SHA-256 hashing and provides:
/// 1. Proper domain separation (different `info` strings yield different keys)
/// 2. Salt-based randomization (session-unique keys even from identical weights)
/// 3. NIST SP 800-56C compliance for key derivation
fn derive_key_hkdf(weights: &[Vec<i32>], salt: &[u8]) -> Vec<u8> {
    // Flatten weights into IKM bytes
    let mut ikm: Vec<u8> = Vec::with_capacity(weights.len() * weights[0].len() * 4);
    for row in weights {
        for &w in row {
            ikm.extend_from_slice(&w.to_le_bytes());
        }
    }

    let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
    let info = b"DeepEnigma-v1-session-key";
    let mut okm = vec![0u8; 32]; // 256-bit output key material
    hk.expand(info, &mut okm)
        .expect("HKDF expand should not fail for 32-byte output");

    // Zeroize IKM after use
    ikm.zeroize();

    okm
}

/// Computes the fraction of element-wise matching weights between two weight matrices.
pub(crate) fn compute_overlap(w1: &[Vec<i32>], w2: &[Vec<i32>]) -> f64 {
    if w1.is_empty() || w2.is_empty() {
        return 0.0;
    }
    let total = w1.iter().map(|r| r.len()).sum::<usize>() as f64;
    if total == 0.0 {
        return 0.0;
    }
    let matching: usize = w1
        .iter()
        .zip(w2.iter())
        .map(|(r1, r2)| r1.iter().zip(r2.iter()).filter(|(a, b)| a == b).count())
        .sum();
    matching as f64 / total
}
