//! Neural Key Exchange Protocol Module.
//!
//! Orchestrates the key exchange simulation between Alice and Bob, synchronising
//! their E-TPM weights over a public channel, and deriving a final key via HKDF.

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;
use rand::Rng;
#[cfg(feature = "std")]
use std::time::Instant;
use sha2::Sha256;
use hkdf::Hkdf;
use zeroize::Zeroize;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::etpm::{ActivationType, ETPM};
use crate::auth::{ZKPProver, ZKPVerifier};

/// Result type alias supporting both PyO3 and pure Rust environments.
#[cfg(feature = "extension-module")]
type ProtocolResult<T> = PyResult<T>;

#[cfg(not(feature = "extension-module"))]
type ProtocolResult<T> = Result<T, &'static str>;

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


#[cfg_attr(feature = "extension-module", pyclass)]
#[derive(Clone, Debug)]
pub struct KeyExchangeResult {
    /// Whether Alice and Bob successfully agreed on the same key.
    pub success: bool,
    /// Total rounds executed before agreement or failure.
    pub rounds: u32,
    /// Deriver key in hex format.
    pub key_hex: String,
    /// Elapsed execution time in milliseconds.
    pub sync_time_ms: f64,
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl KeyExchangeResult {
    fn __repr__(&self) -> String {
        format!(
            "KeyExchangeResult(success={}, rounds={}, key_hex=\"{}\", sync_time_ms={:.2})",
            self.success, self.rounds, self.key_hex, self.sync_time_ms
        )
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
    pub fn key_hex(&self) -> String {
        self.key_hex.clone()
    }

    #[getter]
    pub fn sync_time_ms(&self) -> f64 {
        self.sync_time_ms
    }
}

/// Configuration options for the key exchange.
#[cfg_attr(feature = "extension-module", pyclass)]
#[derive(Clone, Debug)]
pub struct KeyExchangeConfig {
    /// K hidden units.
    pub k: usize,
    /// N inputs per unit.
    pub n: usize,
    /// Synaptic depth limit L.
    pub l: i32,
    /// Maximum allowed rounds before failing.
    pub max_rounds: u32,
    /// Update rule to apply ("hebbian", "antihebbian", "randomwalk").
    pub update_rule: String,
    /// Activation function type ("standard", "chaotic", "hybrid").
    pub activation_type: String,
    /// Number of chaotic transform iterations for key hardening (Hybrid mode).
    pub chaotic_iterations: u32,
    /// Automatically scale up L dynamically during long synchronization.
    pub adaptive_l_scaling: bool,
    /// Active query threshold H. If Some(H), active query selection is enabled.
    pub active_query_threshold: Option<i32>,
}

impl KeyExchangeConfig {
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
            active_query_threshold: None,
        }
    }
}

// Python bindings for KeyExchangeConfig
#[cfg(feature = "extension-module")]
#[pymethods]
impl KeyExchangeConfig {
    #[new]
    #[pyo3(signature = (k, n, l, max_rounds = 10000, update_rule = "hebbian".to_string(), activation_type = "hybrid".to_string(), chaotic_iterations = 100, adaptive_l_scaling = false, active_query_threshold = None))]
    pub fn py_new(
        k: usize,
        n: usize,
        l: i32,
        max_rounds: u32,
        update_rule: String,
        activation_type: String,
        chaotic_iterations: u32,
        adaptive_l_scaling: bool,
        active_query_threshold: Option<i32>,
    ) -> Self {
        let mut cfg = Self::new(k, n, l, max_rounds, update_rule, activation_type, chaotic_iterations, adaptive_l_scaling);
        cfg.active_query_threshold = active_query_threshold;
        cfg
    }

    #[getter]
    pub fn k(&self) -> usize {
        self.k
    }
    #[setter]
    pub fn set_k(&mut self, value: usize) {
        self.k = value;
    }

    #[getter]
    pub fn n(&self) -> usize {
        self.n
    }
    #[setter]
    pub fn set_n(&mut self, value: usize) {
        self.n = value;
    }

    #[getter]
    pub fn l(&self) -> i32 {
        self.l
    }
    #[setter]
    pub fn set_l(&mut self, value: i32) {
        self.l = value;
    }

    #[getter]
    pub fn max_rounds(&self) -> u32 {
        self.max_rounds
    }
    #[setter]
    pub fn set_max_rounds(&mut self, value: u32) {
        self.max_rounds = value;
    }

    #[getter]
    pub fn update_rule(&self) -> String {
        self.update_rule.clone()
    }
    #[setter]
    pub fn set_update_rule(&mut self, value: String) {
        self.update_rule = value;
    }

    #[getter]
    pub fn activation_type(&self) -> String {
        self.activation_type.clone()
    }
    #[setter]
    pub fn set_activation_type(&mut self, value: String) {
        self.activation_type = value;
    }

    #[getter]
    pub fn chaotic_iterations(&self) -> u32 {
        self.chaotic_iterations
    }
    #[setter]
    pub fn set_chaotic_iterations(&mut self, value: u32) {
        self.chaotic_iterations = value;
    }

    #[getter]
    pub fn adaptive_l_scaling(&self) -> bool {
        self.adaptive_l_scaling
    }
    #[setter]
    pub fn set_adaptive_l_scaling(&mut self, value: bool) {
        self.adaptive_l_scaling = value;
    }

    #[getter]
    pub fn active_query_threshold(&self) -> Option<i32> {
        self.active_query_threshold
    }
    #[setter]
    pub fn set_active_query_threshold(&mut self, value: Option<i32>) {
        self.active_query_threshold = value;
    }
}

/// KeyExchange coordinator managing Alice and Bob ETPMs.
#[cfg_attr(feature = "extension-module", pyclass)]
pub struct KeyExchange {
    alice: ETPM,
    bob: ETPM,
    config: KeyExchangeConfig,
}

impl KeyExchange {
    pub fn new(config: &KeyExchangeConfig) -> ProtocolResult<Self> {
        let alice = ETPM::new(config.k, config.n, config.l, &config.activation_type)?;
        let bob = ETPM::new(config.k, config.n, config.l, &config.activation_type)?;

        Ok(Self {
            alice,
            bob,
            config: config.clone(),
        })
    }

    /// Orchestrates a standard unauthenticated key exchange synchronization.
    pub fn run(&mut self) -> ProtocolResult<KeyExchangeResult> {
        #[cfg(feature = "std")]
        let start_time = Instant::now();
        let mut rounds = 0;
        let mut rng = crate::rng::secure_rng();

        // Ensure initially randomized weights differ (at least 30% overlap safety margin)
        let mut safety_margin = 0;
        while compute_overlap(&self.alice.weights, &self.bob.weights) > 0.3 {
            self.alice.initialize_weights(None)?;
            self.bob.initialize_weights(None)?;
            safety_margin += 1;
            if safety_margin > 10 {
                break;
            }
        }

        // Salt derived from initial public inputs to guarantee unique session key
        let mut salt = Vec::new();

        while rounds < self.config.max_rounds {
            rounds += 1;

            // Generate random input vector, optionally filtered by Active Query threshold
            let inputs: Vec<Vec<i32>> = if let Some(threshold) = self.config.active_query_threshold {
                let mut candidate;
                let mut attempts = 0;
                loop {
                    candidate = (0..self.config.k)
                        .map(|_| {
                            (0..self.config.n)
                                .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                                .collect()
                        })
                        .collect();

                    let fields = self.alice.calculate_local_fields(&candidate);
                    let min_field = fields.iter().map(|f| f.abs()).min().unwrap_or(0);
                    if min_field <= threshold {
                        break;
                    }
                    attempts += 1;
                    if attempts > 100 {
                        break;
                    }
                }
                candidate
            } else {
                (0..self.config.k)
                    .map(|_| {
                        (0..self.config.n)
                            .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                            .collect()
                    })
                    .collect()
            };

            if rounds == 1 {
                // Flatten first round inputs for salt
                for row in &inputs {
                    for &val in row {
                        salt.push(if val == 1 { 1u8 } else { 0u8 });
                    }
                }
            }

            let tau_a = self.alice.calculate_output(inputs.clone())?;
            let tau_b = self.bob.calculate_output(inputs)?;

            // Mutual learning update only when outputs match
            if tau_a == tau_b {
                self.alice.update_weights(tau_a, &self.config.update_rule)?;
                self.bob.update_weights(tau_b, &self.config.update_rule)?;
            }

            // Adaptive L Scaling to recover from geometric attacks or bad locks
            if self.config.adaptive_l_scaling && rounds > 0 && rounds % 3000 == 0 {
                let new_l = self.alice.l + 2;
                self.alice.scale_synaptic_depth(new_l)?;
                self.bob.scale_synaptic_depth(new_l)?;
            }

            // Sync successful when Alice and Bob weight matrices match exactly
            if self.alice.weights == self.bob.weights {
                let final_key = self.derive_key(salt)?;
                #[cfg(feature = "std")]
                let sync_time_ms = start_time.elapsed().as_secs_f64() * 1000.0;
                #[cfg(not(feature = "std"))]
                let sync_time_ms = 0.0;
                return Ok(KeyExchangeResult {
                    success: true,
                    rounds,
                    key_hex: hex::encode(final_key),
                    sync_time_ms,
                });
            }
        }

        Ok(KeyExchangeResult {
            success: false,
            rounds: self.config.max_rounds,
            key_hex: String::new(),
            sync_time_ms: 0.0,
        })
    }

    /// Orchestrates an authenticated key exchange using a Zero-Knowledge Proof (PSK).
    pub fn authenticated_run(&mut self, psk: Vec<u8>) -> ProtocolResult<KeyExchangeResult> {
        let mut prover = ZKPProver::new(psk.clone());
        let mut verifier = ZKPVerifier::new(psk);

        // 1. Commitment Phase
        let commitment = prover.create_commitment();
        verifier.receive_commitment(commitment);

        // 2. Challenge Phase
        let challenge = verifier.create_challenge();
        let response = prover.respond(challenge);

        // 3. Verification Phase
        let authenticated = verifier.verify(
            prover.get_nonce(),
            response,
            prover.get_session_counter(),
        )?;

        if !authenticated {
            return Err(make_err!("Authentication failed: Zero-Knowledge proof mismatch"));
        }

        // Run key exchange synchronisation after successful mutual authentication
        self.run()
    }

    /// Derives the final symmetric key using HKDF-SHA256.
    fn derive_key(&self, salt: Vec<u8>) -> ProtocolResult<Vec<u8>> {
        // Prepare weights input
        let final_weights = if self.alice.activation_type == ActivationType::Hybrid {
            self.alice.chaotic_transform(self.config.chaotic_iterations)
        } else {
            self.alice.weights.clone()
        };

        let mut ikm = Vec::new();
        for row in &final_weights {
            for &w in row {
                ikm.extend_from_slice(&w.to_le_bytes());
            }
        }

        // Derive 32-byte (256-bit) symmetric key
        let hk = Hkdf::<Sha256>::new(Some(&salt), &ikm);
        let mut okm = vec![0u8; 32];
        hk.expand(b"DeepEnigma-Symmetric-Key", &mut okm)
            .map_err(|_| make_err!("HKDF expansion failed"))?;

        // Securely wipe intermediate input keying material
        ikm.zeroize();

        Ok(okm)
    }
}

// Python bindings for KeyExchange
#[cfg(feature = "extension-module")]
#[pymethods]
impl KeyExchange {
    #[new]
    pub fn py_new(config: &KeyExchangeConfig) -> ProtocolResult<Self> {
        Self::new(config)
    }

    #[pyo3(name = "run")]
    pub fn py_run(&mut self) -> ProtocolResult<KeyExchangeResult> {
        self.run()
    }

    #[pyo3(name = "authenticated_run")]
    pub fn py_authenticated_run(&mut self, psk: Vec<u8>) -> ProtocolResult<KeyExchangeResult> {
        self.authenticated_run(psk)
    }
}

/// Helper function to compute matching weight ratio between two matrices.
pub fn compute_overlap(w1: &[Vec<i32>], w2: &[Vec<i32>]) -> f64 {
    if w1.is_empty() || w2.is_empty() || w1.len() != w2.len() || w1[0].len() != w2[0].len() {
        return 0.0;
    }

    let k = w1.len();
    let n = w1[0].len();
    let total = (k * n) as f64;
    let mut matching = 0.0;

    for i in 0..k {
        for j in 0..n {
            if w1[i][j] == w2[i][j] {
                matching += 1.0;
            }
        }
    }
    matching / total
}
