pub mod benchmark;
pub mod etpm;
pub mod protocol;
pub mod security;

pub use benchmark::{Benchmark, BenchmarkResult};
pub use etpm::{ActivationType, UpdateRule, ETPM};
pub use protocol::{KeyExchange, KeyExchangeConfig, KeyExchangeResult};
pub use security::{AttackResult, SecurityAnalyzer};

use pyo3::prelude::*;

/// DeepEnigma: Neural Network-based Cryptographic Key Exchange Module.
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
    Ok(())
}
