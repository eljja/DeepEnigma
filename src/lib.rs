#![cfg_attr(not(feature = "std"), no_std)]

pub mod auth;
#[cfg(feature = "std")]
pub mod benchmark;
pub mod constant_time;
pub mod etpm;
pub mod handshake;
pub mod protocol;
pub mod rng;
pub mod security;
pub mod neural;
#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
pub mod wasm;

pub use auth::{ZKPProver, ZKPVerifier};
#[cfg(feature = "std")]
pub use benchmark::{Benchmark, BenchmarkResult};
pub use constant_time::{ct_eq, ct_mask_i32, ct_select_i32};
pub use etpm::{ActivationType, UpdateRule, ETPM};
pub use handshake::{HandshakeMessage, ParameterNegotiator};
pub use protocol::{KeyExchange, KeyExchangeConfig, KeyExchangeResult};
pub use rng::{secure_rng, SecureRng};
pub use security::{AttackResult, SecurityAnalyzer};
pub use neural::{DenseLayer, NeuralNet, Activation, hamming_encode, hamming_decode};

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;

/// Python module entry point for the `deep_enigma` package.
#[cfg(feature = "extension-module")]
#[pymodule]
fn deep_enigma(m: &Bound<'_, PyModule>) -> PyResult<()> {
    // Core E-TPM types
    m.add_class::<ETPM>()?;
    m.add_class::<ActivationType>()?;
    m.add_class::<UpdateRule>()?;
    // Key exchange protocol
    m.add_class::<KeyExchangeConfig>()?;
    m.add_class::<KeyExchangeResult>()?;
    m.add_class::<KeyExchange>()?;
    // Handshake
    m.add_class::<HandshakeMessage>()?;
    m.add_class::<ParameterNegotiator>()?;
    // Security analysis
    m.add_class::<AttackResult>()?;
    m.add_class::<SecurityAnalyzer>()?;
    // Benchmarking
    m.add_class::<BenchmarkResult>()?;
    m.add_class::<Benchmark>()?;
    // ZKP Authentication
    m.add_class::<ZKPProver>()?;
    m.add_class::<ZKPVerifier>()?;
    // Neural Cryptography
    m.add_class::<neural::PyDenseLayer>()?;
    m.add_class::<neural::PyNeuralNet>()?;
    Ok(())
}
