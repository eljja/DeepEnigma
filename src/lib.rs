pub mod auth;
pub mod benchmark;
pub mod constant_time;
pub mod etpm;
pub mod protocol;
pub mod security;

pub use auth::{ZKPProver, ZKPVerifier};
pub use benchmark::{Benchmark, BenchmarkResult};
pub use constant_time::{ct_eq, ct_mask_i32, ct_select_i32};
pub use etpm::{ActivationType, UpdateRule, ETPM};
pub use protocol::{KeyExchange, KeyExchangeConfig, KeyExchangeResult};
pub use security::{AttackResult, SecurityAnalyzer};

use pyo3::prelude::*;

/// Python module entry point for the `deep_enigma` package.
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
    // Security analysis
    m.add_class::<AttackResult>()?;
    m.add_class::<SecurityAnalyzer>()?;
    // Benchmarking
    m.add_class::<BenchmarkResult>()?;
    m.add_class::<Benchmark>()?;
    // ZKP Authentication
    m.add_class::<ZKPProver>()?;
    m.add_class::<ZKPVerifier>()?;
    Ok(())
}
