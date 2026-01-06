/* tslint:disable */
/* eslint-disable */
export const memory: WebAssembly.Memory;
export const get_version: () => [number, number];
export const init: () => void;
export const wasm_alloc_background_buffer: (a: number, b: number) => number;
export const wasm_alloc_lights_buffer: (a: number) => number;
export const wasm_alloc_pixel_buffer: (a: number, b: number) => number;
export const wasm_render_lights_scalar: (a: number, b: number, c: number, d: number, e: number) => void;
export const wasm_render_lights_simd: (a: number, b: number, c: number, d: number, e: number) => void;
export const __wbindgen_externrefs: WebAssembly.Table;
export const __wbindgen_free: (a: number, b: number, c: number) => void;
export const __wbindgen_start: () => void;
