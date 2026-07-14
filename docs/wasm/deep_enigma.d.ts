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
    constructor(k: number, n: number, l: number, activation_type: string);
    scale_synaptic_depth(new_l: number): void;
    update_weights(tau: number, rule: string): void;
}

export class WasmKeyExchangeResult {
    private constructor();
    free(): void;
    [Symbol.dispose](): void;
    rounds: number;
    success: boolean;
    sync_time_ms: number;
    readonly key_hex: string;
}

/**
 * Runs a full Alice-Bob key exchange simulation from the browser.
 */
export function run_wasm_key_exchange(k: number, n: number, l: number, max_rounds: number, update_rule: string, activation_type: string, adaptive_l_scaling: boolean, active_query_threshold: number): WasmKeyExchangeResult;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
    readonly memory: WebAssembly.Memory;
    readonly __wbg_get_wasmkeyexchangeresult_rounds: (a: number) => number;
    readonly __wbg_get_wasmkeyexchangeresult_success: (a: number) => number;
    readonly __wbg_get_wasmkeyexchangeresult_sync_time_ms: (a: number) => number;
    readonly __wbg_set_wasmkeyexchangeresult_rounds: (a: number, b: number) => void;
    readonly __wbg_set_wasmkeyexchangeresult_success: (a: number, b: number) => void;
    readonly __wbg_set_wasmkeyexchangeresult_sync_time_ms: (a: number, b: number) => void;
    readonly __wbg_wasmetpm_free: (a: number, b: number) => void;
    readonly __wbg_wasmkeyexchangeresult_free: (a: number, b: number) => void;
    readonly run_wasm_key_exchange: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number, j: number) => [number, number, number];
    readonly wasmetpm_calculate_local_fields: (a: number, b: number, c: number) => [number, number, number, number];
    readonly wasmetpm_calculate_output: (a: number, b: number, c: number) => [number, number, number];
    readonly wasmetpm_chaotic_transform_flat: (a: number, b: number) => [number, number];
    readonly wasmetpm_get_k: (a: number) => number;
    readonly wasmetpm_get_l: (a: number) => number;
    readonly wasmetpm_get_n: (a: number) => number;
    readonly wasmetpm_get_weights_flat: (a: number) => [number, number];
    readonly wasmetpm_new: (a: number, b: number, c: number, d: number, e: number) => [number, number, number];
    readonly wasmetpm_scale_synaptic_depth: (a: number, b: number) => [number, number];
    readonly wasmetpm_update_weights: (a: number, b: number, c: number, d: number) => [number, number];
    readonly wasmkeyexchangeresult_key_hex: (a: number) => [number, number];
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
