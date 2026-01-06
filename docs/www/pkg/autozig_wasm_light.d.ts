/* tslint:disable */
/* eslint-disable */

/**
 * 获取版本信息
 */
export function get_version(): string;

/**
 * 初始化函数
 */
export function init(): void;

/**
 * 分配底图缓冲区（零拷贝）
 */
export function wasm_alloc_background_buffer(width: number, height: number): number;

/**
 * 分配光源缓冲区（零拷贝）
 */
export function wasm_alloc_lights_buffer(count: number): number;

/**
 * 分配像素缓冲区（零拷贝）
 */
export function wasm_alloc_pixel_buffer(width: number, height: number): number;

/**
 * Zig Scalar 标量渲染（对比基准）
 */
export function wasm_render_lights_scalar(pixel_ptr: number, width: number, height: number, lights_ptr: number, num_lights: number): void;

/**
 * Zig SIMD 向量化渲染
 */
export function wasm_render_lights_simd(pixel_ptr: number, width: number, height: number, lights_ptr: number, num_lights: number): void;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly get_version: () => [number, number];
  readonly init: () => void;
  readonly wasm_alloc_background_buffer: (a: number, b: number) => number;
  readonly wasm_alloc_lights_buffer: (a: number) => number;
  readonly wasm_alloc_pixel_buffer: (a: number, b: number) => number;
  readonly wasm_render_lights_scalar: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly wasm_render_lights_simd: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly __wbindgen_externrefs: WebAssembly.Table;
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
