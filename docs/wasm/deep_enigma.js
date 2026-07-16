/* @ts-self-types="./deep_enigma.d.ts" */

/**
 * A wrapper to initialize ETPM in JS and run custom steps.
 */
export class WasmETPM {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmETPMFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmetpm_free(ptr, 0);
    }
    /**
     * @param {Int32Array} inputs_flat
     * @returns {Int32Array}
     */
    calculate_local_fields(inputs_flat) {
        const ptr0 = passArray32ToWasm0(inputs_flat, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmetpm_calculate_local_fields(this.__wbg_ptr, ptr0, len0);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v2 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v2;
    }
    /**
     * @param {Int32Array} inputs_flat
     * @returns {number}
     */
    calculate_output(inputs_flat) {
        const ptr0 = passArray32ToWasm0(inputs_flat, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmetpm_calculate_output(this.__wbg_ptr, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        return ret[0];
    }
    /**
     * @param {number} iterations
     * @returns {Int32Array}
     */
    chaotic_transform_flat(iterations) {
        const ret = wasm.wasmetpm_chaotic_transform_flat(this.__wbg_ptr, iterations);
        var v1 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @returns {number}
     */
    get_k() {
        const ret = wasm.wasmetpm_get_k(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get_l() {
        const ret = wasm.wasmetpm_get_l(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get_n() {
        const ret = wasm.wasmetpm_get_n(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {Int32Array}
     */
    get_weights_flat() {
        const ret = wasm.wasmetpm_get_weights_flat(this.__wbg_ptr);
        var v1 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {number} iterations
     * @returns {Int32Array}
     */
    hyperchaotic_transform_flat(iterations) {
        const ret = wasm.wasmetpm_hyperchaotic_transform_flat(this.__wbg_ptr, iterations);
        var v1 = getArrayI32FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 4, 4);
        return v1;
    }
    /**
     * @param {number} k
     * @param {number} n
     * @param {number} l
     * @param {string} activation_type
     */
    constructor(k, n, l, activation_type) {
        const ptr0 = passStringToWasm0(activation_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmetpm_new(k, n, l, ptr0, len0);
        if (ret[2]) {
            throw takeFromExternrefTable0(ret[1]);
        }
        this.__wbg_ptr = ret[0];
        WasmETPMFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    /**
     * @param {number} new_l
     */
    scale_synaptic_depth(new_l) {
        const ret = wasm.wasmetpm_scale_synaptic_depth(this.__wbg_ptr, new_l);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {number} tau
     * @param {string} rule
     */
    update_weights(tau, rule) {
        const ptr0 = passStringToWasm0(rule, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmetpm_update_weights(this.__wbg_ptr, tau, ptr0, len0);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
}
if (Symbol.dispose) WasmETPM.prototype[Symbol.dispose] = WasmETPM.prototype.free;

export class WasmHyperchaoticSystem {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmHyperchaoticSystemFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmhyperchaoticsystem_free(ptr, 0);
    }
    /**
     * @param {number} len
     * @returns {Float64Array}
     */
    generate_sequence(len) {
        const ret = wasm.wasmhyperchaoticsystem_generate_sequence(this.__wbg_ptr, len);
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @param {number} x_init
     * @param {number} y_init
     * @param {number} z_init
     * @param {number} w_init
     */
    constructor(x_init, y_init, z_init, w_init) {
        const ret = wasm.wasmhyperchaoticsystem_new(x_init, y_init, z_init, w_init);
        this.__wbg_ptr = ret;
        WasmHyperchaoticSystemFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
    next() {
        wasm.wasmhyperchaoticsystem_next(this.__wbg_ptr);
    }
    /**
     * @returns {number}
     */
    get w() {
        const ret = wasm.wasmhyperchaoticsystem_w(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get x() {
        const ret = wasm.wasmhyperchaoticsystem_x(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get y() {
        const ret = wasm.wasmhyperchaoticsystem_y(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get z() {
        const ret = wasm.wasmhyperchaoticsystem_z(this.__wbg_ptr);
        return ret;
    }
}
if (Symbol.dispose) WasmHyperchaoticSystem.prototype[Symbol.dispose] = WasmHyperchaoticSystem.prototype.free;

export class WasmIntegerNeuralNet {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmIntegerNeuralNetFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmintegerneuralnet_free(ptr, 0);
    }
    /**
     * @param {Int8Array} weights_flat
     * @param {Int32Array} biases
     * @param {number} out_channels
     * @param {number} in_channels
     * @param {number} scale_in
     * @param {number} scale_w
     * @param {number} scale_out
     * @param {string} act
     */
    add_layer(weights_flat, biases, out_channels, in_channels, scale_in, scale_w, scale_out, act) {
        const ptr0 = passArray8ToWasm0(weights_flat, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray32ToWasm0(biases, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(act, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmintegerneuralnet_add_layer(this.__wbg_ptr, ptr0, len0, ptr1, len1, out_channels, in_channels, scale_in, scale_w, scale_out, ptr2, len2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {Int8Array} scrambled_cipher
     * @param {Int8Array} input_key_int8
     * @param {WasmHyperchaoticSystem} hc
     * @param {number} scale_out_alice
     * @returns {Int8Array}
     */
    decrypt_scrambled(scrambled_cipher, input_key_int8, hc, scale_out_alice) {
        const ptr0 = passArray8ToWasm0(scrambled_cipher, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(input_key_int8, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        _assertClass(hc, WasmHyperchaoticSystem);
        const ret = wasm.wasmintegerneuralnet_decrypt_scrambled(this.__wbg_ptr, ptr0, len0, ptr1, len1, hc.__wbg_ptr, scale_out_alice);
        var v3 = getArrayI8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v3;
    }
    /**
     * @param {Int8Array} input
     * @returns {Int8Array}
     */
    forward(input) {
        const ptr0 = passArray8ToWasm0(input, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmintegerneuralnet_forward(this.__wbg_ptr, ptr0, len0);
        var v2 = getArrayI8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    /**
     * @param {Int8Array} input
     * @param {WasmHyperchaoticSystem} hc
     * @param {number} scale_out
     * @returns {Int8Array}
     */
    forward_scrambled(input, hc, scale_out) {
        const ptr0 = passArray8ToWasm0(input, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(hc, WasmHyperchaoticSystem);
        const ret = wasm.wasmintegerneuralnet_forward_scrambled(this.__wbg_ptr, ptr0, len0, hc.__wbg_ptr, scale_out);
        var v2 = getArrayI8FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 1, 1);
        return v2;
    }
    constructor() {
        const ret = wasm.wasmintegerneuralnet_new();
        this.__wbg_ptr = ret;
        WasmIntegerNeuralNetFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) WasmIntegerNeuralNet.prototype[Symbol.dispose] = WasmIntegerNeuralNet.prototype.free;

export class WasmKeyExchangeResult {
    static __wrap(ptr) {
        const obj = Object.create(WasmKeyExchangeResult.prototype);
        obj.__wbg_ptr = ptr;
        WasmKeyExchangeResultFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmKeyExchangeResultFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmkeyexchangeresult_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get rounds() {
        const ret = wasm.__wbg_get_wasmkeyexchangeresult_rounds(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {boolean}
     */
    get success() {
        const ret = wasm.__wbg_get_wasmkeyexchangeresult_success(this.__wbg_ptr);
        return ret !== 0;
    }
    /**
     * @returns {number}
     */
    get sync_time_ms() {
        const ret = wasm.__wbg_get_wasmkeyexchangeresult_sync_time_ms(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set rounds(arg0) {
        wasm.__wbg_set_wasmkeyexchangeresult_rounds(this.__wbg_ptr, arg0);
    }
    /**
     * @param {boolean} arg0
     */
    set success(arg0) {
        wasm.__wbg_set_wasmkeyexchangeresult_success(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set sync_time_ms(arg0) {
        wasm.__wbg_set_wasmkeyexchangeresult_sync_time_ms(this.__wbg_ptr, arg0);
    }
    /**
     * @returns {Float64Array}
     */
    extract_session_key() {
        const ret = wasm.wasmkeyexchangeresult_extract_session_key(this.__wbg_ptr);
        if (ret[3]) {
            throw takeFromExternrefTable0(ret[2]);
        }
        var v1 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v1;
    }
    /**
     * @returns {string}
     */
    get key_hex() {
        let deferred1_0;
        let deferred1_1;
        try {
            const ret = wasm.wasmkeyexchangeresult_key_hex(this.__wbg_ptr);
            deferred1_0 = ret[0];
            deferred1_1 = ret[1];
            return getStringFromWasm0(ret[0], ret[1]);
        } finally {
            wasm.__wbindgen_free(deferred1_0, deferred1_1, 1);
        }
    }
}
if (Symbol.dispose) WasmKeyExchangeResult.prototype[Symbol.dispose] = WasmKeyExchangeResult.prototype.free;

export class WasmLweSecurityMetrics {
    static __wrap(ptr) {
        const obj = Object.create(WasmLweSecurityMetrics.prototype);
        obj.__wbg_ptr = ptr;
        WasmLweSecurityMetricsFinalization.register(obj, obj.__wbg_ptr, obj);
        return obj;
    }
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmLweSecurityMetricsFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmlwesecuritymetrics_free(ptr, 0);
    }
    /**
     * @returns {number}
     */
    get classical_security_bits() {
        const ret = wasm.__wbg_get_wasmlwesecuritymetrics_classical_security_bits(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get dimension() {
        const ret = wasm.__wbg_get_wasmlwesecuritymetrics_dimension(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get error_std_dev() {
        const ret = wasm.__wbg_get_wasmlwesecuritymetrics_error_std_dev(this.__wbg_ptr);
        return ret;
    }
    /**
     * @returns {number}
     */
    get modulus() {
        const ret = wasm.__wbg_get_wasmlwesecuritymetrics_modulus(this.__wbg_ptr);
        return ret >>> 0;
    }
    /**
     * @returns {number}
     */
    get quantum_security_bits() {
        const ret = wasm.__wbg_get_wasmlwesecuritymetrics_quantum_security_bits(this.__wbg_ptr);
        return ret;
    }
    /**
     * @param {number} arg0
     */
    set classical_security_bits(arg0) {
        wasm.__wbg_set_wasmlwesecuritymetrics_classical_security_bits(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set dimension(arg0) {
        wasm.__wbg_set_wasmlwesecuritymetrics_dimension(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set error_std_dev(arg0) {
        wasm.__wbg_set_wasmlwesecuritymetrics_error_std_dev(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set modulus(arg0) {
        wasm.__wbg_set_wasmlwesecuritymetrics_modulus(this.__wbg_ptr, arg0);
    }
    /**
     * @param {number} arg0
     */
    set quantum_security_bits(arg0) {
        wasm.__wbg_set_wasmlwesecuritymetrics_quantum_security_bits(this.__wbg_ptr, arg0);
    }
}
if (Symbol.dispose) WasmLweSecurityMetrics.prototype[Symbol.dispose] = WasmLweSecurityMetrics.prototype.free;

export class WasmNeuralNet {
    __destroy_into_raw() {
        const ptr = this.__wbg_ptr;
        this.__wbg_ptr = 0;
        WasmNeuralNetFinalization.unregister(this);
        return ptr;
    }
    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_wasmneuralnet_free(ptr, 0);
    }
    /**
     * Adds a dense layer to the network.
     * Weights must be passed as a flat array of size `out_channels * in_channels` in row-major order.
     * @param {Float64Array} weights_flat
     * @param {Float64Array} biases
     * @param {number} out_channels
     * @param {number} in_channels
     * @param {string} act
     */
    add_layer(weights_flat, biases, out_channels, in_channels, act) {
        const ptr0 = passArrayF64ToWasm0(weights_flat, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF64ToWasm0(biases, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        const ptr2 = passStringToWasm0(act, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
        const len2 = WASM_VECTOR_LEN;
        const ret = wasm.wasmneuralnet_add_layer(this.__wbg_ptr, ptr0, len0, ptr1, len1, out_channels, in_channels, ptr2, len2);
        if (ret[1]) {
            throw takeFromExternrefTable0(ret[0]);
        }
    }
    /**
     * @param {Float64Array} scrambled_cipher
     * @param {Float64Array} input_key
     * @param {WasmHyperchaoticSystem} hc
     * @returns {Float64Array}
     */
    decrypt_scrambled(scrambled_cipher, input_key, hc) {
        const ptr0 = passArrayF64ToWasm0(scrambled_cipher, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArrayF64ToWasm0(input_key, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        _assertClass(hc, WasmHyperchaoticSystem);
        const ret = wasm.wasmneuralnet_decrypt_scrambled(this.__wbg_ptr, ptr0, len0, ptr1, len1, hc.__wbg_ptr);
        var v3 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v3;
    }
    /**
     * @param {Float64Array} input
     * @returns {Float64Array}
     */
    forward(input) {
        const ptr0 = passArrayF64ToWasm0(input, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ret = wasm.wasmneuralnet_forward(this.__wbg_ptr, ptr0, len0);
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    /**
     * @param {Float64Array} input
     * @param {WasmHyperchaoticSystem} hc
     * @returns {Float64Array}
     */
    forward_scrambled(input, hc) {
        const ptr0 = passArrayF64ToWasm0(input, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(hc, WasmHyperchaoticSystem);
        const ret = wasm.wasmneuralnet_forward_scrambled(this.__wbg_ptr, ptr0, len0, hc.__wbg_ptr);
        var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
        wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
        return v2;
    }
    constructor() {
        const ret = wasm.wasmneuralnet_new();
        this.__wbg_ptr = ret;
        WasmNeuralNetFinalization.register(this, this.__wbg_ptr, this);
        return this;
    }
}
if (Symbol.dispose) WasmNeuralNet.prototype[Symbol.dispose] = WasmNeuralNet.prototype.free;

/**
 * @param {number} dimension
 * @param {number} modulus
 * @param {number} error_std_dev
 * @returns {WasmLweSecurityMetrics}
 */
export function estimate_wasm_lwe_security(dimension, modulus, error_std_dev) {
    const ret = wasm.estimate_wasm_lwe_security(dimension, modulus, error_std_dev);
    return WasmLweSecurityMetrics.__wrap(ret);
}

/**
 * Runs a full Alice-Bob key exchange simulation from the browser.
 * @param {number} k
 * @param {number} n
 * @param {number} l
 * @param {number} max_rounds
 * @param {string} update_rule
 * @param {string} activation_type
 * @param {boolean} adaptive_l_scaling
 * @param {number} active_query_threshold
 * @param {number} physical_channel_correlation
 * @returns {WasmKeyExchangeResult}
 */
export function run_wasm_key_exchange(k, n, l, max_rounds, update_rule, activation_type, adaptive_l_scaling, active_query_threshold, physical_channel_correlation) {
    const ptr0 = passStringToWasm0(update_rule, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len0 = WASM_VECTOR_LEN;
    const ptr1 = passStringToWasm0(activation_type, wasm.__wbindgen_malloc, wasm.__wbindgen_realloc);
    const len1 = WASM_VECTOR_LEN;
    const ret = wasm.run_wasm_key_exchange(k, n, l, max_rounds, ptr0, len0, ptr1, len1, adaptive_l_scaling, active_query_threshold, physical_channel_correlation);
    if (ret[2]) {
        throw takeFromExternrefTable0(ret[1]);
    }
    return WasmKeyExchangeResult.__wrap(ret[0]);
}

/**
 * @param {number} q
 * @param {number} scale
 * @returns {number}
 */
export function wasm_dequantize(q, scale) {
    const ret = wasm.wasm_dequantize(q, scale);
    return ret;
}

/**
 * @param {Float64Array} data
 * @returns {Float64Array}
 */
export function wasm_hamming_decode(data) {
    const ptr0 = passArrayF64ToWasm0(data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.wasm_hamming_decode(ptr0, len0);
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * @param {Float64Array} data
 * @returns {Float64Array}
 */
export function wasm_hamming_encode(data) {
    const ptr0 = passArrayF64ToWasm0(data, wasm.__wbindgen_malloc);
    const len0 = WASM_VECTOR_LEN;
    const ret = wasm.wasm_hamming_encode(ptr0, len0);
    var v2 = getArrayF64FromWasm0(ret[0], ret[1]).slice();
    wasm.__wbindgen_free(ret[0], ret[1] * 8, 8);
    return v2;
}

/**
 * @param {number} x
 * @param {number} scale
 * @returns {number}
 */
export function wasm_quantize(x, scale) {
    const ret = wasm.wasm_quantize(x, scale);
    return ret;
}
function __wbg_get_imports() {
    const import0 = {
        __proto__: null,
        __wbg___wbindgen_is_function_acc5528be2b923f2: function(arg0) {
            const ret = typeof(arg0) === 'function';
            return ret;
        },
        __wbg___wbindgen_is_object_0beba4a1980d3eea: function(arg0) {
            const val = arg0;
            const ret = typeof(val) === 'object' && val !== null;
            return ret;
        },
        __wbg___wbindgen_is_string_1fca8072260dd261: function(arg0) {
            const ret = typeof(arg0) === 'string';
            return ret;
        },
        __wbg___wbindgen_is_undefined_721f8decd50c87a3: function(arg0) {
            const ret = arg0 === undefined;
            return ret;
        },
        __wbg___wbindgen_throw_ea4887a5f8f9a9db: function(arg0, arg1) {
            throw new Error(getStringFromWasm0(arg0, arg1));
        },
        __wbg_call_5575218572ead796: function() { return handleError(function (arg0, arg1, arg2) {
            const ret = arg0.call(arg1, arg2);
            return ret;
        }, arguments); },
        __wbg_crypto_38df2bab126b63dc: function(arg0) {
            const ret = arg0.crypto;
            return ret;
        },
        __wbg_getRandomValues_c44a50d8cfdaebeb: function() { return handleError(function (arg0, arg1) {
            arg0.getRandomValues(arg1);
        }, arguments); },
        __wbg_length_589238bdcf171f0e: function(arg0) {
            const ret = arg0.length;
            return ret;
        },
        __wbg_msCrypto_bd5a034af96bcba6: function(arg0) {
            const ret = arg0.msCrypto;
            return ret;
        },
        __wbg_new_with_length_9b650f44b5c44a4e: function(arg0) {
            const ret = new Uint8Array(arg0 >>> 0);
            return ret;
        },
        __wbg_node_84ea875411254db1: function(arg0) {
            const ret = arg0.node;
            return ret;
        },
        __wbg_process_44c7a14e11e9f69e: function(arg0) {
            const ret = arg0.process;
            return ret;
        },
        __wbg_prototypesetcall_d721637c7ca66eb8: function(arg0, arg1, arg2) {
            Uint8Array.prototype.set.call(getArrayU8FromWasm0(arg0, arg1), arg2);
        },
        __wbg_randomFillSync_6c25eac9869eb53c: function() { return handleError(function (arg0, arg1) {
            arg0.randomFillSync(arg1);
        }, arguments); },
        __wbg_require_b4edbdcf3e2a1ef0: function() { return handleError(function () {
            const ret = module.require;
            return ret;
        }, arguments); },
        __wbg_static_accessor_GLOBAL_THIS_2fee5048bcca5938: function() {
            const ret = typeof globalThis === 'undefined' ? null : globalThis;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_GLOBAL_ce44e66a4935da8c: function() {
            const ret = typeof global === 'undefined' ? null : global;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_SELF_44f6e0cb5e67cdad: function() {
            const ret = typeof self === 'undefined' ? null : self;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_static_accessor_WINDOW_168f178805d978fe: function() {
            const ret = typeof window === 'undefined' ? null : window;
            return isLikeNone(ret) ? 0 : addToExternrefTable0(ret);
        },
        __wbg_subarray_b0e8ac4ed313fea8: function(arg0, arg1, arg2) {
            const ret = arg0.subarray(arg1 >>> 0, arg2 >>> 0);
            return ret;
        },
        __wbg_versions_276b2795b1c6a219: function(arg0) {
            const ret = arg0.versions;
            return ret;
        },
        __wbindgen_cast_0000000000000001: function(arg0, arg1) {
            // Cast intrinsic for `Ref(Slice(U8)) -> NamedExternref("Uint8Array")`.
            const ret = getArrayU8FromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_cast_0000000000000002: function(arg0, arg1) {
            // Cast intrinsic for `Ref(String) -> Externref`.
            const ret = getStringFromWasm0(arg0, arg1);
            return ret;
        },
        __wbindgen_init_externref_table: function() {
            const table = wasm.__wbindgen_externrefs;
            const offset = table.grow(4);
            table.set(0, undefined);
            table.set(offset + 0, undefined);
            table.set(offset + 1, null);
            table.set(offset + 2, true);
            table.set(offset + 3, false);
        },
    };
    return {
        __proto__: null,
        "./deep_enigma_bg.js": import0,
    };
}

const WasmETPMFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmetpm_free(ptr, 1));
const WasmHyperchaoticSystemFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmhyperchaoticsystem_free(ptr, 1));
const WasmIntegerNeuralNetFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmintegerneuralnet_free(ptr, 1));
const WasmKeyExchangeResultFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmkeyexchangeresult_free(ptr, 1));
const WasmLweSecurityMetricsFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmlwesecuritymetrics_free(ptr, 1));
const WasmNeuralNetFinalization = (typeof FinalizationRegistry === 'undefined')
    ? { register: () => {}, unregister: () => {} }
    : new FinalizationRegistry(ptr => wasm.__wbg_wasmneuralnet_free(ptr, 1));

function addToExternrefTable0(obj) {
    const idx = wasm.__externref_table_alloc();
    wasm.__wbindgen_externrefs.set(idx, obj);
    return idx;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
}

function getArrayF64FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getFloat64ArrayMemory0().subarray(ptr / 8, ptr / 8 + len);
}

function getArrayI32FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt32ArrayMemory0().subarray(ptr / 4, ptr / 4 + len);
}

function getArrayI8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getInt8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

function getArrayU8FromWasm0(ptr, len) {
    ptr = ptr >>> 0;
    return getUint8ArrayMemory0().subarray(ptr / 1, ptr / 1 + len);
}

let cachedFloat64ArrayMemory0 = null;
function getFloat64ArrayMemory0() {
    if (cachedFloat64ArrayMemory0 === null || cachedFloat64ArrayMemory0.byteLength === 0) {
        cachedFloat64ArrayMemory0 = new Float64Array(wasm.memory.buffer);
    }
    return cachedFloat64ArrayMemory0;
}

let cachedInt32ArrayMemory0 = null;
function getInt32ArrayMemory0() {
    if (cachedInt32ArrayMemory0 === null || cachedInt32ArrayMemory0.byteLength === 0) {
        cachedInt32ArrayMemory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32ArrayMemory0;
}

let cachedInt8ArrayMemory0 = null;
function getInt8ArrayMemory0() {
    if (cachedInt8ArrayMemory0 === null || cachedInt8ArrayMemory0.byteLength === 0) {
        cachedInt8ArrayMemory0 = new Int8Array(wasm.memory.buffer);
    }
    return cachedInt8ArrayMemory0;
}

function getStringFromWasm0(ptr, len) {
    return decodeText(ptr >>> 0, len);
}

let cachedUint32ArrayMemory0 = null;
function getUint32ArrayMemory0() {
    if (cachedUint32ArrayMemory0 === null || cachedUint32ArrayMemory0.byteLength === 0) {
        cachedUint32ArrayMemory0 = new Uint32Array(wasm.memory.buffer);
    }
    return cachedUint32ArrayMemory0;
}

let cachedUint8ArrayMemory0 = null;
function getUint8ArrayMemory0() {
    if (cachedUint8ArrayMemory0 === null || cachedUint8ArrayMemory0.byteLength === 0) {
        cachedUint8ArrayMemory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8ArrayMemory0;
}

function handleError(f, args) {
    try {
        return f.apply(this, args);
    } catch (e) {
        const idx = addToExternrefTable0(e);
        wasm.__wbindgen_exn_store(idx);
    }
}

function isLikeNone(x) {
    return x === undefined || x === null;
}

function passArray32ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 4, 4) >>> 0;
    getUint32ArrayMemory0().set(arg, ptr / 4);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1, 1) >>> 0;
    getUint8ArrayMemory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passArrayF64ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 8, 8) >>> 0;
    getFloat64ArrayMemory0().set(arg, ptr / 8);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function passStringToWasm0(arg, malloc, realloc) {
    if (realloc === undefined) {
        const buf = cachedTextEncoder.encode(arg);
        const ptr = malloc(buf.length, 1) >>> 0;
        getUint8ArrayMemory0().subarray(ptr, ptr + buf.length).set(buf);
        WASM_VECTOR_LEN = buf.length;
        return ptr;
    }

    let len = arg.length;
    let ptr = malloc(len, 1) >>> 0;

    const mem = getUint8ArrayMemory0();

    let offset = 0;

    for (; offset < len; offset++) {
        const code = arg.charCodeAt(offset);
        if (code > 0x7F) break;
        mem[ptr + offset] = code;
    }
    if (offset !== len) {
        if (offset !== 0) {
            arg = arg.slice(offset);
        }
        ptr = realloc(ptr, len, len = offset + arg.length * 3, 1) >>> 0;
        const view = getUint8ArrayMemory0().subarray(ptr + offset, ptr + len);
        const ret = cachedTextEncoder.encodeInto(arg, view);

        offset += ret.written;
        ptr = realloc(ptr, len, offset, 1) >>> 0;
    }

    WASM_VECTOR_LEN = offset;
    return ptr;
}

function takeFromExternrefTable0(idx) {
    const value = wasm.__wbindgen_externrefs.get(idx);
    wasm.__externref_table_dealloc(idx);
    return value;
}

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
cachedTextDecoder.decode();
const MAX_SAFARI_DECODE_BYTES = 2146435072;
let numBytesDecoded = 0;
function decodeText(ptr, len) {
    numBytesDecoded += len;
    if (numBytesDecoded >= MAX_SAFARI_DECODE_BYTES) {
        cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });
        cachedTextDecoder.decode();
        numBytesDecoded = len;
    }
    return cachedTextDecoder.decode(getUint8ArrayMemory0().subarray(ptr, ptr + len));
}

const cachedTextEncoder = new TextEncoder();

if (!('encodeInto' in cachedTextEncoder)) {
    cachedTextEncoder.encodeInto = function (arg, view) {
        const buf = cachedTextEncoder.encode(arg);
        view.set(buf);
        return {
            read: arg.length,
            written: buf.length
        };
    };
}

let WASM_VECTOR_LEN = 0;

let wasmModule, wasmInstance, wasm;
function __wbg_finalize_init(instance, module) {
    wasmInstance = instance;
    wasm = instance.exports;
    wasmModule = module;
    cachedFloat64ArrayMemory0 = null;
    cachedInt32ArrayMemory0 = null;
    cachedInt8ArrayMemory0 = null;
    cachedUint32ArrayMemory0 = null;
    cachedUint8ArrayMemory0 = null;
    wasm.__wbindgen_start();
    return wasm;
}

async function __wbg_load(module, imports) {
    if (typeof Response === 'function' && module instanceof Response) {
        if (typeof WebAssembly.instantiateStreaming === 'function') {
            try {
                return await WebAssembly.instantiateStreaming(module, imports);
            } catch (e) {
                const validResponse = module.ok && expectedResponseType(module.type);

                if (validResponse && module.headers.get('Content-Type') !== 'application/wasm') {
                    console.warn("`WebAssembly.instantiateStreaming` failed because your server does not serve Wasm with `application/wasm` MIME type. Falling back to `WebAssembly.instantiate` which is slower. Original error:\n", e);

                } else { throw e; }
            }
        }

        const bytes = await module.arrayBuffer();
        return await WebAssembly.instantiate(bytes, imports);
    } else {
        const instance = await WebAssembly.instantiate(module, imports);

        if (instance instanceof WebAssembly.Instance) {
            return { instance, module };
        } else {
            return instance;
        }
    }

    function expectedResponseType(type) {
        switch (type) {
            case 'basic': case 'cors': case 'default': return true;
        }
        return false;
    }
}

function initSync(module) {
    if (wasm !== undefined) return wasm;


    if (module !== undefined) {
        if (Object.getPrototypeOf(module) === Object.prototype) {
            ({module} = module)
        } else {
            console.warn('using deprecated parameters for `initSync()`; pass a single object instead')
        }
    }

    const imports = __wbg_get_imports();
    if (!(module instanceof WebAssembly.Module)) {
        module = new WebAssembly.Module(module);
    }
    const instance = new WebAssembly.Instance(module, imports);
    return __wbg_finalize_init(instance, module);
}

async function __wbg_init(module_or_path) {
    if (wasm !== undefined) return wasm;


    if (module_or_path !== undefined) {
        if (Object.getPrototypeOf(module_or_path) === Object.prototype) {
            ({module_or_path} = module_or_path)
        } else {
            console.warn('using deprecated parameters for the initialization function; pass a single object instead')
        }
    }

    if (module_or_path === undefined) {
        module_or_path = new URL('deep_enigma_bg.wasm', import.meta.url);
    }
    const imports = __wbg_get_imports();

    if (typeof module_or_path === 'string' || (typeof Request === 'function' && module_or_path instanceof Request) || (typeof URL === 'function' && module_or_path instanceof URL)) {
        module_or_path = fetch(module_or_path);
    }

    const { instance, module } = await __wbg_load(await module_or_path, imports);

    return __wbg_finalize_init(instance, module);
}

export { initSync, __wbg_init as default };
