//! Zero-Knowledge Proof (ZKP) Mutual Authentication module.
//!
//! Provides a lightweight, hash-based proof of knowledge of a pre-shared secret (PSK)
//! without revealing the secret itself, protecting against Man-in-the-Middle (MitM)
//! and replay attacks during the E-TPM key exchange setup.
//!
//! # Replay Attack Defense
//! Each commitment binds a monotonic session counter into the hash, ensuring that
//! captured authentication transcripts cannot be replayed in future sessions.

use pyo3::prelude::*;
use rand::Rng;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use crate::constant_time::ct_eq;

/// Computes SHA-256 hash of the given data.
fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Prover for the Hash-based Zero-Knowledge Proof of Knowledge.
///
/// Holds the pre-shared key (PSK), generates commitments, and responds
/// to verification challenges without revealing the PSK.
#[pyclass]
#[derive(Clone)]
pub struct ZKPProver {
    psk: Vec<u8>,
    nonce: Vec<u8>,
    session_counter: u64,
}

/// Securely wipe PSK and nonce from memory on drop.
impl Drop for ZKPProver {
    fn drop(&mut self) {
        self.psk.zeroize();
        self.nonce.zeroize();
    }
}

#[pymethods]
impl ZKPProver {
    /// Creates a new Prover instance initialized with the pre-shared secret.
    #[new]
    pub fn new(psk: Vec<u8>) -> Self {
        Self {
            psk,
            nonce: vec![0; 32],
            session_counter: 0,
        }
    }

    /// Generates a random 32-byte nonce, stores it, and returns its SHA-256 commitment.
    ///
    /// The commitment includes a monotonically-increasing session counter to
    /// prevent replay attacks: `C = SHA256(nonce || counter_bytes)`.
    pub fn create_commitment(&mut self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut r = vec![0u8; 32];
        rng.fill(&mut r[..]);
        self.nonce = r;
        self.session_counter += 1;

        // Bind session counter into commitment to prevent replay
        let mut commit_data = self.nonce.clone();
        commit_data.extend_from_slice(&self.session_counter.to_le_bytes());
        sha256(&commit_data)
    }

    /// Computes the response to a verification challenge:
    /// `z = SHA256(PSK || nonce || challenge || counter)`.
    pub fn respond(&self, challenge: Vec<u8>) -> Vec<u8> {
        let mut data = Vec::with_capacity(
            self.psk.len() + self.nonce.len() + challenge.len() + 8,
        );
        data.extend_from_slice(&self.psk);
        data.extend_from_slice(&self.nonce);
        data.extend_from_slice(&challenge);
        data.extend_from_slice(&self.session_counter.to_le_bytes());
        sha256(&data)
    }

    /// Returns the raw nonce generated during `create_commitment`.
    pub fn get_nonce(&self) -> Vec<u8> {
        self.nonce.clone()
    }

    /// Returns the current session counter value.
    pub fn get_session_counter(&self) -> u64 {
        self.session_counter
    }
}

/// Verifier for the Hash-based Zero-Knowledge Proof of Knowledge.
///
/// Receives a prover's commitment, issues a random challenge, and verifies
/// the response using constant-time comparisons.
#[pyclass]
#[derive(Clone)]
pub struct ZKPVerifier {
    psk: Vec<u8>,
    commitment: Vec<u8>,
    challenge: Vec<u8>,
    last_seen_counter: u64,
}

/// Securely wipe PSK from memory on drop.
impl Drop for ZKPVerifier {
    fn drop(&mut self) {
        self.psk.zeroize();
        self.commitment.zeroize();
        self.challenge.zeroize();
    }
}

#[pymethods]
impl ZKPVerifier {
    /// Creates a new Verifier instance initialized with the pre-shared secret.
    #[new]
    pub fn new(psk: Vec<u8>) -> Self {
        Self {
            psk,
            commitment: vec![],
            challenge: vec![],
            last_seen_counter: 0,
        }
    }

    /// Receives and registers the prover's commitment.
    pub fn receive_commitment(&mut self, commitment: Vec<u8>) {
        self.commitment = commitment;
    }

    /// Generates and returns a random 32-byte verification challenge.
    pub fn create_challenge(&mut self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut c = vec![0u8; 32];
        rng.fill(&mut c[..]);
        self.challenge = c;
        self.challenge.clone()
    }

    /// Verifies the prover's response using **constant-time** comparisons.
    ///
    /// Checks that:
    /// 1. The session counter is strictly greater than the last seen counter (replay defense)
    /// 2. `SHA256(nonce || counter) == commitment` (commitment integrity)
    /// 3. `SHA256(PSK || nonce || challenge || counter) == response` (secret knowledge)
    ///
    /// All byte comparisons use `ct_eq` to prevent timing side-channel attacks.
    pub fn verify(&mut self, nonce: Vec<u8>, response: Vec<u8>, session_counter: u64) -> bool {
        if self.commitment.is_empty() || self.challenge.is_empty() {
            return false;
        }

        // Replay defense: reject if counter is not strictly increasing
        if session_counter <= self.last_seen_counter {
            return false;
        }

        // Verify commitment match (constant-time)
        let mut commit_data = nonce.clone();
        commit_data.extend_from_slice(&session_counter.to_le_bytes());
        let expected_commitment = sha256(&commit_data);
        if !ct_eq(&expected_commitment, &self.commitment) {
            return false;
        }

        // Verify secret knowledge match (constant-time)
        let mut data = Vec::with_capacity(self.psk.len() + nonce.len() + self.challenge.len() + 8);
        data.extend_from_slice(&self.psk);
        data.extend_from_slice(&nonce);
        data.extend_from_slice(&self.challenge);
        data.extend_from_slice(&session_counter.to_le_bytes());

        let expected_response = sha256(&data);
        let valid = ct_eq(&expected_response, &response);

        if valid {
            self.last_seen_counter = session_counter;
        }

        valid
    }
}
