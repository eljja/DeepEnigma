//! Performance benchmarking module for E-TPM operations.
//!
//! Provides tools to measure the throughput and latency of E-TPM operations.

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;
use std::time::Instant;
use rand::Rng;

use crate::etpm::ETPM;

/// Result type alias supporting both PyO3 and pure Rust environments.
#[cfg(feature = "extension-module")]
type BenchmarkResultType<T> = PyResult<T>;

#[cfg(not(feature = "extension-module"))]
type BenchmarkResultType<T> = Result<T, &'static str>;

/// Result of a single benchmark run.
#[cfg_attr(feature = "extension-module", pyclass)]
#[derive(Clone, Debug)]
pub struct BenchmarkResult {
    /// Name of the benchmarked operation.
    pub operation: String,
    /// Number of iterations executed.
    pub iterations: u32,
    /// Total wall-clock time in milliseconds.
    pub total_time_ms: f64,
    /// Average time per operation in microseconds.
    pub avg_time_us: f64,
    /// Throughput in operations per second.
    pub ops_per_sec: f64,
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl BenchmarkResult {
    fn __repr__(&self) -> String {
        format!(
            "BenchmarkResult(operation='{}', iterations={}, total_time_ms={:.2}, avg_time_us={:.2}, ops_per_sec={:.0})",
            self.operation, self.iterations, self.total_time_ms, self.avg_time_us, self.ops_per_sec
        )
    }

    #[getter]
    pub fn operation(&self) -> String {
        self.operation.clone()
    }

    #[getter]
    pub fn iterations(&self) -> u32 {
        self.iterations
    }

    #[getter]
    pub fn total_time_ms(&self) -> f64 {
        self.total_time_ms
    }

    #[getter]
    pub fn avg_time_us(&self) -> f64 {
        self.avg_time_us
    }

    #[getter]
    pub fn ops_per_sec(&self) -> f64 {
        self.ops_per_sec
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
#[cfg_attr(feature = "extension-module", pyclass)]
pub struct Benchmark {
    etpm: ETPM,
    k: usize,
    n: usize,
    l: i32,
}

impl Benchmark {
    /// Creates a new benchmark harness with an E-TPM of the given dimensions.
    pub fn new(k: usize, n: usize, l: i32) -> BenchmarkResultType<Self> {
        let etpm = ETPM::new(k, n, l, "hybrid")?;
        Ok(Self { etpm, k, n, l })
    }

    /// Benchmarks [`ETPM::calculate_output`] over `iterations` calls.
    pub fn bench_calculate_output(&mut self, iterations: u32) -> BenchmarkResultType<BenchmarkResult> {
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
    pub fn bench_update_weights(&mut self, iterations: u32) -> BenchmarkResultType<BenchmarkResult> {
        let inputs: Vec<Vec<Vec<i32>>> = (0..iterations)
            .map(|_| random_inputs(self.k, self.n))
            .collect();

        let taus: Vec<i32> = inputs
            .iter()
            .map(|inp| self.etpm.calculate_output(inp.clone()))
            .collect::<Result<Vec<_>, _>>()?;

        let start = Instant::now();
        for &tau in &taus {
            self.etpm.update_weights(tau, "hebbian")?;
        }
        let elapsed = start.elapsed();

        Ok(make_result("update_weights", iterations, elapsed))
    }

    /// Runs `trials` independent full-synchronization attempts between two
    /// freshly-created E-TPMs, returning per-trial statistics.
    pub fn bench_full_sync(&mut self, trials: u32) -> BenchmarkResultType<Vec<BenchmarkResult>> {
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
    ) -> BenchmarkResultType<Vec<BenchmarkResult>> {
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

// Python bindings for Benchmark
#[cfg(feature = "extension-module")]
#[pymethods]
impl Benchmark {
    #[new]
    pub fn py_new(k: usize, n: usize, l: i32) -> BenchmarkResultType<Self> {
        Self::new(k, n, l)
    }

    #[pyo3(name = "bench_calculate_output")]
    pub fn py_bench_calculate_output(&mut self, iterations: u32) -> BenchmarkResultType<BenchmarkResult> {
        self.bench_calculate_output(iterations)
    }

    #[pyo3(name = "bench_update_weights")]
    pub fn py_bench_update_weights(&mut self, iterations: u32) -> BenchmarkResultType<BenchmarkResult> {
        self.bench_update_weights(iterations)
    }

    #[pyo3(name = "bench_full_sync")]
    pub fn py_bench_full_sync(&mut self, trials: u32) -> BenchmarkResultType<Vec<BenchmarkResult>> {
        self.bench_full_sync(trials)
    }

    #[pyo3(name = "compare_parameters")]
    pub fn py_compare_parameters(
        &self,
        params: Vec<(usize, usize, i32)>,
    ) -> BenchmarkResultType<Vec<BenchmarkResult>> {
        self.compare_parameters(params)
    }
}
