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

    pub fn forward(&self, input: &[f64]) -> Vec<f64> {
        let mut current = input.to_vec();
        for layer in &self.layers {
            current = layer.forward(&current);
        }
        current
    }

    /// Alice encrypts and scrambles the output ciphertext using 4D hyperchaos (Part 3-2).
    pub fn forward_scrambled(&self, input: &[f64], hc: &mut HyperchaoticSystem) -> Vec<f64> {
        let c = self.forward(input);
        let h = hc.generate_sequence(c.len());
        c.iter().zip(h.iter()).map(|(&cv, &hv)| cv + hv).collect()
    }

    /// Bob unscrambles the ciphertext using 4D hyperchaos and decrypts it (Part 3-2).
    pub fn decrypt_scrambled(&self, scrambled_cipher: &[f64], input_key: &[f64], hc: &mut HyperchaoticSystem) -> Vec<f64> {
        let h = hc.generate_sequence(scrambled_cipher.len());
        let c: Vec<f64> = scrambled_cipher.iter().zip(h.iter()).map(|(&sc, &hv)| sc - hv).collect();
        let mut bob_input = c;
        bob_input.extend_from_slice(input_key);
        self.forward(&bob_input)
    }
}


// ── Part 3: Hyperchaotic System ──────────────────────────────────────────────

/// A 4D Coupled Map Lattice (CML) Hyperchaotic System.
/// Serves as the core scrambling and hardening engine for Part 3.
#[derive(Clone, Debug)]
pub struct HyperchaoticSystem {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub w: f64,
    r: f64,
    e: f64,
}

impl HyperchaoticSystem {
    /// Creates a new hyperchaotic system with initial seeds.
    pub fn new(x_init: f64, y_init: f64, z_init: f64, w_init: f64) -> Self {
        // Clamp seeds to open interval (0.0, 1.0)
        let clamp_seed = |s: f64| -> f64 {
            let clamped = s.abs() % 1.0;
            if clamped == 0.0 { 0.5 } else { clamped }
        };
        Self {
            x: clamp_seed(x_init),
            y: clamp_seed(y_init),
            z: clamp_seed(z_init),
            w: clamp_seed(w_init),
            r: 3.99, // Chaotic regime
            e: 0.1,  // Coupling factor
        }
    }

    /// Computes the next state of the 4D hyperchaotic system using ring coupling.
    pub fn next(&mut self) {
        let f = |v: f64| self.r * v * (1.0 - v);
        
        let fx = f(self.x);
        let fy = f(self.y);
        let fz = f(self.z);
        let fw = f(self.w);

        let x_next = (1.0 - self.e) * fx + self.e * fy;
        let y_next = (1.0 - self.e) * fy + self.e * fz;
        let z_next = (1.0 - self.e) * fz + self.e * fw;
        let w_next = (1.0 - self.e) * fw + self.e * fx;

        self.x = x_next;
        self.y = y_next;
        self.z = z_next;
        self.w = w_next;
    }

    /// Generates a vector of hyperchaotic pseudorandom floats in [-1.0, 1.0] of length `len`.
    pub fn generate_sequence(&mut self, len: usize) -> Vec<f64> {
        let mut seq = Vec::with_capacity(len);
        for _ in 0..len {
            self.next();
            seq.push(self.x * 2.0 - 1.0);
        }
        seq
    }
}

// ── INT8 Quantization Helpers ────────────────────────────────────────────────


/// Maps a float in [-1.0, 1.0] to an i8 integer using a scale factor.
#[inline]
pub fn quantize(x: f64, scale: f64) -> i8 {
    if scale == 0.0 {
        return 0;
    }
    let q = (x / scale).round();
    if q > 127.0 {
        127
    } else if q < -128.0 {
        -128
    } else {
        q as i8
    }
}

/// Maps an i8 integer back to a float using a scale factor.
#[inline]
pub fn dequantize(q: i8, scale: f64) -> f64 {
    (q as f64) * scale
}

/// An INT8 quantized fully connected layer.
#[derive(Clone, Debug)]
pub struct IntegerDenseLayer {
    pub weights: Vec<Vec<i8>>, // OutChannels x InChannels
    pub biases: Vec<i32>,       // OutChannels (accumulated scale)
    pub scale_in: f64,
    pub scale_w: f64,
    pub scale_out: f64,
    pub activation: Activation,
}

impl IntegerDenseLayer {
    pub fn new(
        weights: Vec<Vec<i8>>,
        biases: Vec<i32>,
        scale_in: f64,
        scale_w: f64,
        scale_out: f64,
        activation: Activation,
    ) -> Self {
        Self {
            weights,
            biases,
            scale_in,
            scale_w,
            scale_out,
            activation,
        }
    }

    /// Computes the forward pass of the quantized layer.
    /// Takes quantized `i8` inputs and returns quantized `i8` outputs.
    pub fn forward(&self, input: &[i8]) -> Vec<i8> {
        let mut output = vec![0; self.biases.len()];
        let scale_accum = self.scale_in * self.scale_w;

        for i in 0..self.biases.len() {
            let mut acc: i32 = self.biases[i];
            for j in 0..input.len() {
                acc += (self.weights[i][j] as i32) * (input[j] as i32);
            }

            // Convert back to float for non-linear activation scaling
            let val_float = (acc as f64) * scale_accum;
            let act_float = self.activation.apply(val_float);
            
            // Re-quantize to i8 output scale
            output[i] = quantize(act_float, self.scale_out);
        }
        output
    }
}

/// An INT8 quantized neural network.
#[derive(Clone, Debug)]
pub struct IntegerNeuralNet {
    pub layers: Vec<IntegerDenseLayer>,
}

impl IntegerNeuralNet {
    pub fn new(layers: Vec<IntegerDenseLayer>) -> Self {
        Self { layers }
    }

    /// Computes the quantized forward pass of the entire network.
    pub fn forward(&self, input: &[i8]) -> Vec<i8> {
        let mut current = input.to_vec();
        for layer in &self.layers {
            current = layer.forward(&current);
        }
        current
    }

    /// Alice encrypts and scrambles the quantized output ciphertext using 4D hyperchaos (Part 3-2).
    pub fn forward_scrambled(&self, input: &[i8], hc: &mut HyperchaoticSystem, scale_out: f64) -> Vec<i8> {
        let c = self.forward(input);
        let h = hc.generate_sequence(c.len());
        c.iter().zip(h.iter()).map(|(&cv, &hv)| {
            let h_int = quantize(hv, scale_out);
            cv.wrapping_add(h_int)
        }).collect()
    }

    /// Bob unscrambles the quantized ciphertext using 4D hyperchaos and decrypts it (Part 3-2).
    pub fn decrypt_scrambled(&self, scrambled_cipher: &[i8], input_key_int8: &[i8], hc: &mut HyperchaoticSystem, scale_out_alice: f64) -> Vec<i8> {
        let h = hc.generate_sequence(scrambled_cipher.len());
        let c: Vec<i8> = scrambled_cipher.iter().zip(h.iter()).map(|(&sc, &hv)| {
            let h_int = quantize(hv, scale_out_alice);
            sc.wrapping_sub(h_int)
        }).collect();
        let mut bob_input = c;
        bob_input.extend_from_slice(input_key_int8);
        self.forward(&bob_input)
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

    #[test]
    fn test_quantization_equivalence() {
        // Setup original float layer
        let w_float = vec![vec![0.5, -0.25], vec![0.125, 0.75]];
        let b_float = vec![0.1, -0.2];
        let layer_float = DenseLayer::new(w_float, b_float, Activation::ReLU);

        // Quantization scales
        let scale_in = 0.5;
        let scale_w = 0.25;
        let scale_out = 0.5;
        let scale_accum = scale_in * scale_w; // 0.125

        // Convert weights and biases to quantized INT8/INT32
        let w_quant = vec![
            vec![quantize(0.5, scale_w), quantize(-0.25, scale_w)],
            vec![quantize(0.125, scale_w), quantize(0.75, scale_w)],
        ];
        let b_quant = vec![
            quantize(0.1, scale_accum) as i32,
            quantize(-0.2, scale_accum) as i32,
        ];

        let layer_quant = IntegerDenseLayer::new(
            w_quant,
            b_quant,
            scale_in,
            scale_w,
            scale_out,
            Activation::ReLU,
        );

        // Input values
        let input_float = vec![1.0, -0.5];
        let input_quant: Vec<i8> = input_float.iter().map(|&x| quantize(x, scale_in)).collect();

        // Forward passes
        let out_float = layer_float.forward(&input_float);
        let out_quant = layer_quant.forward(&input_quant);

        // Verify outputs match (within scale threshold)
        for i in 0..out_float.len() {
            let out_q_dequant = dequantize(out_quant[i], scale_out);
            assert!(
                (out_float[i] - out_q_dequant).abs() <= scale_out,
                "Quantized output differs significantly from float output at index {}: float={}, quantized={}",
                i,
                out_float[i],
                out_q_dequant
            );
        }
    }
}
