//! Performance benchmarking module for E-TPM operations.
//!
//! Provides tools to measure the throughput and latency of core E-TPM
//! operations such as `calculate_output`, `update_weights`, and full
//! synchronization rounds across different parameter configurations.

use pyo3::prelude::*;
use rand::prelude::*;
use std::time::Instant;

use crate::etpm::ETPM;

/// Result of a single benchmark run.
#[pyclass]
#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    /// Name of the benchmarked operation.
    #[pyo3(get)]
    pub operation: String,
    /// Number of iterations executed.
    #[pyo3(get)]
    pub iterations: u32,
    /// Total wall-clock time in milliseconds.
    #[pyo3(get)]
    pub total_time_ms: f64,
    /// Average time per operation in microseconds.
    #[pyo3(get)]
    pub avg_time_us: f64,
    /// Throughput in operations per second.
    #[pyo3(get)]
    pub ops_per_sec: f64,
}

#[pymethods]
impl BenchmarkResult {
    fn __repr__(&self) -> String {
        format!(
            "BenchmarkResult(operation='{}', iterations={}, total_time_ms={:.2}, avg_time_us={:.2}, ops_per_sec={:.0})",
            self.operation, self.iterations, self.total_time_ms, self.avg_time_us, self.ops_per_sec
        )
    }
}

/// Generates a random K×N input matrix with values in {-1, 1}.
fn random_inputs(k: usize, n: usize) -> Vec<Vec<i32>> {
    let mut rng = crate::rng::secure_rng();
    (0..k)
        .map(|_| {
            (0..n)
                .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                .collect()
        })
        .collect()
}

/// Computes a [`BenchmarkResult`] from timing data.
fn make_result(operation: &str, iterations: u32, elapsed: std::time::Duration) -> BenchmarkResult {
    let total_ms = elapsed.as_secs_f64() * 1000.0;
    let avg_us = if iterations > 0 {
        total_ms * 1000.0 / iterations as f64
    } else {
        0.0
    };
    let ops = if total_ms > 0.0 {
        iterations as f64 / (total_ms / 1000.0)
    } else {
        0.0
    };

    BenchmarkResult {
        operation: operation.to_string(),
        iterations,
        total_time_ms: total_ms,
        avg_time_us: avg_us,
        ops_per_sec: ops,
    }
}

/// Benchmark harness for E-TPM operations.
///
/// Create an instance with the desired `K`, `N`, `L` parameters and then
/// call the individual `bench_*` methods to collect timing data.
#[pyclass]
pub struct Benchmark {
    etpm: ETPM,
    k: usize,
    n: usize,
    l: i32,
}

#[pymethods]
impl Benchmark {
    /// Creates a new benchmark harness with an E-TPM of the given dimensions.
    #[new]
    pub fn new(k: usize, n: usize, l: i32) -> PyResult<Self> {
        let etpm = ETPM::new(k, n, l, "hybrid")?;
        Ok(Self { etpm, k, n, l })
    }

    /// Benchmarks [`ETPM::calculate_output`] over `iterations` calls.
    pub fn bench_calculate_output(&mut self, iterations: u32) -> PyResult<BenchmarkResult> {
        // Pre-generate all input matrices to exclude generation cost from timing.
        let inputs: Vec<Vec<Vec<i32>>> = (0..iterations)
            .map(|_| random_inputs(self.k, self.n))
            .collect();

        let start = Instant::now();
        for input in &inputs {
            let _ = self.etpm.calculate_output(input.clone())?;
        }
        let elapsed = start.elapsed();

        Ok(make_result("calculate_output", iterations, elapsed))
    }

    /// Benchmarks [`ETPM::update_weights`] over `iterations` calls.
    ///
    /// Each iteration first computes an output (to set internal state) and
    /// then measures the weight update.
    pub fn bench_update_weights(&mut self, iterations: u32) -> PyResult<BenchmarkResult> {
        let inputs: Vec<Vec<Vec<i32>>> = (0..iterations)
            .map(|_| random_inputs(self.k, self.n))
            .collect();

        // Compute outputs first so that last_input / outputs are populated.
        let taus: Vec<i32> = inputs
            .iter()
            .map(|inp| self.etpm.calculate_output(inp.clone()))
            .collect::<PyResult<Vec<_>>>()?;

        let start = Instant::now();
        for &tau in &taus {
            self.etpm.update_weights(tau, "hebbian")?;
        }
        let elapsed = start.elapsed();

        Ok(make_result("update_weights", iterations, elapsed))
    }

    /// Runs `trials` independent full-synchronization attempts between two
    /// freshly-created E-TPMs, returning per-trial statistics.
    ///
    /// Each trial runs until the two machines' weights match or a cap of
    /// 10 000 rounds is reached.
    pub fn bench_full_sync(&mut self, trials: u32) -> PyResult<Vec<BenchmarkResult>> {
        const MAX_ROUNDS: u32 = 10_000;
        let mut results = Vec::with_capacity(trials as usize);

        for trial in 0..trials {
            let mut a = ETPM::new(self.k, self.n, self.l, "hybrid")?;
            let mut b = ETPM::new(self.k, self.n, self.l, "hybrid")?;

            let start = Instant::now();
            let mut rounds: u32 = 0;

            loop {
                let inputs = random_inputs(self.k, self.n);
                let tau_a = a.calculate_output(inputs.clone())?;
                let tau_b = b.calculate_output(inputs.clone())?;

                if tau_a == tau_b {
                    a.update_weights(tau_a, "hebbian")?;
                    b.update_weights(tau_b, "hebbian")?;
                }

                rounds += 1;

                if a.get_weights() == b.get_weights() || rounds >= MAX_ROUNDS {
                    break;
                }
            }
            let elapsed = start.elapsed();

            results.push(make_result(
                &format!("full_sync_trial_{}", trial + 1),
                rounds,
                elapsed,
            ));
        }

        Ok(results)
    }

    /// Benchmarks a single synchronization run for each `(K, N, L)` tuple
    /// in `params`, allowing easy comparison of different parameter sizes.
    pub fn compare_parameters(
        &self,
        params: Vec<(usize, usize, i32)>,
    ) -> PyResult<Vec<BenchmarkResult>> {
        const MAX_ROUNDS: u32 = 10_000;
        let mut results = Vec::with_capacity(params.len());

        for (k, n, l) in &params {
            let mut a = ETPM::new(*k, *n, *l, "hybrid")?;
            let mut b = ETPM::new(*k, *n, *l, "hybrid")?;

            let start = Instant::now();
            let mut rounds: u32 = 0;

            loop {
                let inputs = random_inputs(*k, *n);
                let tau_a = a.calculate_output(inputs.clone())?;
                let tau_b = b.calculate_output(inputs.clone())?;

                if tau_a == tau_b {
                    a.update_weights(tau_a, "hebbian")?;
                    b.update_weights(tau_b, "hebbian")?;
                }

                rounds += 1;

                if a.get_weights() == b.get_weights() || rounds >= MAX_ROUNDS {
                    break;
                }
            }
            let elapsed = start.elapsed();

            results.push(make_result(
                &format!("sync_K{}_N{}_L{}", k, n, l),
                rounds,
                elapsed,
            ));
        }

        Ok(results)
    }
}
