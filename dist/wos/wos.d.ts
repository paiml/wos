/* tslint:disable */
/* eslint-disable */

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

/**
 * Get the default UX layout configuration as JSON string
 *
 * # Examples
 *
 * ```
 * let config_json = wos::get_default_config();
 * assert!(!config_json.is_empty());
 * assert!(config_json.contains("version"));
 * // Should be valid JSON
 * let parsed: serde_json::Value = serde_json::from_str(&config_json).unwrap();
 * assert!(parsed.is_object());
 * ```
 */
export function getDefaultConfig(): string;

/**
 * Initialize WOS pure WASM mode
 *
 * This is the main entry point - automatically called when the WASM module loads.
 * Zero JavaScript: all DOM manipulation is done through web-sys bindings.
 */
export function init_pure_wasm(): void;

/**
 * Load UX layout configuration from YAML string
 *
 * Returns the config as JSON string on success, or error message on failure
 *
 * # Examples
 *
 * ```
 * // Invalid YAML returns an error
 * let invalid_yaml = "{ malformed yaml [[[";
 * let result = wos::load_config_from_yaml(invalid_yaml);
 * assert!(result.is_err());
 * ```
 */
export function loadConfigFromYaml(yaml: string): string;

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
 *
 * # Examples
 *
 * ```
 * // Invalid configuration returns an error
 * let invalid_yaml = "version: 'bad version format'";
 * assert!(wos::validate_config(invalid_yaml).is_err());
 * ```
 */
export function validateConfig(yaml: string): void;

/**
 * Get WOS version
 *
 * # Examples
 *
 * ```
 * let version = wos::wos_version();
 * assert!(version.starts_with("WOS v"));
 * assert!(version.contains("kernel:"));
 * assert!(version.contains("userspace:"));
 * ```
 */
export function wos_version(): string;

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_woswasm_free: (a: number, b: number) => void;
  readonly getDefaultConfig: () => [number, number];
  readonly loadConfigFromYaml: (a: number, b: number) => [number, number, number, number];
  readonly loadConfigFromYamlWithFallback: (a: number, b: number) => [number, number];
  readonly validateConfig: (a: number, b: number) => [number, number];
  readonly wos_version: () => [number, number];
  readonly woswasm_executeCommand: (a: number, b: number, c: number) => [number, number];
  readonly woswasm_executeSyscall: (a: number, b: number, c: number, d: number) => [number, number, number, number];
  readonly woswasm_exportQualityHtml: (a: number) => [number, number];
  readonly woswasm_exportQualityMarkdown: (a: number) => [number, number];
  readonly woswasm_exportQualitySarif: (a: number) => [number, number];
  readonly woswasm_getCurrentState: (a: number) => [number, number];
  readonly woswasm_getCurrentUser: (a: number) => [number, number];
  readonly woswasm_getCurrentWorkingDirectory: (a: number) => [number, number];
  readonly woswasm_getKernelHistory: (a: number) => [number, number];
  readonly woswasm_getQualityMetrics: (a: number) => [number, number, number, number];
  readonly woswasm_getState: (a: number) => [number, number, number, number];
  readonly woswasm_jumpToPosition: (a: number, b: number) => [number, number];
  readonly woswasm_new: () => number;
  readonly woswasm_processCount: (a: number) => number;
  readonly woswasm_reset: (a: number) => void;
  readonly woswasm_setState: (a: number, b: number, c: number) => [number, number];
  readonly init_pure_wasm: () => void;
  readonly wasm_bindgen__convert__closures_____invoke__h7d928a9f858445a7: (a: number, b: number, c: any) => void;
  readonly wasm_bindgen__closure__destroy__hcf43f0be3d966007: (a: number, b: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_exn_store: (a: number) => void;
  readonly __externref_table_alloc: () => number;
  readonly __wbindgen_externrefs: WebAssembly.Table;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __externref_table_dealloc: (a: number) => void;
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
