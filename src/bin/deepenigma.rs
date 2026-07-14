//! DeepEnigma CLI — demonstrates E-TPM key exchange and benchmarking.
//!
//! Usage:
//!   deepenigma [OPTIONS]
//!
//! Options:
//!   --k <value>          Number of hidden units (default: 4)
//!   --n <value>          Synaptic input size per unit (default: 128)
//!   --l <value>          Synaptic depth (default: 8)
//!   --max-rounds <value> Maximum synchronization rounds (default: 10000)
//!   --benchmark          Run performance benchmarks instead of key exchange

use deep_enigma::benchmark::Benchmark;
use deep_enigma::etpm::ETPM;
use rand::prelude::*;
use sha2::Sha256;
use hkdf::Hkdf;
use std::time::Instant;
use zeroize::Zeroize;

// ── CLI argument parsing ────────────────────────────────────────────────────

struct Args {
    k: usize,
    n: usize,
    l: i32,
    max_rounds: u32,
    benchmark: bool,
    update_rule: String,
    activation_type: String,
}

fn parse_args() -> Result<Args, String> {
    let args: Vec<String> = std::env::args().collect();
    let mut cfg = Args {
        k: 4,
        n: 128,
        l: 8,
        max_rounds: 10_000,
        benchmark: false,
        update_rule: "hebbian".to_string(),
        activation_type: "hybrid".to_string(),
    };

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--k" => {
                i += 1;
                cfg.k = args.get(i).ok_or("--k requires a value")?
                    .parse().map_err(|e| format!("invalid value for --k: {}", e))?;
            }
            "--n" => {
                i += 1;
                cfg.n = args.get(i).ok_or("--n requires a value")?
                    .parse().map_err(|e| format!("invalid value for --n: {}", e))?;
            }
            "--l" => {
                i += 1;
                cfg.l = args.get(i).ok_or("--l requires a value")?
                    .parse().map_err(|e| format!("invalid value for --l: {}", e))?;
            }
            "--max-rounds" => {
                i += 1;
                cfg.max_rounds = args.get(i).ok_or("--max-rounds requires a value")?
                    .parse().map_err(|e| format!("invalid value for --max-rounds: {}", e))?;
            }
            "--rule" => {
                i += 1;
                cfg.update_rule = args.get(i).ok_or("--rule requires a value")?.clone();
            }
            "--activation" => {
                i += 1;
                cfg.activation_type = args.get(i).ok_or("--activation requires a value")?.clone();
            }
            "--benchmark" => {
                cfg.benchmark = true;
            }
            other => {
                return Err(format!("Unknown argument: {}", other));
            }
        }
        i += 1;
    }
    Ok(cfg)
}

// ── Helpers ─────────────────────────────────────────────────────────────────

/// Generates a random K×N input matrix with values in {-1, 1}.
fn random_inputs(k: usize, n: usize) -> Vec<Vec<i32>> {
    let mut rng = deep_enigma::secure_rng();
    (0..k)
        .map(|_| {
            (0..n)
                .map(|_| if rng.gen_bool(0.5) { 1 } else { -1 })
                .collect()
        })
        .collect()
}

/// Derives a 256-bit hex key from a weight matrix using HKDF-SHA256.
fn derive_key(weights: &[Vec<i32>]) -> Result<String, String> {
    let mut ikm: Vec<u8> = Vec::with_capacity(weights.len() * weights[0].len() * 4);
    for row in weights {
        for &w in row {
            ikm.extend_from_slice(&w.to_le_bytes());
        }
    }
    let hk = Hkdf::<Sha256>::new(None, &ikm);
    let info = b"DeepEnigma-Symmetric-Key";
    let mut okm = vec![0u8; 32];
    hk.expand(info, &mut okm).map_err(|e| format!("HKDF expand failed: {}", e))?;
    ikm.zeroize();
    Ok(hex::encode(okm))
}

fn print_banner() {
    println!("╔══════════════════════════════════════════════════════╗");
    println!("║          DeepEnigma — E-TPM Key Exchange            ║");
    println!("╚══════════════════════════════════════════════════════╝");
}

// ── Key exchange demo ───────────────────────────────────────────────────────

fn run_key_exchange(args: &Args) -> Result<(), String> {
    print_banner();
    println!();
    println!("Parameters: K={}, N={}, L={}, max_rounds={}", args.k, args.n, args.l, args.max_rounds);
    println!("{}", "─".repeat(54));

    let mut alice = ETPM::new(args.k, args.n, args.l, &args.activation_type)
        .map_err(|e| format!("Failed to create Alice's E-TPM: {}", e))?;
    let mut bob = ETPM::new(args.k, args.n, args.l, &args.activation_type)
        .map_err(|e| format!("Failed to create Bob's E-TPM: {}", e))?;

    let start = Instant::now();
    let mut synced = false;

    for round in 1..=args.max_rounds {
        let inputs = random_inputs(args.k, args.n);
        let tau_a = alice.calculate_output(inputs.clone())
            .map_err(|e| format!("Alice calculate_output failed at round {}: {}", round, e))?;
        let tau_b = bob.calculate_output(inputs.clone())
            .map_err(|e| format!("Bob calculate_output failed at round {}: {}", round, e))?;

        if tau_a == tau_b {
            alice.update_weights(tau_a, &args.update_rule)
                .map_err(|e| format!("Alice update_weights failed: {}", e))?;
            bob.update_weights(tau_b, &args.update_rule)
                .map_err(|e| format!("Bob update_weights failed: {}", e))?;
        }

        if round % 500 == 0 {
            let matching: usize = alice
                .get_weights()
                .iter()
                .zip(bob.get_weights().iter())
                .flat_map(|(ra, rb)| ra.iter().zip(rb.iter()))
                .filter(|(a, b)| a == b)
                .count();
            let total = args.k * args.n;
            let pct = 100.0 * matching as f64 / total as f64;
            println!("  Round {:>6}: weight agreement {:.1}% ({}/{})", round, pct, matching, total);
        }

        if alice.get_weights() == bob.get_weights() {
            let elapsed = start.elapsed();
            println!();
            println!("✓ Synchronized after {} rounds ({:.2} ms)", round, elapsed.as_secs_f64() * 1000.0);

            let final_weights = if args.activation_type.to_lowercase() == "hybrid" {
                alice.chaotic_transform(100)
            } else {
                alice.get_weights()
            };
            let key = derive_key(&final_weights)?;
            println!();
            println!("Derived 256-bit key (hex):");
            println!("  {}", key);
            println!();
            println!("Timing: {:.3} s  ({:.1} rounds/s)",
                elapsed.as_secs_f64(),
                round as f64 / elapsed.as_secs_f64(),
            );
            synced = true;
            break;
        }
    }

    if !synced {
        let elapsed = start.elapsed();
        println!();
        println!("✗ Failed to synchronize within {} rounds ({:.2} s)", args.max_rounds, elapsed.as_secs_f64());
    }
    Ok(())
}

// ── Benchmark mode ──────────────────────────────────────────────────────────

fn run_benchmarks(args: &Args) -> Result<(), String> {
    print_banner();
    println!();
    println!("Benchmark mode — K={}, N={}, L={}", args.k, args.n, args.l);
    println!("{}", "─".repeat(54));

    let mut bench = Benchmark::new(args.k, args.n, args.l)
        .map_err(|e| format!("Failed to create benchmark harness: {}", e))?;

    // calculate_output benchmark
    println!("\n[1/3] Benchmarking calculate_output (10000 iterations)...");
    let res = bench.bench_calculate_output(10_000)
        .map_err(|e| format!("calculate_output benchmark failed: {}", e))?;
    print_result(&res);

    // update_weights benchmark
    println!("\n[2/3] Benchmarking update_weights (10000 iterations)...");
    let res = bench.bench_update_weights(10_000)
        .map_err(|e| format!("update_weights benchmark failed: {}", e))?;
    print_result(&res);

    // full sync trials
    println!("\n[3/3] Running 5 full synchronization trials...");
    let results = bench.bench_full_sync(5)
        .map_err(|e| format!("full_sync benchmark failed: {}", e))?;

    println!();
    println!("{:<22} {:>10} {:>12} {:>12} {:>14}",
        "Operation", "Iters", "Total (ms)", "Avg (µs)", "Ops/s");
    println!("{}", "─".repeat(72));
    for r in &results {
        println!("{:<22} {:>10} {:>12.2} {:>12.2} {:>14.0}",
            r.operation, r.iterations, r.total_time_ms, r.avg_time_us, r.ops_per_sec);
    }
    println!();
    Ok(())
}

fn print_result(r: &deep_enigma::benchmark::BenchmarkResult) {
    println!("  {} — {} iters in {:.2} ms ({:.2} µs/op, {:.0} ops/s)",
        r.operation, r.iterations, r.total_time_ms, r.avg_time_us, r.ops_per_sec);
}

// ── Entry point ─────────────────────────────────────────────────────────────

fn main() {
    let args = match parse_args() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    };

    let result = if args.benchmark {
        run_benchmarks(&args)
    } else {
        run_key_exchange(&args)
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }
}
