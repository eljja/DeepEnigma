//! Zero-Knowledge Proof Authentication module.
//!
//! Provides Fiat-Shamir-like Hash-based ZKP authentication for E-TPM key exchange.
//! Mitigates replay attacks by binding commitments to monotonically increasing session counters.

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;
use rand::Rng;
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

use crate::constant_time::ct_eq;

/// Result type alias supporting both PyO3 and pure Rust environments.
#[cfg(feature = "extension-module")]
type AuthResult<T> = PyResult<T>;

#[cfg(not(feature = "extension-module"))]
type AuthResult<T> = Result<T, &'static str>;

#[cfg(feature = "extension-module")]
macro_rules! make_err {
    ($msg:expr) => {
        pyo3::exceptions::PyValueError::new_err($msg)
    };
}

#[cfg(not(feature = "extension-module"))]
macro_rules! make_err {
    ($msg:expr) => {
        $msg
    };
}

/// Helper function to compute SHA-256 hash of a byte slice.
fn sha256(data: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

/// Prover for the Hash-based Zero-Knowledge Proof of Knowledge.
#[cfg_attr(feature = "extension-module", pyclass)]
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

impl ZKPProver {
    /// Minimum PSK length is 16 bytes for adequate security.
    pub fn new(psk: Vec<u8>) -> AuthResult<Self> {
        if psk.len() < 16 {
            return Err(make_err!("PSK must be at least 16 bytes"));
        }
        Ok(Self {
            psk,
            nonce: vec![0; 32],
            session_counter: 0,
        })
    }

    pub fn create_commitment(&mut self) -> Vec<u8> {
        let mut rng = crate::rng::secure_rng();
        let mut r = vec![0u8; 32];
        rng.fill(&mut r[..]);
        self.nonce = r;
        self.session_counter += 1;

        // Bind session counter into commitment to prevent replay
        let mut commit_data = self.nonce.clone();
        commit_data.extend_from_slice(&self.session_counter.to_le_bytes());
        sha256(&commit_data)
    }

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

    pub fn get_nonce(&self) -> Vec<u8> {
        self.nonce.clone()
    }

    pub fn get_session_counter(&self) -> u64 {
        self.session_counter
    }
}

// Python bindings for ZKPProver
#[cfg(feature = "extension-module")]
#[pymethods]
impl ZKPProver {
    #[new]
    pub fn py_new(psk: Vec<u8>) -> AuthResult<Self> {
        Self::new(psk)
    }

    #[pyo3(name = "create_commitment")]
    pub fn py_create_commitment(&mut self) -> Vec<u8> {
        self.create_commitment()
    }

    #[pyo3(name = "respond")]
    pub fn py_respond(&self, challenge: Vec<u8>) -> Vec<u8> {
        self.respond(challenge)
    }

    #[pyo3(name = "get_nonce")]
    pub fn py_get_nonce(&self) -> Vec<u8> {
        self.get_nonce()
    }

    #[pyo3(name = "get_session_counter")]
    pub fn py_get_session_counter(&self) -> u64 {
        self.get_session_counter()
    }
}


/// Verifier for the Hash-based Zero-Knowledge Proof of Knowledge.
#[cfg_attr(feature = "extension-module", pyclass)]
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

impl ZKPVerifier {
    /// Minimum PSK length is 16 bytes for adequate security.
    pub fn new(psk: Vec<u8>) -> AuthResult<Self> {
        if psk.len() < 16 {
            return Err(make_err!("PSK must be at least 16 bytes"));
        }
        Ok(Self {
            psk,
            commitment: vec![],
            challenge: vec![],
            last_seen_counter: 0,
        })
    }

    pub fn receive_commitment(&mut self, commitment: Vec<u8>) {
        self.commitment = commitment;
    }

    pub fn create_challenge(&mut self) -> Vec<u8> {
        let mut rng = crate::rng::secure_rng();
        let mut c = vec![0u8; 32];
        rng.fill(&mut c[..]);
        self.challenge = c.clone();
        c
    }

    pub fn verify(&mut self, nonce: Vec<u8>, response: Vec<u8>, counter: u64) -> AuthResult<bool> {
        if counter <= self.last_seen_counter {
            return Err(make_err!("Replay attack detected: session counter is not increasing"));
        }

        // 1. Verify commitment matches nonce and counter
        let mut commit_data = nonce.clone();
        commit_data.extend_from_slice(&counter.to_le_bytes());
        let expected_commit = sha256(&commit_data);

        let commitment_valid = ct_eq(&self.commitment, &expected_commit);

        // 2. Verify response matches PSK, nonce, challenge, and counter
        let mut resp_data = Vec::with_capacity(
            self.psk.len() + nonce.len() + self.challenge.len() + 8,
        );
        resp_data.extend_from_slice(&self.psk);
        resp_data.extend_from_slice(&nonce);
        resp_data.extend_from_slice(&self.challenge);
        resp_data.extend_from_slice(&counter.to_le_bytes());
        let expected_resp = sha256(&resp_data);

        let response_valid = ct_eq(&response, &expected_resp);

        if commitment_valid && response_valid {
            self.last_seen_counter = counter;
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// Python bindings for ZKPVerifier
#[cfg(feature = "extension-module")]
#[pymethods]
impl ZKPVerifier {
    #[new]
    pub fn py_new(psk: Vec<u8>) -> AuthResult<Self> {
        Self::new(psk)
    }

    #[pyo3(name = "receive_commitment")]
    pub fn py_receive_commitment(&mut self, commitment: Vec<u8>) {
        self.receive_commitment(commitment);
    }

    #[pyo3(name = "create_challenge")]
    pub fn py_create_challenge(&mut self) -> Vec<u8> {
        self.create_challenge()
    }

    #[pyo3(name = "verify")]
    pub fn py_verify(&mut self, nonce: Vec<u8>, response: Vec<u8>, counter: u64) -> AuthResult<bool> {
        self.verify(nonce, response, counter)
    }
}
