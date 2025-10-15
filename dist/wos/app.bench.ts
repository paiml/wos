/**
 * Performance Benchmarks for WOS Frontend
 *
 * These benchmarks measure the performance of critical frontend operations.
 * Run with: deno bench app.bench.ts --allow-net
 */

import { DOMParser } from "https://deno.land/x/deno_dom@v0.1.43/deno-dom-wasm.ts";

// ============================================================================
// Setup
// ============================================================================

function createMockDocument() {
  const html = `
    <!DOCTYPE html>
    <html><body>
      <div id="terminal"></div>
      <input id="command-input" />
    </body></html>
  `;
  return new DOMParser().parseFromString(html, "text/html")!;
}

const doc = createMockDocument();
const output = doc.getElementById("terminal")!;

const terminal = {
  output: output,
  printLine(text: string, type: string) {
    const line = doc.createElement("div");
    line.className = `line ${type}`;
    line.textContent = text;
    this.output.appendChild(line);
  },
  clear() {
    this.output.innerHTML = "";
  },
};

// ============================================================================
// Terminal Benchmarks
// ============================================================================

Deno.bench("Terminal.printLine - single call", () => {
  terminal.clear();
  terminal.printLine("test output", "output");
});

Deno.bench("Terminal.printLine - 10 calls", () => {
  terminal.clear();
  for (let i = 0; i < 10; i++) {
    terminal.printLine(`line ${i}`, "output");
  }
});

Deno.bench("Terminal.printLine - 100 calls", () => {
  terminal.clear();
  for (let i = 0; i < 100; i++) {
    terminal.printLine(`line ${i}`, "output");
  }
});

Deno.bench("Terminal.printLine - 1000 calls", () => {
  terminal.clear();
  for (let i = 0; i < 1000; i++) {
    terminal.printLine(`line ${i}`, "output");
  }
});

Deno.bench("Terminal.printLine - long text", () => {
  terminal.clear();
  const longText = "x".repeat(1000);
  terminal.printLine(longText, "output");
});

Deno.bench("Terminal.printLine - special characters", () => {
  terminal.clear();
  terminal.printLine("Test: <>&\"'", "output");
});

Deno.bench("Terminal.clear - empty terminal", () => {
  terminal.clear();
  terminal.clear();
});

Deno.bench("Terminal.clear - 100 lines", () => {
  terminal.clear();
  for (let i = 0; i < 100; i++) {
    terminal.printLine(`line ${i}`, "output");
  }
  terminal.clear();
});

Deno.bench("Terminal.clear - 1000 lines", () => {
  terminal.clear();
  for (let i = 0; i < 1000; i++) {
    terminal.printLine(`line ${i}`, "output");
  }
  terminal.clear();
});

// ============================================================================
// Command History Benchmarks
// ============================================================================

Deno.bench("CommandHistory - add single command", () => {
  const history: string[] = [];
  history.push("echo test");
});

Deno.bench("CommandHistory - add 100 commands", () => {
  const history: string[] = [];
  for (let i = 0; i < 100; i++) {
    history.push(`command ${i}`);
  }
});

Deno.bench("CommandHistory - navigate up 10 times", () => {
  const history = Array.from({ length: 100 }, (_, i) => `cmd ${i}`);
  let historyIndex = history.length;

  for (let i = 0; i < 10; i++) {
    if (historyIndex > 0) historyIndex--;
  }
});

Deno.bench("CommandHistory - navigate down 10 times", () => {
  const history = Array.from({ length: 100 }, (_, i) => `cmd ${i}`);
  let historyIndex = 0;

  for (let i = 0; i < 10; i++) {
    if (historyIndex < history.length) historyIndex++;
  }
});

Deno.bench("CommandHistory - full up/down cycle", () => {
  const history = Array.from({ length: 50 }, (_, i) => `cmd ${i}`);
  let historyIndex = history.length;

  // Go to beginning
  while (historyIndex > 0) historyIndex--;

  // Go to end
  while (historyIndex < history.length) historyIndex++;
});

// ============================================================================
// Command Parsing Benchmarks
// ============================================================================

Deno.bench("CommandParsing - simple command", () => {
  const command = "help";
  const parts = command.trim().split(/\s+/);
  const cmdName = parts[0];
  const args = parts.slice(1);
});

Deno.bench("CommandParsing - command with args", () => {
  const command = "echo hello world";
  const parts = command.trim().split(/\s+/);
  const cmdName = parts[0];
  const args = parts.slice(1);
});

Deno.bench("CommandParsing - command with many args", () => {
  const command = "echo a b c d e f g h i j k l m n o p q r s t";
  const parts = command.trim().split(/\s+/);
  const cmdName = parts[0];
  const args = parts.slice(1);
});

Deno.bench("CommandParsing - command with whitespace", () => {
  const command = "  echo    hello    world  ";
  const parts = command.trim().split(/\s+/);
  const cmdName = parts[0];
  const args = parts.slice(1);
});

Deno.bench("CommandParsing - parse 100 commands", () => {
  const commands = Array.from({ length: 100 }, (_, i) => `cmd ${i} arg1 arg2`);

  for (const command of commands) {
    const parts = command.trim().split(/\s+/);
    const cmdName = parts[0];
    const args = parts.slice(1);
  }
});

// ============================================================================
// State Management Benchmarks
// ============================================================================

Deno.bench("State - JSON.stringify small state", () => {
  const state = { processes: 5, memory: 1024 };
  JSON.stringify(state);
});

Deno.bench("State - JSON.stringify large state", () => {
  const state = {
    processes: 100,
    memory: 1048576,
    vfs: Array.from({ length: 100 }, (_, i) => ({ path: `/file${i}`, size: i })),
  };
  JSON.stringify(state);
});

Deno.bench("State - JSON.parse small state", () => {
  const json = '{"processes":5,"memory":1024}';
  JSON.parse(json);
});

Deno.bench("State - JSON.parse large state", () => {
  const state = {
    processes: 100,
    memory: 1048576,
    vfs: Array.from({ length: 100 }, (_, i) => ({ path: `/file${i}`, size: i })),
  };
  const json = JSON.stringify(state);
  JSON.parse(json);
});

Deno.bench("State - localStorage roundtrip", () => {
  const mockStorage = new Map<string, string>();
  const localStorage = {
    setItem: (key: string, value: string) => mockStorage.set(key, value),
    getItem: (key: string) => mockStorage.get(key) || null,
  };

  const state = { processes: 5, memory: 1024 };
  localStorage.setItem("wos-state", JSON.stringify(state));
  const loaded = localStorage.getItem("wos-state");
  if (loaded) JSON.parse(loaded);
});

// ============================================================================
// Input Validation Benchmarks
// ============================================================================

Deno.bench("InputValidation - validate simple command", () => {
  const command = "help";
  const isValid = command.trim().length > 0;
});

Deno.bench("InputValidation - validate empty command", () => {
  const command = "";
  const isValid = command.trim().length > 0;
});

Deno.bench("InputValidation - validate whitespace command", () => {
  const command = "   ";
  const isValid = command.trim().length > 0;
});

Deno.bench("InputValidation - validate 100 commands", () => {
  const commands = Array.from({ length: 100 }, (_, i) =>
    i % 2 === 0 ? `cmd ${i}` : "   "
  );

  for (const command of commands) {
    const isValid = command.trim().length > 0;
  }
});

// ============================================================================
// DOM Manipulation Benchmarks
// ============================================================================

Deno.bench("DOM - createElement", () => {
  doc.createElement("div");
});

Deno.bench("DOM - createElement and set properties", () => {
  const el = doc.createElement("div");
  el.className = "line output";
  el.textContent = "test";
});

Deno.bench("DOM - appendChild single element", () => {
  const parent = doc.createElement("div");
  const child = doc.createElement("div");
  parent.appendChild(child);
});

Deno.bench("DOM - appendChild 10 elements", () => {
  const parent = doc.createElement("div");
  for (let i = 0; i < 10; i++) {
    const child = doc.createElement("div");
    parent.appendChild(child);
  }
});

Deno.bench("DOM - appendChild 100 elements", () => {
  const parent = doc.createElement("div");
  for (let i = 0; i < 100; i++) {
    const child = doc.createElement("div");
    parent.appendChild(child);
  }
});

Deno.bench("DOM - innerHTML clear", () => {
  const el = doc.createElement("div");
  for (let i = 0; i < 10; i++) {
    const child = doc.createElement("div");
    el.appendChild(child);
  }
  el.innerHTML = "";
});

Deno.bench("DOM - getElementById", () => {
  doc.getElementById("terminal");
});

// ============================================================================
// Integration Benchmarks
// ============================================================================

Deno.bench("Integration - complete command flow", () => {
  terminal.clear();
  const history: string[] = [];

  const command = "echo hello world";
  const parts = command.trim().split(/\s+/);
  const cmdName = parts[0];
  const args = parts.slice(1);

  terminal.printLine(`$ ${command}`, "input");
  terminal.printLine(args.join(" "), "output");

  if (command.trim()) {
    history.push(command.trim());
  }
});

Deno.bench("Integration - 10 command workflow", () => {
  terminal.clear();
  const history: string[] = [];

  for (let i = 0; i < 10; i++) {
    const command = `echo test ${i}`;
    const parts = command.trim().split(/\s+/);
    const cmdName = parts[0];
    const args = parts.slice(1);

    terminal.printLine(`$ ${command}`, "input");
    terminal.printLine(args.join(" "), "output");

    if (command.trim()) {
      history.push(command.trim());
    }
  }
});

Deno.bench("Integration - 100 command workflow", () => {
  terminal.clear();
  const history: string[] = [];

  for (let i = 0; i < 100; i++) {
    const command = `echo test ${i}`;
    const parts = command.trim().split(/\s+/);
    const cmdName = parts[0];
    const args = parts.slice(1);

    terminal.printLine(`$ ${command}`, "input");
    terminal.printLine(args.join(" "), "output");

    if (command.trim()) {
      history.push(command.trim());
    }
  }
});

Deno.bench("Integration - state persistence workflow", () => {
  const mockStorage = new Map<string, string>();
  const localStorage = {
    setItem: (key: string, value: string) => mockStorage.set(key, value),
    getItem: (key: string) => mockStorage.get(key) || null,
  };

  // Execute some commands
  const history: string[] = [];
  for (let i = 0; i < 5; i++) {
    history.push(`cmd ${i}`);
  }

  // Save state
  const state = { history, processes: 5 };
  localStorage.setItem("wos-state", JSON.stringify(state));

  // Load state
  const loaded = localStorage.getItem("wos-state");
  if (loaded) {
    const parsedState = JSON.parse(loaded);
  }
});

// ============================================================================
// Summary
// ============================================================================

console.log(`
📊 Benchmark Suite Summary:
- Terminal operations: 9 benchmarks
- Command history: 5 benchmarks
- Command parsing: 5 benchmarks
- State management: 5 benchmarks
- Input validation: 4 benchmarks
- DOM manipulation: 7 benchmarks
- Integration workflows: 4 benchmarks

Total: 39 performance benchmarks

Run with:
  deno bench app.bench.ts --allow-net
`);
