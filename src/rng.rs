//! Cryptographically Secure Random Number Generation (CSPRNG) utility.
//!
//! Provides explicit CSPRNG seeding using OS entropy via `OsRng` to prevent
//! weak or predictable randomness, which is critical for cryptographic keys
//! and input vector generation.

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use rand::rngs::OsRng;
use rand::{CryptoRng, RngCore, SeedableRng};
use rand_chacha::ChaCha20Rng;

/// Returns an initialized cryptographically secure pseudorandom number generator (CSPRNG).
///
/// Under the hood, this uses **ChaCha20Rng** seeded directly from the operating system's
/// secure entropy source (`OsRng`). If the OS entropy source fails or is unavailable,
/// this function will panic rather than silently falling back to a weak generator.
///
/// # Cryptographic properties
/// - **Seeding**: Directly from OS-provided hardware/cryptographic entropy.
/// - **Algorithmic Security**: ChaCha20 has no known practical cryptanalytic attacks.
/// - **Backtracking Resistance**: Even if the generator's state is compromised later,
///   an attacker cannot determine past random outputs.
pub fn secure_rng() -> ChaCha20Rng {
    // Seed the ChaCha20 CSPRNG directly from the OS-supplied entropy pool.
    // If OsRng fails (e.g., out of system descriptors, hardware failure), it will panic,
    // ensuring we never generate cryptographic parameters using predictable values.
    ChaCha20Rng::from_rng(OsRng).expect("Fatal: Failed to seed CSPRNG from OS entropy source.")
}

/// A wrapper struct that implements `RngCore` and `CryptoRng` using `secure_rng()`.
#[allow(dead_code)]
pub struct SecureRng {
    inner: ChaCha20Rng,
}

impl SecureRng {
    pub fn new() -> Self {
        Self {
            inner: secure_rng(),
        }
    }
}

impl RngCore for SecureRng {
    fn next_u32(&mut self) -> u32 {
        self.inner.next_u32()
    }

    fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    fn fill_bytes(&mut self, dest: &mut [u8]) {
        self.inner.fill_bytes(dest);
    }

    fn try_fill_bytes(&mut self, dest: &mut [u8]) -> Result<(), rand::Error> {
        self.inner.try_fill_bytes(dest)
    }
}

impl CryptoRng for SecureRng {}

/// Simulates spatially correlated wireless channel noise/observations for Alice and Bob.
///
/// This simulates a physical layer channel key agreement input source.
/// - `correlation` specifies the correlation coefficient R in [0.0, 1.0].
/// - When `correlation` is 1.0, both vectors are identical.
/// - When `correlation` is 0.0, they are completely independent.
pub fn generate_correlated_noise(
    seed_alice: u64,
    seed_bob: u64,
    correlation: f64,
    len: usize,
) -> (Vec<i32>, Vec<i32>) {
    use rand::{Rng, SeedableRng};
    
    // Alice's local generator
    let mut rng_a = ChaCha20Rng::seed_from_u64(seed_alice);
    // Bob's local generator
    let mut rng_b = ChaCha20Rng::seed_from_u64(seed_bob);
    // Shared environment generator (representing the physical channel correlation)
    let mut rng_shared = ChaCha20Rng::seed_from_u64(seed_alice ^ seed_bob);

    let mut out_a = Vec::with_capacity(len);
    let mut out_b = Vec::with_capacity(len);

    let alpha = correlation.clamp(0.0, 1.0);

    for _ in 0..len {
        let s = rng_shared.gen_range(-1.0..=1.0);
        let n_a = rng_a.gen_range(-1.0..=1.0);
        let n_b = rng_b.gen_range(-1.0..=1.0);

        let val_a = alpha * s + (1.0 - alpha) * n_a;
        let val_b = alpha * s + (1.0 - alpha) * n_b;

        out_a.push(if val_a >= 0.0 { 1 } else { -1 });
        out_b.push(if val_b >= 0.0 { 1 } else { -1 });
    }

    (out_a, out_b)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::Rng;

    #[test]
    fn test_secure_rng_generation() {
        let mut rng = SecureRng::new();
        let val1 = rng.gen::<u64>();
        let val2 = rng.gen::<u64>();
        // Very high probability they are different
        assert_ne!(val1, val2);
    }

    #[test]
    fn test_secure_rng_fill() {
        let mut rng = SecureRng::new();
        let mut buf = [0u8; 100];
        rng.fill_bytes(&mut buf);
        // Verify buffer is no longer all zeros
        assert!(buf.iter().any(|&b| b != 0));
    }

    #[test]
    fn test_correlated_noise() {
        // High correlation: should be identical
        let (a_high, b_high) = generate_correlated_noise(12345, 67890, 1.0, 100);
        assert_eq!(a_high, b_high, "1.0 correlation must yield identical observations");

        // Moderate correlation: should have some matching and some non-matching
        let (a_mod, b_mod) = generate_correlated_noise(12345, 67890, 0.5, 100);
        let matches = a_mod.iter().zip(b_mod.iter()).filter(|(&x, &y)| x == y).count();
        assert!(matches > 40 && matches < 100, "0.5 correlation should have moderate matches: {}", matches);

        // Low correlation: should be mostly independent
        let (a_low, b_low) = generate_correlated_noise(12345, 67890, 0.0, 1000);
        let low_matches = a_low.iter().zip(b_low.iter()).filter(|(&x, &y)| x == y).count();
        // Expect close to 50% matching due to binary distribution, but not 100%
        assert!(low_matches > 400 && low_matches < 600, "0.0 correlation should be random coin tosses: {}", low_matches);
    }
}
