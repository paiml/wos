/* tslint:disable */
/* eslint-disable */
/**
 * Load UX layout configuration from YAML with fallback to default
 *
 * Never fails - returns default config if YAML is invalid.
 * Returns the config as JSON string.
 */
export function loadConfigFromYamlWithFallback(yaml: string): string;
/**
 * Validate a UX layout configuration YAML string
 *
 * Returns Ok(()) if valid, Err(message) if invalid
 */
export function validateConfig(yaml: string): void;
/**
 * Get the default UX layout configuration as JSON string
 */
export function getDefaultConfig(): string;
/**
 * Load UX layout configuration from YAML string
 *
 * Returns the config as JSON string on success, or error message on failure
 */
export function loadConfigFromYaml(yaml: string): string;
/**
 * Get WOS version
 */
export function wos_version(): string;
/**
 * WASM-bindgen wrapper for WOS kernel
 */
export class WosWasm {
  free(): void;
  [Symbol.dispose](): void;
  /**
   * Get number of processes
   */
  processCount(): number;
  /**
   * Execute a command string (shell-like interface)
   *
   * Parses a command and executes it, returning the output.
   * Supports pipelines and command chaining with |, &&, ||, ;
   * Supports variable assignment (VAR=value) and expansion ($VAR)
   */
  executeCommand(command: string): string;
  /**
   * Execute a syscall and return the output as JSON
   *
   * Takes a syscall as JSON string, executes it, and returns the output as JSON
   */
  executeSyscall(syscall_json: string, calling_pid: number): string;
  /**
   * Get current user
   */
  getCurrentUser(): string;
  /**
   * WOS-302: Jump to specific position in kernel history
   *
   * Restores kernel state to the specified history position
   */
  jumpToPosition(position: number): void;
  /**
   * WOS-302: Get current kernel state as JSON for state inspector
   *
   * Returns full kernel state including processes, memory, filesystem
   */
  getCurrentState(): string;
  /**
   * WOS-302: Get kernel history as JSON for time-travel debugger
   *
   * Returns array of SystemCallTrace entries with timestamps
   */
  getKernelHistory(): string;
  /**
   * Export quality report as HTML
   */
  exportQualityHtml(): string;
  /**
   * Get quality metrics as JSON
   */
  getQualityMetrics(): string;
  /**
   * Export quality report as SARIF
   */
  exportQualitySarif(): string;
  /**
   * Export quality report as Markdown
   */
  exportQualityMarkdown(): string;
  /**
   * Get current working directory
   */
  getCurrentWorkingDirectory(): string;
  /**
   * Create a new WOS instance
   */
  constructor();
  /**
   * Reset to initial state
   */
  reset(): void;
  /**
   * Get current kernel state as JSON
   */
  getState(): string;
  /**
   * Set kernel state from JSON
   */
  setState(state_json: string): void;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_woswasm_free: (a: number, b: number) => void;
  readonly getDefaultConfig: (a: number) => void;
  readonly loadConfigFromYaml: (a: number, b: number, c: number) => void;
  readonly loadConfigFromYamlWithFallback: (a: number, b: number, c: number) => void;
  readonly validateConfig: (a: number, b: number, c: number) => void;
  readonly wos_version: (a: number) => void;
  readonly woswasm_executeCommand: (a: number, b: number, c: number, d: number) => void;
  readonly woswasm_executeSyscall: (a: number, b: number, c: number, d: number, e: number) => void;
  readonly woswasm_exportQualityHtml: (a: number, b: number) => void;
  readonly woswasm_exportQualityMarkdown: (a: number, b: number) => void;
  readonly woswasm_exportQualitySarif: (a: number, b: number) => void;
  readonly woswasm_getCurrentState: (a: number, b: number) => void;
  readonly woswasm_getCurrentUser: (a: number, b: number) => void;
  readonly woswasm_getCurrentWorkingDirectory: (a: number, b: number) => void;
  readonly woswasm_getKernelHistory: (a: number, b: number) => void;
  readonly woswasm_getQualityMetrics: (a: number, b: number) => void;
  readonly woswasm_getState: (a: number, b: number) => void;
  readonly woswasm_jumpToPosition: (a: number, b: number, c: number) => void;
  readonly woswasm_new: () => number;
  readonly woswasm_processCount: (a: number) => number;
  readonly woswasm_reset: (a: number) => void;
  readonly woswasm_setState: (a: number, b: number, c: number, d: number) => void;
  readonly __wbindgen_add_to_stack_pointer: (a: number) => number;
  readonly __wbindgen_export_0: (a: number, b: number) => number;
  readonly __wbindgen_export_1: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_2: (a: number, b: number, c: number) => void;
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
