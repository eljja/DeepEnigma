//! Protocol Handshake and Parameter Negotiation module.
//!
//! Provides structures and validation logic for Alice and Bob to negotiate
//! cryptographic E-TPM parameters (K, N, L) and verify protocol version compatibility
//! before starting key exchange.

use pyo3::prelude::*;

/// Message exchanged during the initial handshake phase.
#[pyclass]
#[derive(Clone, Debug)]
pub struct HandshakeMessage {
    /// Protocol version identifier (e.g., "DeepEnigma-v1").
    #[pyo3(get, set)]
    pub version: String,
    /// Proposed number of hidden units (K).
    #[pyo3(get, set)]
    pub k: usize,
    /// Proposed number of input neurons per unit (N).
    #[pyo3(get, set)]
    pub n: usize,
    /// Proposed synaptic depth limit (L).
    #[pyo3(get, set)]
    pub l: i32,
    /// Proposed activation type ("standard", "chaotic", "hybrid").
    #[pyo3(get, set)]
    pub activation_type: String,
    /// Proposed update rule ("hebbian", "antihebbian", "randomwalk").
    #[pyo3(get, set)]
    pub update_rule: String,
    /// Alice's ZKP commitment (32 bytes), if authentication is enabled.
    #[pyo3(get, set)]
    pub commitment: Vec<u8>,
}

#[pymethods]
impl HandshakeMessage {
    #[new]
    #[pyo3(signature = (k, n, l, activation_type = "hybrid".to_string(), update_rule = "hebbian".to_string(), commitment = vec![]))]
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
        data
    }

    /// Deserializes a byte vector back into a HandshakeMessage.
    #[staticmethod]
    pub fn deserialize(data: Vec<u8>) -> PyResult<Self> {
        if data.is_empty() {
            return Err(pyo3::exceptions::PyValueError::new_err("Empty handshake data"));
        }
        let mut offset = 0;

        // Version
        let v_len = data[offset] as usize;
        offset += 1;
        if offset + v_len > data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Malformed version field"));
        }
        let version = String::from_utf8(data[offset..offset+v_len].to_vec())
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid UTF-8 in version"))?;
        offset += v_len;

        // K
        if offset + 8 > data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Malformed K field"));
        }
        let mut k_bytes = [0u8; 8];
        k_bytes.copy_from_slice(&data[offset..offset+8]);
        let k = u64::from_le_bytes(k_bytes) as usize;
        offset += 8;

        // N
        if offset + 8 > data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Malformed N field"));
        }
        let mut n_bytes = [0u8; 8];
        n_bytes.copy_from_slice(&data[offset..offset+8]);
        let n = u64::from_le_bytes(n_bytes) as usize;
        offset += 8;

        // L
        if offset + 4 > data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Malformed L field"));
        }
        let mut l_bytes = [0u8; 4];
        l_bytes.copy_from_slice(&data[offset..offset+4]);
        let l = i32::from_le_bytes(l_bytes);
        offset += 4;

        // Activation type
        if offset >= data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Missing activation_type length"));
        }
        let act_len = data[offset] as usize;
        offset += 1;
        if offset + act_len > data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Malformed activation_type field"));
        }
        let activation_type = String::from_utf8(data[offset..offset+act_len].to_vec())
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid UTF-8 in activation_type"))?;
        offset += act_len;

        // Update rule
        if offset >= data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Missing update_rule length"));
        }
        let rule_len = data[offset] as usize;
        offset += 1;
        if offset + rule_len > data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Malformed update_rule field"));
        }
        let update_rule = String::from_utf8(data[offset..offset+rule_len].to_vec())
            .map_err(|_| pyo3::exceptions::PyValueError::new_err("Invalid UTF-8 in update_rule"))?;
        offset += rule_len;

        // Commitment
        if offset >= data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Missing commitment length"));
        }
        let commit_len = data[offset] as usize;
        offset += 1;
        if offset + commit_len > data.len() {
            return Err(pyo3::exceptions::PyValueError::new_err("Malformed commitment field"));
        }
        let commitment = data[offset..offset+commit_len].to_vec();

        Ok(Self {
            version,
            k,
            n,
            l,
            activation_type,
            update_rule,
            commitment,
        })
    }
}

/// Negotiates parameters between Alice's proposal and Bob's constraints.
///
/// Rules for negotiation:
/// 1. Protocol version must match exactly.
/// 2. If K or N parameters mismatch, negotiation fails (E-TPM structures are incompatible).
/// 3. L (synaptic depth) is negotiated to the **maximum** of the two proposed depths
///    to optimize security against geometric attacks.
/// 4. Activation type and update rule must match.
#[pyclass]
pub struct ParameterNegotiator;

#[pymethods]
impl ParameterNegotiator {
    /// Validates and negotiates two handshake messages, returning the agreed parameter configuration.
    #[staticmethod]
    pub fn negotiate(alice: &HandshakeMessage, bob: &HandshakeMessage) -> PyResult<HandshakeMessage> {
        if alice.version != bob.version {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Protocol version mismatch: Alice proposed {}, Bob proposed {}",
                alice.version, bob.version
            )));
        }

        if alice.k != bob.k {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Incompatible E-TPM layout: Alice proposed K={}, Bob proposed K={}",
                alice.k, bob.k
            )));
        }

        if alice.n != bob.n {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Incompatible E-TPM layout: Alice proposed N={}, Bob proposed N={}",
                alice.n, bob.n
            )));
        }

        if alice.activation_type != bob.activation_type {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Activation type mismatch: Alice proposed {}, Bob proposed {}",
                alice.activation_type, bob.activation_type
            )));
        }

        if alice.update_rule != bob.update_rule {
            return Err(pyo3::exceptions::PyValueError::new_err(format!(
                "Update rule mismatch: Alice proposed {}, Bob proposed {}",
                alice.update_rule, bob.update_rule
            )));
        }

        // L negotiation: select the maximum to raise security strength.
        let agreed_l = std::cmp::max(alice.l, bob.l);

        Ok(HandshakeMessage {
            version: alice.version.clone(),
            k: alice.k,
            n: alice.n,
            l: agreed_l,
            activation_type: alice.activation_type.clone(),
            update_rule: alice.update_rule.clone(),
            commitment: bob.commitment.clone(), // Verifier commitment returned
        })
    }
}
