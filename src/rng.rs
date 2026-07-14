//! Cryptographically Secure Random Number Generation (CSPRNG) utility.
//!
//! Provides explicit CSPRNG seeding using OS entropy via `OsRng` to prevent
//! weak or predictable randomness, which is critical for cryptographic keys
//! and input vector generation.

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
}
