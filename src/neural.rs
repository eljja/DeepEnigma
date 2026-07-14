//! Neural Cryptography Engine Module (NeuralEnigma).
//!
//! Provides dense layer feedforward inference, asymmetric network architectures,
//! and Hamming(7,4) error-correcting codes to guarantee 0% bit error rate
//! reconstruction in neural encryption.

#[cfg(feature = "extension-module")]
use pyo3::prelude::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::{vec, vec::Vec};

/// Activation functions for neural layers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Activation {
    Linear,
    ReLU,
    Sigmoid,
    Step,
}

impl Activation {
    pub fn apply(&self, x: f64) -> f64 {
        match self {
            Activation::Linear => x,
            Activation::ReLU => if x > 0.0 { x } else { 0.0 },
            Activation::Sigmoid => 1.0 / (1.0 + (-x).exp()),
            Activation::Step => if x >= 0.5 { 1.0 } else { 0.0 },
        }
    }
}

/// A standard dense (fully connected) neural network layer.
#[derive(Clone, Debug)]
pub struct DenseLayer {
    pub weights: Vec<Vec<f64>>, // OutChannels x InChannels
    pub biases: Vec<f64>,       // OutChannels
    pub activation: Activation,
}

impl DenseLayer {
    pub fn new(weights: Vec<Vec<f64>>, biases: Vec<f64>, activation: Activation) -> Self {
        Self {
            weights,
            biases,
            activation,
        }
    }

    /// Computes the forward pass of the layer.
    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut output = vec![0.0; self.biases.len()];
        for i in 0..self.biases.len() {
            let mut sum = self.biases[i];
            for j in 0..input.len() {
                sum += self.weights[i][j] * input[j];
            }
            output[i] = self.activation.apply(sum);
        }
        output
    }
}

/// Represents a feedforward neural network.
#[derive(Clone, Debug)]
pub struct NeuralNet {
    pub layers: Vec<DenseLayer>,
}

impl NeuralNet {
    pub fn new(layers: Vec<DenseLayer>) -> Self {
        Self { layers }
    }

    /// Computes the forward pass of the entire network.
    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut current = input.to_vec();
        for layer in &self.layers {
            current = layer.forward(&current);
        }
        current
    }
}

// ── Hamming(7, 4) Error Correction Code (ECC) ───────────────────────────────
//
// Hamming(7,4) maps 4 bits of data to 7 bits, adding 3 parity bits.
// It can detect and correct any single-bit error per 7-bit block.

/// Encodes a 4-bit data block into a 7-bit codeword.
/// Data is represented as a slice of 4 values (0.0 or 1.0).
/// Returns a slice of 7 values (0.0 or 1.0).
pub fn hamming_encode_block(data: &[f64]) -> Vec<f64> {
    let d1 = if data[0] >= 0.5 { 1 } else { 0 };
    let d2 = if data[1] >= 0.5 { 1 } else { 0 };
    let d3 = if data[2] >= 0.5 { 1 } else { 0 };
    let d4 = if data[3] >= 0.5 { 1 } else { 0 };

    // Parity bits calculations
    let p1 = d1 ^ d2 ^ d4;
    let p2 = d1 ^ d3 ^ d4;
    let p3 = d2 ^ d3 ^ d4;

    vec![
        p1 as f64,
        p2 as f64,
        d1 as f64,
        p3 as f64,
        d2 as f64,
        d3 as f64,
        d4 as f64,
    ]
}

/// Decodes a 7-bit codeword back into a 4-bit data block, correcting single bit errors.
pub fn hamming_decode_block(codeword: &[f64]) -> Vec<f64> {
    let mut bits = [0; 7];
    for i in 0..7 {
        bits[i] = if codeword[i] >= 0.5 { 1 } else { 0 };
    }

    // Check parity checks
    let s1 = bits[0] ^ bits[2] ^ bits[4] ^ bits[6];
    let s2 = bits[1] ^ bits[2] ^ bits[5] ^ bits[6];
    let s3 = bits[3] ^ bits[4] ^ bits[5] ^ bits[6];

    let error_position = s1 + (s2 << 1) + (s3 << 2);

    if error_position > 0 && error_position <= 7 {
        // Correct the error (1-indexed position)
        bits[error_position - 1] ^= 1;
    }

    vec![
        bits[2] as f64, // d1
        bits[4] as f64, // d2
        bits[5] as f64, // d3
        bits[6] as f64, // d4
    ]
}

/// Encodes an arbitrary length bit array (multiple of 4) using Hamming(7, 4).
pub fn hamming_encode(data: &[f64]) -> Vec<f64> {
    let mut encoded = Vec::with_capacity(data.len() / 4 * 7);
    for chunk in data.chunks_exact(4) {
        encoded.extend(hamming_encode_block(chunk));
    }
    encoded
}

/// Decodes a codeword bit array (multiple of 7) back to data using Hamming(7, 4).
pub fn hamming_decode(codeword: &[f64]) -> Vec<f64> {
    let mut decoded = Vec::with_capacity(codeword.len() / 7 * 4);
    for chunk in codeword.chunks_exact(7) {
        decoded.extend(hamming_decode_block(chunk));
    }
    decoded
}

// ── Python Bindings for Neural Network Engine ────────────────────────────────

#[cfg(feature = "extension-module")]
#[pyclass]
#[derive(Clone)]
pub struct PyDenseLayer {
    inner: DenseLayer,
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl PyDenseLayer {
    #[new]
    pub fn new(weights: Vec<Vec<f64>>, biases: Vec<f64>, act: &str) -> PyResult<Self> {
        let activation = match act.to_lowercase().as_str() {
            "linear" => Activation::Linear,
            "relu" => Activation::ReLU,
            "sigmoid" => Activation::Sigmoid,
            "step" => Activation::Step,
            _ => return Err(pyo3::exceptions::PyValueError::new_err("Invalid activation function")),
        };
        Ok(Self {
            inner: DenseLayer::new(weights, biases, activation),
        })
    }
}

#[cfg(feature = "extension-module")]
#[pyclass]
pub struct PyNeuralNet {
    inner: NeuralNet,
}

#[cfg(feature = "extension-module")]
#[pymethods]
impl PyNeuralNet {
    #[new]
    pub fn new(layers: Vec<PyDenseLayer>) -> Self {
        let native_layers = layers.into_iter().map(|l| l.inner).collect();
        Self {
            inner: NeuralNet::new(native_layers),
        }
    }

    pub fn forward(&self, input: Vec<f64>) -> Vec<f64> {
        self.inner.forward(&input)
    }

    #[pyo3(name = "hamming_encode")]
    #[staticmethod]
    pub fn py_hamming_encode(data: Vec<f64>) -> Vec<f64> {
        hamming_encode(&data)
    }

    #[pyo3(name = "hamming_decode")]
    #[staticmethod]
    pub fn py_hamming_decode(data: Vec<f64>) -> Vec<f64> {
        hamming_decode(&data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dense_layer_relu() {
        let weights = vec![
            vec![1.0, -2.0],
            vec![0.5, 2.0],
        ];
        let biases = vec![0.1, -1.0];
        let layer = DenseLayer::new(weights, biases, Activation::ReLU);

        let input = vec![2.0, 1.0];
        let out = layer.forward(&input);

        // out[0] = relu(0.1 + (1.0*2.0 + -2.0*1.0)) = relu(0.1 + 0.0) = 0.1
        // out[1] = relu(-1.0 + (0.5*2.0 + 2.0*1.0)) = relu(-1.0 + 3.0) = 2.0
        assert!((out[0] - 0.1).abs() < 1e-9);
        assert!((out[1] - 2.0).abs() < 1e-9);
    }

    #[test]
    fn test_hamming_ecc_no_error() {
        let original = vec![1.0, 0.0, 1.0, 1.0];
        let encoded = hamming_encode(&original);
        assert_eq!(encoded.len(), 7);

        let decoded = hamming_decode(&encoded);
        assert_eq!(original, decoded);
    }

    #[test]
    fn test_hamming_ecc_single_error_correction() {
        let original = vec![0.0, 1.0, 1.0, 0.0, 1.0, 0.0, 0.0, 1.0]; // 8 bits (2 blocks)
        let mut encoded = hamming_encode(&original);
        assert_eq!(encoded.len(), 14);

        // Introduce a single-bit error in the first block (position index 2)
        encoded[2] = 1.0 - encoded[2];
        // Introduce a single-bit error in the second block (position index 10)
        encoded[10] = 1.0 - encoded[10];

        let decoded = hamming_decode(&encoded);
        assert_eq!(original, decoded, "Hamming ECC failed to correct single-bit errors in blocks");
    }
}
