//! Protocol Handshake and Parameter Negotiation module.
//!
//! Provides structures and validation logic for Alice and Bob to negotiate
//! cryptographic E-TPM parameters (K, N, L) and verify protocol version compatibility
//! before starting key exchange.

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::string::ToString;

/// Result type alias supporting both PyO3 and pure Rust environments.
#[cfg(feature = "extension-module")]
type HandshakeResult<T> = PyResult<T>;

#[cfg(not(feature = "extension-module"))]
type HandshakeResult<T> = Result<T, &'static str>;

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

/// Message exchanged during the initial handshake phase.
#[cfg_attr(feature = "extension-module", pyclass)]
#[derive(Clone, Debug)]
pub struct HandshakeMessage {
    /// Protocol version identifier (e.g., "DeepEnigma-v1").
    pub version: String,
    /// Proposed number of hidden units (K).
    pub k: usize,
    /// Proposed number of input neurons per unit (N).
    pub n: usize,
    /// Proposed synaptic depth limit (L).
    pub l: i32,
    /// Proposed activation type ("standard", "chaotic", "hybrid").
    pub activation_type: String,
    /// Proposed update rule ("hebbian", "antihebbian", "randomwalk").
    pub update_rule: String,
    /// Alice's ZKP commitment (32 bytes), if authentication is enabled.
    pub commitment: Vec<u8>,
    /// Proposed active query threshold (None if disabled).
    pub active_query_threshold: Option<i32>,
}

impl HandshakeMessage {
    pub fn new(
        k: usize,
        n: usize,
        l: i32,
        activation_type: String,
        update_rule: String,
        commitment: Vec<u8>,
    ) -> Self {
        Self {
            version: "DeepEnigma-v1".to_string(),
            k,
            n,
            l,
            activation_type,
            update_rule,
            commitment,
            active_query_threshold: None,
        }
    }

    /// Serializes the handshake message into a byte vector for transmission.
    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        // Version length (1 byte) + version string
        data.push(self.version.len() as u8);
        data.extend_from_slice(self.version.as_bytes());
        // K (8 bytes), N (8 bytes), L (4 bytes)
        data.extend_from_slice(&(self.k as u64).to_le_bytes());
        data.extend_from_slice(&(self.n as u64).to_le_bytes());
        data.extend_from_slice(&self.l.to_le_bytes());
        // Activation type string (length + string)
        data.push(self.activation_type.len() as u8);
        data.extend_from_slice(self.activation_type.as_bytes());
        // Update rule string (length + string)
        data.push(self.update_rule.len() as u8);
        data.extend_from_slice(self.update_rule.as_bytes());
        // Commitment length (1 byte) + commitment bytes
        data.push(self.commitment.len() as u8);
        data.extend_from_slice(&self.commitment);

        // Active query threshold presence (1 byte) + optional value (4 bytes)
        if let Some(h) = self.active_query_threshold {
            data.push(1);
            data.extend_from_slice(&h.to_le_bytes());
        } else {
            data.push(0);
        }
        data
    }

    /// Deserializes a byte vector back into a HandshakeMessage.
    pub fn deserialize(data: Vec<u8>) -> HandshakeResult<Self> {
        if data.is_empty() {
            return Err(make_err!("Empty handshake data"));
        }
        let mut offset = 0;

        // Version
        let v_len = data[offset] as usize;
        offset += 1;
        if offset + v_len > data.len() {
            return Err(make_err!("Malformed version field"));
        }
        let version = String::from_utf8(data[offset..offset+v_len].to_vec())
            .map_err(|_| make_err!("Invalid UTF-8 in version"))?;
        offset += v_len;

        // K
        if offset + 8 > data.len() {
            return Err(make_err!("Malformed K field"));
        }
        let mut k_bytes = [0u8; 8];
        k_bytes.copy_from_slice(&data[offset..offset+8]);
        let k = u64::from_le_bytes(k_bytes) as usize;
        offset += 8;

        // N
        if offset + 8 > data.len() {
            return Err(make_err!("Malformed N field"));
        }
        let mut n_bytes = [0u8; 8];
        n_bytes.copy_from_slice(&data[offset..offset+8]);
        let n = u64::from_le_bytes(n_bytes) as usize;
        offset += 8;

        // L
        if offset + 4 > data.len() {
            return Err(make_err!("Malformed L field"));
        }
        let mut l_bytes = [0u8; 4];
        l_bytes.copy_from_slice(&data[offset..offset+4]);
        let l = i32::from_le_bytes(l_bytes);
        offset += 4;

        // Activation type
        if offset >= data.len() {
            return Err(make_err!("Missing activation_type length"));
        }
        let act_len = data[offset] as usize;
        offset += 1;
        if offset + act_len > data.len() {
            return Err(make_err!("Malformed activation_type field"));
        }
        let activation_type = String::from_utf8(data[offset..offset+act_len].to_vec())
            .map_err(|_| make_err!("Invalid UTF-8 in activation_type"))?;
        offset += act_len;

        // Update rule
        if offset >= data.len() {
            return Err(make_err!("Missing update_rule length"));
        }
        let rule_len = data[offset] as usize;
        offset += 1;
        if offset + rule_len > data.len() {
            return Err(make_err!("Malformed update_rule field"));
        }
        let update_rule = String::from_utf8(data[offset..offset+rule_len].to_vec())
            .map_err(|_| make_err!("Invalid UTF-8 in update_rule"))?;
        offset += rule_len;

        // Commitment
        if offset >= data.len() {
            return Err(make_err!("Missing commitment length"));
        }
        let commit_len = data[offset] as usize;
        offset += 1;
        if offset + commit_len > data.len() {
            return Err(make_err!("Malformed commitment field"));
        }
        let commitment = data[offset..offset+commit_len].to_vec();
        offset += commit_len;

        // Active query threshold
        let active_query_threshold = if offset < data.len() {
            let threshold_present = data[offset];
            offset += 1;
            if threshold_present == 1 {
                if offset + 4 > data.len() {
                    return Err(make_err!("Malformed active_query_threshold field"));
                }
                let mut h_bytes = [0u8; 4];
                h_bytes.copy_from_slice(&data[offset..offset+4]);
                let h = i32::from_le_bytes(h_bytes);
                Some(h)
            } else {
                None
            }
        } else {
            None
        };

        Ok(Self {
            version,
            k,
            n,
            l,
            activation_type,
            update_rule,
            commitment,
            active_query_threshold,
        })
    }
}

// Python bindings for HandshakeMessage
#[cfg(feature = "extension-module")]
#[pymethods]
impl HandshakeMessage {
    #[new]
    #[pyo3(signature = (k, n, l, activation_type = "hybrid".to_string(), update_rule = "hebbian".to_string(), commitment = vec![], active_query_threshold = None))]
    pub fn py_new(
        k: usize,
        n: usize,
        l: i32,
        activation_type: String,
        update_rule: String,
        commitment: Vec<u8>,
        active_query_threshold: Option<i32>,
    ) -> Self {
        let mut msg = Self::new(k, n, l, activation_type, update_rule, commitment);
        msg.active_query_threshold = active_query_threshold;
        msg
    }

    #[pyo3(name = "serialize")]
    pub fn py_serialize(&self) -> Vec<u8> {
        self.serialize()
    }

    #[pyo3(name = "deserialize")]
    #[staticmethod]
    pub fn py_deserialize(data: Vec<u8>) -> HandshakeResult<Self> {
        Self::deserialize(data)
    }

    #[getter]
    pub fn version(&self) -> String {
        self.version.clone()
    }
    #[setter]
    pub fn set_version(&mut self, value: String) {
        self.version = value;
    }

    #[getter]
    pub fn k(&self) -> usize {
        self.k
    }
    #[setter]
    pub fn set_k(&mut self, value: usize) {
        self.k = value;
    }

    #[getter]
    pub fn n(&self) -> usize {
        self.n
    }
    #[setter]
    pub fn set_n(&mut self, value: usize) {
        self.n = value;
    }

    #[getter]
    pub fn l(&self) -> i32 {
        self.l
    }
    #[setter]
    pub fn set_l(&mut self, value: i32) {
        self.l = value;
    }

    #[getter]
    pub fn activation_type(&self) -> String {
        self.activation_type.clone()
    }
    #[setter]
    pub fn set_activation_type(&mut self, value: String) {
        self.activation_type = value;
    }

    #[getter]
    pub fn update_rule(&self) -> String {
        self.update_rule.clone()
    }
    #[setter]
    pub fn set_update_rule(&mut self, value: String) {
        self.update_rule = value;
    }

    #[getter]
    pub fn commitment(&self) -> Vec<u8> {
        self.commitment.clone()
    }
    #[setter]
    pub fn set_commitment(&mut self, value: Vec<u8>) {
        self.commitment = value;
    }

    #[getter]
    pub fn active_query_threshold(&self) -> Option<i32> {
        self.active_query_threshold
    }
    #[setter]
    pub fn set_active_query_threshold(&mut self, value: Option<i32>) {
        self.active_query_threshold = value;
    }
}

/// Negotiates parameters between Alice's proposal and Bob's constraints.
#[cfg_attr(feature = "extension-module", pyclass)]
pub struct ParameterNegotiator;

impl ParameterNegotiator {
    pub fn negotiate(alice: &HandshakeMessage, bob: &HandshakeMessage) -> HandshakeResult<HandshakeMessage> {
        if alice.version != bob.version {
            return Err(make_err!("Protocol version mismatch"));
        }

        if alice.k != bob.k {
            return Err(make_err!("Incompatible E-TPM layout: K mismatch"));
        }

        if alice.n != bob.n {
            return Err(make_err!("Incompatible E-TPM layout: N mismatch"));
        }

        if alice.activation_type != bob.activation_type {
            return Err(make_err!("Activation type mismatch"));
        }

        if alice.update_rule != bob.update_rule {
            return Err(make_err!("Update rule mismatch"));
        }

        // L negotiation: select the maximum to raise security strength.
        let agreed_l = core::cmp::max(alice.l, bob.l);

        // Active query threshold negotiation: select the minimum of proposed values if both present, else None
        let agreed_threshold = match (alice.active_query_threshold, bob.active_query_threshold) {
            (Some(h1), Some(h2)) => Some(core::cmp::min(h1, h2)),
            _ => None,
        };

        Ok(HandshakeMessage {
            version: alice.version.clone(),
            k: alice.k,
            n: alice.n,
            l: agreed_l,
            activation_type: alice.activation_type.clone(),
            update_rule: alice.update_rule.clone(),
            commitment: bob.commitment.clone(), // Verifier commitment returned
            active_query_threshold: agreed_threshold,
        })
    }
}

// Python bindings for ParameterNegotiator
#[cfg(feature = "extension-module")]
#[pymethods]
impl ParameterNegotiator {
    #[pyo3(name = "negotiate")]
    #[staticmethod]
    pub fn py_negotiate(alice: &HandshakeMessage, bob: &HandshakeMessage) -> HandshakeResult<HandshakeMessage> {
        Self::negotiate(alice, bob)
    }
}
