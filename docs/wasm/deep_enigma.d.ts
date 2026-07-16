/* tslint:disable */
/* eslint-disable */

/**
 * A wrapper to initialize ETPM in JS and run custom steps.
 */
export class WasmETPM {
    free(): void;
    [Symbol.dispose](): void;
    calculate_local_fields(inputs_flat: Int32Array): Int32Array;
    calculate_output(inputs_flat: Int32Array): number;
    chaotic_transform_flat(iterations: number): Int32Array;
    get_k(): number;
    get_l(): number;
    get_n(): number;
    get_weights_flat(): Int32Array;
    hyperchaotic_transform_flat(iterations: number): Int32Array;
    constructor(k: number, n: number, l: number, activation_type: string);
    scale_synaptic_depth(new_l: number): void;
    update_weights(tau: number, rule: string): void;
}

export class WasmHyperchaoticSystem {
    free(): void;
    [Symbol.dispose](): void;
    generate_sequence(len: number): Float64Array;
    constructor(x_init: number, y_init: number, z_init: number, w_init: number);
    next(): void;
    readonly w: number;
    readonly x: number;
    readonly y: number;
    readonly z: number;
}

export class WasmIntegerNeuralNet {
    free(): void;
    [Symbol.dispose](): void;
    add_layer(weights_flat: Int8Array, biases: Int32Array, out_channels: number, in_channels: number, scale_in: number, scale_w: number, scale_out: number, act: string): void;
    decrypt_scrambled(scrambled_cipher: Int8Array, input_key_int8: Int8Array, hc: WasmHyperchaoticSystem, scale_out_alice: number): Int8Array;
    forward(input: Int8Array): Int8Array;
    forward_scrambled(input: Int8Array, hc: WasmHyperchaoticSystem, scale_out: number): Int8Array;
    constructor();
}

export class WasmKeyExchangeResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    extract_session_key(): Float64Array;
    rounds: number;
    success: boolean;
    sync_time_ms: number;
    readonly key_hex: string;
}

export class WasmLweSecurityMetrics {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    classical_security_bits: number;
    dimension: number;
    error_std_dev: number;
    modulus: number;
    quantum_security_bits: number;
}

export class WasmNeuralNet {
    free(): void;
    [Symbol.dispose](): void;
    /**
     * Adds a dense layer to the network.
     * Weights must be passed as a flat array of size `out_channels * in_channels` in row-major order.
     */
    add_layer(weights_flat: Float64Array, biases: Float64Array, out_channels: number, in_channels: number, act: string): void;
    decrypt_scrambled(scrambled_cipher: Float64Array, input_key: Float64Array, hc: WasmHyperchaoticSystem): Float64Array;
    forward(input: Float64Array): Float64Array;
    forward_scrambled(input: Float64Array, hc: WasmHyperchaoticSystem): Float64Array;
    constructor();
}

export function estimate_wasm_lwe_security(dimension: number, modulus: number, error_std_dev: number): WasmLweSecurityMetrics;

/**
 * Runs a full Alice-Bob key exchange simulation from the browser.
 */
export function run_wasm_key_exchange(k: number, n: number, l: number, max_rounds: number, update_rule: string, activation_type: string, adaptive_l_scaling: boolean, active_query_threshold: number, physical_channel_correlation: number): WasmKeyExchangeResult;

export function wasm_dequantize(q: number, scale: number): number;

export function wasm_hamming_decode(data: Float64Array): Float64Array;

export function wasm_hamming_encode(data: Float64Array): Float64Array;

export function wasm_quantize(x: number, scale: number): number;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_wasmkeyexchangeresult_rounds: (a: number) => number;
    readonly __wbg_get_wasmkeyexchangeresult_success: (a: number) => number;
    readonly __wbg_get_wasmkeyexchangeresult_sync_time_ms: (a: number) => number;
    readonly __wbg_get_wasmlwesecuritymetrics_classical_security_bits: (a: number) => number;
    readonly __wbg_get_wasmlwesecuritymetrics_dimension: (a: number) => number;
    readonly __wbg_get_wasmlwesecuritymetrics_modulus: (a: number) => number;
    readonly __wbg_get_wasmlwesecuritymetrics_quantum_security_bits: (a: number) => number;
    readonly __wbg_set_wasmkeyexchangeresult_rounds: (a: number, b: number) => void;
    readonly __wbg_set_wasmkeyexchangeresult_success: (a: number, b: number) => void;
    readonly __wbg_set_wasmkeyexchangeresult_sync_time_ms: (a: number, b: number) => void;
    readonly __wbg_set_wasmlwesecuritymetrics_classical_security_bits: (a: number, b: number) => void;
    readonly __wbg_set_wasmlwesecuritymetrics_dimension: (a: number, b: number) => void;
    readonly __wbg_set_wasmlwesecuritymetrics_modulus: (a: number, b: number) => void;
    readonly __wbg_set_wasmlwesecuritymetrics_quantum_security_bits: (a: number, b: number) => void;
    readonly __wbg_wasmetpm_free: (a: number, b: number) => void;
    readonly __wbg_wasmhyperchaoticsystem_free: (a: number, b: number) => void;
    readonly __wbg_wasmintegerneuralnet_free: (a: number, b: number) => void;
    readonly __wbg_wasmkeyexchangeresult_free: (a: number, b: number) => void;
    readonly __wbg_wasmlwesecuritymetrics_free: (a: number, b: number) => void;
    readonly __wbg_wasmneuralnet_free: (a: number, b: number) => void;
    readonly estimate_wasm_lwe_security: (a: number, b: number, c: number) => number;
    readonly run_wasm_key_exchange: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number) => [number, number, number];
    readonly wasm_dequantize: (a: number, b: number) => number;
    readonly wasm_hamming_decode: (a: number, b: number) => [number, number];
    readonly wasm_hamming_encode: (a: number, b: number) => [number, number];
    readonly wasm_quantize: (a: number, b: number) => number;
    readonly wasmetpm_calculate_local_fields: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmetpm_calculate_output: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmetpm_chaotic_transform_flat: (a: number, b: number) => [number, number];
    readonly wasmetpm_get_k: (a: number) => number;
    readonly wasmetpm_get_l: (a: number) => number;
    readonly wasmetpm_get_n: (a: number) => number;
    readonly wasmetpm_get_weights_flat: (a: number) => [number, number];
    readonly wasmetpm_hyperchaotic_transform_flat: (a: number, b: number) => [number, number];
    readonly wasmetpm_new: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly wasmetpm_scale_synaptic_depth: (a: number, b: number) => [number, number];
    readonly wasmetpm_update_weights: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmhyperchaoticsystem_generate_sequence: (a: number, b: number) => [number, number];
    readonly wasmhyperchaoticsystem_new: (a: number, b: number, c: number, d: number) => number;
    readonly wasmhyperchaoticsystem_next: (a: number) => void;
    readonly wasmhyperchaoticsystem_w: (a: number) => number;
    readonly wasmhyperchaoticsystem_x: (a: number) => number;
    readonly wasmhyperchaoticsystem_y: (a: number) => number;
    readonly wasmhyperchaoticsystem_z: (a: number) => number;
    readonly wasmintegerneuralnet_add_layer: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number, k: number, l: number) => [number, number];
    readonly wasmintegerneuralnet_decrypt_scrambled: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
    readonly wasmintegerneuralnet_forward: (a: number, b: number, c: number) => [number, number];
    readonly wasmintegerneuralnet_forward_scrambled: (a: number, b: number, c: number, d: number, e: number) => [number, number];
    readonly wasmintegerneuralnet_new: () => number;
    readonly wasmkeyexchangeresult_extract_session_key: (a: number) => [number, number, number, number];
    readonly wasmkeyexchangeresult_key_hex: (a: number) => [number, number];
    readonly wasmneuralnet_add_layer: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
    readonly wasmneuralnet_decrypt_scrambled: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
    readonly wasmneuralnet_forward: (a: number, b: number, c: number) => [number, number];
    readonly wasmneuralnet_forward_scrambled: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmneuralnet_new: () => number;
    readonly __wbg_set_wasmlwesecuritymetrics_error_std_dev: (a: number, b: number) => void;
    readonly __wbg_get_wasmlwesecuritymetrics_error_std_dev: (a: number) => number;
    readonly __wbindgen_exn_store: (a: number) => void;
    readonly __externref_table_alloc: () => number;
    readonly __wbindgen_externrefs: WebAssembly.Table;
    readonly __wbindgen_malloc: (a: number, b: number) => number;
    readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
    readonly __externref_table_dealloc: (a: number) => void;
    readonly __wbindgen_free: (a: number, b: number, c: number) => void;
    readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;

/**
 * Instantiates the given `module`, which can either be bytes or
 * a precompiled `WebAssembly.Module`.
 *
 * @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
 *
 * @returns {InitOutput}
 */
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
 * If `module_or_path` is {RequestInfo} or {URL}, makes a request and
 * for everything else, calls `WebAssembly.instantiate` directly.
 *
 * @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
 *
 * @returns {Promise<InitOutput>}
 */
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
