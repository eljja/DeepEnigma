//! WebAssembly bindings for E-TPM operations.
//!
//! Exposes E-TPM synchronization functions directly to JavaScript, allowing
//! the exact same Rust cryptographic logic to run in web browsers.

use wasm_bindgen::prelude::*;

#[cfg(not(feature = "std"))]
extern crate alloc;
#[cfg(not(feature = "std"))]
use alloc::vec;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::string::String;

use crate::etpm::ETPM;
use crate::protocol::{KeyExchange, KeyExchangeConfig};

#[wasm_bindgen]
pub struct WasmKeyExchangeResult {
    pub success: bool,
    pub rounds: u32,
    key_hex: String,
    pub sync_time_ms: f64,
}

#[wasm_bindgen]
impl WasmKeyExchangeResult {
    #[wasm_bindgen(getter)]
    pub fn key_hex(&self) -> String {
        self.key_hex.clone()
    }

    pub fn extract_session_key(&self) -> Result<Vec<f64>, JsValue> {
        let native = crate::protocol::KeyExchangeResult {
            success: self.success,
            rounds: self.rounds,
            key_hex: self.key_hex.clone(),
            sync_time_ms: self.sync_time_ms,
        };
        native.extract_session_key().map_err(|e| JsValue::from_str(e))
    }
}

/// Runs a full Alice-Bob key exchange simulation from the browser.
#[wasm_bindgen]
pub fn run_wasm_key_exchange(
    k: usize,
    n: usize,
    l: i32,
    max_rounds: u32,
    update_rule: String,
    activation_type: String,
    adaptive_l_scaling: bool,
    active_query_threshold: i32,
) -> Result<WasmKeyExchangeResult, JsValue> {
    let mut config = KeyExchangeConfig::new(
        k,
        n,
        l,
        max_rounds,
        update_rule,
        activation_type,
        100, // chaotic iterations
        adaptive_l_scaling,
    );

    if active_query_threshold >= 0 {
        config.active_query_threshold = Some(active_query_threshold);
    }

    let mut exchange = KeyExchange::new(&config)
        .map_err(|e| JsValue::from_str(e))?;

    let res = exchange.run()
        .map_err(|e| JsValue::from_str(e))?;

    Ok(WasmKeyExchangeResult {
        success: res.success,
        rounds: res.rounds,
        key_hex: res.key_hex,
        sync_time_ms: res.sync_time_ms,
    })
}

/// A wrapper to initialize ETPM in JS and run custom steps.
#[wasm_bindgen]
pub struct WasmETPM {
    inner: ETPM,
}

#[wasm_bindgen]
impl WasmETPM {
    #[wasm_bindgen(constructor)]
    pub fn new(k: usize, n: usize, l: i32, activation_type: &str) -> Result<WasmETPM, JsValue> {
        let inner = ETPM::new(k, n, l, activation_type)
            .map_err(|e| JsValue::from_str(e))?;
        Ok(Self { inner })
    }

    pub fn calculate_output(&mut self, inputs_flat: Vec<i32>) -> Result<i32, JsValue> {
        // Reconstruct flat inputs into Vec<Vec<i32>> of K x N shape
        let k = self.inner.k;
        let n = self.inner.n;
        if inputs_flat.len() != k * n {
            return Err(JsValue::from_str("Invalid inputs length (must be K * N)"));
        }

        let mut inputs = vec![vec![0; n]; k];
        for i in 0..k {
            for j in 0..n {
                inputs[i][j] = inputs_flat[i * n + j];
            }
        }

        self.inner.calculate_output(inputs).map_err(|e| JsValue::from_str(e))
    }

    pub fn update_weights(&mut self, tau: i32, rule: &str) -> Result<(), JsValue> {
        self.inner.update_weights(tau, rule).map_err(|e| JsValue::from_str(e))
    }

    pub fn get_weights_flat(&self) -> Vec<i32> {
        let mut flat = Vec::new();
        for row in &self.inner.weights {
            flat.extend_from_slice(row);
        }
        flat
    }

    pub fn scale_synaptic_depth(&mut self, new_l: i32) -> Result<(), JsValue> {
        self.inner.scale_synaptic_depth(new_l).map_err(|e| JsValue::from_str(e))
    }

    pub fn chaotic_transform_flat(&self, iterations: u32) -> Vec<i32> {
        let transformed = self.inner.chaotic_transform(iterations);
        let mut flat = Vec::new();
        for row in transformed {
            flat.extend_from_slice(&row);
        }
        flat
    }

    pub fn calculate_local_fields(&self, inputs_flat: Vec<i32>) -> Result<Vec<i32>, JsValue> {
        let k = self.inner.k;
        let n = self.inner.n;
        if inputs_flat.len() != k * n {
            return Err(JsValue::from_str("Invalid inputs length (must be K * N)"));
        }

        let mut inputs = vec![vec![0; n]; k];
        for i in 0..k {
            for j in 0..n {
                inputs[i][j] = inputs_flat[i * n + j];
            }
        }

        Ok(self.inner.calculate_local_fields(&inputs))
    }

    pub fn get_l(&self) -> i32 {
        self.inner.l
    }

    pub fn get_k(&self) -> usize {
        self.inner.k
    }

    pub fn get_n(&self) -> usize {
        self.inner.n
    }
}

// ── WASM Bindings for Neural Cryptography ────────────────────────────────────

#[wasm_bindgen]
pub fn wasm_hamming_encode(data: Vec<f64>) -> Vec<f64> {
    crate::neural::hamming_encode(&data)
}

#[wasm_bindgen]
pub fn wasm_hamming_decode(data: Vec<f64>) -> Vec<f64> {
    crate::neural::hamming_decode(&data)
}

#[wasm_bindgen]
pub struct WasmNeuralNet {
    inner: crate::neural::NeuralNet,
}

#[wasm_bindgen]
impl WasmNeuralNet {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::neural::NeuralNet::new(Vec::new()),
        }
    }

    /// Adds a dense layer to the network.
    /// Weights must be passed as a flat array of size `out_channels * in_channels` in row-major order.
    pub fn add_layer(
        &mut self,
        weights_flat: Vec<f64>,
        biases: Vec<f64>,
        out_channels: usize,
        in_channels: usize,
        act: String,
    ) -> Result<(), JsValue> {
        if weights_flat.len() != out_channels * in_channels {
            return Err(JsValue::from_str("Invalid weights array length: must be out_channels * in_channels"));
        }
        if biases.len() != out_channels {
            return Err(JsValue::from_str("Invalid biases array length: must equal out_channels"));
        }

        let activation = match act.to_lowercase().as_str() {
            "linear" => crate::neural::Activation::Linear,
            "relu" => crate::neural::Activation::ReLU,
            "sigmoid" => crate::neural::Activation::Sigmoid,
            "step" => crate::neural::Activation::Step,
            _ => return Err(JsValue::from_str("Invalid activation function name")),
        };

        let mut weights = vec![vec![0.0; in_channels]; out_channels];
        for i in 0..out_channels {
            for j in 0..in_channels {
                weights[i][j] = weights_flat[i * in_channels + j];
            }
        }

        let dense = crate::neural::DenseLayer::new(weights, biases, activation);
        self.inner.layers.push(dense);
        Ok(())
    }

    pub fn forward(&self, input: Vec<f64>) -> Vec<f64> {
        self.inner.forward(&input)
    }
}

#[wasm_bindgen]
pub fn wasm_quantize(x: f64, scale: f64) -> i8 {
    crate::neural::quantize(x, scale)
}

#[wasm_bindgen]
pub fn wasm_dequantize(q: i8, scale: f64) -> f64 {
    crate::neural::dequantize(q, scale)
}

#[wasm_bindgen]
pub struct WasmIntegerNeuralNet {
    inner: crate::neural::IntegerNeuralNet,
}

#[wasm_bindgen]
impl WasmIntegerNeuralNet {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            inner: crate::neural::IntegerNeuralNet::new(Vec::new()),
        }
    }

    pub fn add_layer(
        &mut self,
        weights_flat: Vec<i8>,
        biases: Vec<i32>,
        out_channels: usize,
        in_channels: usize,
        scale_in: f64,
        scale_w: f64,
        scale_out: f64,
        act: String,
    ) -> Result<(), JsValue> {
        if weights_flat.len() != out_channels * in_channels {
            return Err(JsValue::from_str("Invalid weights array length"));
        }
        if biases.len() != out_channels {
            return Err(JsValue::from_str("Invalid biases array length"));
        }

        let activation = match act.to_lowercase().as_str() {
            "linear" => crate::neural::Activation::Linear,
            "relu" => crate::neural::Activation::ReLU,
            "sigmoid" => crate::neural::Activation::Sigmoid,
            "step" => crate::neural::Activation::Step,
            _ => return Err(JsValue::from_str("Invalid activation")),
        };

        let mut weights = vec![vec![0; in_channels]; out_channels];
        for i in 0..out_channels {
            for j in 0..in_channels {
                weights[i][j] = weights_flat[i * in_channels + j];
            }
        }

        let layer = crate::neural::IntegerDenseLayer::new(
            weights,
            biases,
            scale_in,
            scale_w,
            scale_out,
            activation,
        );
        self.inner.layers.push(layer);
        Ok(())
    }

    pub fn forward(&self, input: Vec<i8>) -> Vec<i8> {
        self.inner.forward(&input)
    }
}


