//! Zero-Knowledge Proof (ZKP) Mutual Authentication module.
//!
//! Provides a lightweight, hash-based proof of knowledge of a pre-shared secret (PSK)
//! without revealing the secret itself, protecting against Man-in-the-Middle (MitM)
//! and replay attacks during the E-TPM key exchange setup.

use pyo3::prelude::*;
use rand::Rng;
use sha2::{Digest, Sha256};

/// Computes SHA-256 hash of the given data.
fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Prover for the Hash-based Zero-Knowledge Proof of Knowledge.
#[pyclass]
#[derive(Clone)]
pub struct ZKPProver {
    psk: Vec<u8>,
    nonce: Vec<u8>,
}

#[pymethods]
impl ZKPProver {
    /// Creates a new Prover instance initialized with the pre-shared secret.
    #[new]
    pub fn new(psk: Vec<u8>) -> Self {
        Self {
            psk,
            nonce: vec![0; 32],
        }
    }

    /// Generates a random 32-byte nonce, stores it, and returns its SHA-256 commitment.
    pub fn create_commitment(&mut self) -> Vec<u8> {
        let mut rng = rand::thread_rng();
        let mut r = vec![0u8; 32];
        rng.fill(&mut r[..]);
        self.nonce = r;
        sha256(&self.nonce)
    }

    /// Computes the response to a verification challenge: Hash(PSK || nonce || challenge).
    pub fn respond(&self, challenge: Vec<u8>) -> Vec<u8> {
        let mut data = Vec::with_capacity(self.psk.len() + self.nonce.len() + challenge.len());
        data.extend_from_slice(&self.psk);
        data.extend_from_slice(&self.nonce);
        data.extend_from_slice(&challenge);
        sha256(&data)
    }

    /// Returns the raw nonce generated during `create_commitment`.
    pub fn get_nonce(&self) -> Vec<u8> {
        self.nonce.clone()
    }
}

/// Verifier for the Hash-based Zero-Knowledge Proof of Knowledge.
#[pyclass]
#[derive(Clone)]
pub struct ZKPVerifier {
    psk: Vec<u8>,
    commitment: Vec<u8>,
    challenge: Vec<u8>,
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

    /// Verifies the prover's response.
    ///
    /// Checks that:
    /// 1. Hash(nonce) == commitment (proof of commitment matching)
    /// 2. Hash(PSK || nonce || challenge) == response (proof of secret knowledge)
    pub fn verify(&self, nonce: Vec<u8>, response: Vec<u8>) -> bool {
        if self.commitment.is_empty() || self.challenge.is_empty() {
            return false;
        }

        // Verify commitment match
        let expected_commitment = sha256(&nonce);
        if expected_commitment != self.commitment {
            return false;
        }

        // Verify secret knowledge match
        let mut data = Vec::with_capacity(self.psk.len() + nonce.len() + self.challenge.len());
        data.extend_from_slice(&self.psk);
        data.extend_from_slice(&nonce);
        data.extend_from_slice(&self.challenge);

        let expected_response = sha256(&data);
        expected_response == response
    }
}
