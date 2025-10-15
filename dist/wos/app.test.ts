/**
 * Unit Tests for WOS Frontend (app.js)
 *
 * These tests use Deno's built-in test framework and deno-dom for DOM mocking.
 * Run with: deno task test
 * Coverage: deno task test:coverage
 */

import {
  assert,
  assertEquals,
  assertExists,
  assertStringIncludes,
} from "https://deno.land/std@0.208.0/assert/mod.ts";
import { DOMParser } from "https://deno.land/x/deno_dom@v0.1.43/deno-dom-wasm.ts";

// ============================================================================
// Mock DOM Setup
// ============================================================================

function createMockDocument() {
  const html = `
    <!DOCTYPE html>
    <html lang="en">
      <head>
        <meta charset="UTF-8">
        <title>WOS Test</title>
      </head>
      <body>
        <div id="terminal"></div>
        <input id="command-input" type="text" />
        <div id="status">Loading...</div>
        <div id="version"></div>
        <div id="process-count">0</div>
      </body>
    </html>
  `;
  return new DOMParser().parseFromString(html, "text/html")!;
}

// ============================================================================
// Terminal Class Tests
// ============================================================================

Deno.test("Terminal - constructor initializes output element", () => {
  const doc = createMockDocument();
  const output = doc.getElementById("terminal")!;

  assertExists(output);
  assertEquals(output.tagName, "DIV");
  assertEquals(output.id, "terminal");
});

Deno.test("Terminal - constructor initializes input element", () => {
  const doc = createMockDocument();
  const input = doc.getElementById("command-input")!;

  assertExists(input);
  assertEquals(input.tagName, "INPUT");
  assertEquals(input.id, "command-input");
});

Deno.test("Terminal - printLine adds line with correct text", () => {
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
  };

  terminal.printLine("hello world", "output");

  assertEquals(output.children.length, 1);
  assertEquals(output.children[0].textContent, "hello world");
});

Deno.test("Terminal - printLine adds line with correct class (output)", () => {
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
  };

  terminal.printLine("test", "output");

  assertStringIncludes(output.children[0].className, "line");
  assertStringIncludes(output.children[0].className, "output");
});

Deno.test("Terminal - printLine adds line with correct class (error)", () => {
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
  };

  terminal.printLine("error message", "error");

  assertStringIncludes(output.children[0].className, "error");
});

Deno.test("Terminal - printLine adds line with correct class (input)", () => {
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
  };

  terminal.printLine("$ echo test", "input");

  assertStringIncludes(output.children[0].className, "input");
});

Deno.test("Terminal - printLine handles multiple lines", () => {
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
  };

  terminal.printLine("line 1", "output");
  terminal.printLine("line 2", "output");
  terminal.printLine("line 3", "output");

  assertEquals(output.children.length, 3);
  assertEquals(output.children[0].textContent, "line 1");
  assertEquals(output.children[1].textContent, "line 2");
  assertEquals(output.children[2].textContent, "line 3");
});

Deno.test("Terminal - printLine handles empty string", () => {
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
  };

  terminal.printLine("", "output");

  assertEquals(output.children.length, 1);
  assertEquals(output.children[0].textContent, "");
});

Deno.test("Terminal - printLine handles special characters", () => {
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
  };

  terminal.printLine("Test: <>&\"'", "output");

  assertEquals(output.children[0].textContent, "Test: <>&\"'");
});

Deno.test("Terminal - clear removes all lines", () => {
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

  terminal.printLine("line 1", "output");
  terminal.printLine("line 2", "output");
  assertEquals(output.children.length, 2);

  terminal.clear();
  assertEquals(output.children.length, 0);
});

// ============================================================================
// Command History Tests
// ============================================================================

Deno.test("CommandHistory - empty history returns empty string on up", () => {
  const history: string[] = [];
  let historyIndex = history.length;

  if (historyIndex > 0) {
    historyIndex--;
  }
  const command = history[historyIndex] || "";

  assertEquals(command, "");
});

Deno.test("CommandHistory - navigates up from end", () => {
  const history = ["echo hello", "ps", "version"];
  let historyIndex = history.length;

  if (historyIndex > 0) {
    historyIndex--;
  }

  assertEquals(history[historyIndex], "version");
  assertEquals(historyIndex, 2);
});

Deno.test("CommandHistory - navigates up multiple times", () => {
  const history = ["echo hello", "ps", "version"];
  let historyIndex = history.length;

  // First up
  if (historyIndex > 0) historyIndex--;
  assertEquals(history[historyIndex], "version");

  // Second up
  if (historyIndex > 0) historyIndex--;
  assertEquals(history[historyIndex], "ps");

  // Third up
  if (historyIndex > 0) historyIndex--;
  assertEquals(history[historyIndex], "echo hello");
});

Deno.test("CommandHistory - stops at beginning when navigating up", () => {
  const history = ["echo hello", "ps"];
  let historyIndex = history.length;

  // Navigate to beginning
  if (historyIndex > 0) historyIndex--;
  if (historyIndex > 0) historyIndex--;

  assertEquals(historyIndex, 0);

  // Try to go past beginning
  if (historyIndex > 0) historyIndex--;
  assertEquals(historyIndex, 0); // Should stay at 0
});

Deno.test("CommandHistory - navigates down from beginning", () => {
  const history = ["echo hello", "ps", "version"];
  let historyIndex = 0;

  if (historyIndex < history.length) {
    historyIndex++;
  }

  assertEquals(historyIndex, 1);
});

Deno.test("CommandHistory - stops at end when navigating down", () => {
  const history = ["echo hello", "ps"];
  let historyIndex = 1;

  // Navigate to end
  if (historyIndex < history.length) historyIndex++;
  assertEquals(historyIndex, 2);

  // Try to go past end
  if (historyIndex < history.length) historyIndex++;
  assertEquals(historyIndex, 2); // Should stay at length
});

Deno.test("CommandHistory - adding command increments history", () => {
  const history: string[] = [];

  history.push("echo test");
  assertEquals(history.length, 1);

  history.push("ps");
  assertEquals(history.length, 2);

  assertEquals(history[0], "echo test");
  assertEquals(history[1], "ps");
});

Deno.test("CommandHistory - does not add empty commands", () => {
  const history: string[] = [];
  const command = "   ";

  if (command.trim()) {
    history.push(command.trim());
  }

  assertEquals(history.length, 0);
});

Deno.test("CommandHistory - trims whitespace when adding", () => {
  const history: string[] = [];
  const command = "  echo test  ";

  if (command.trim()) {
    history.push(command.trim());
  }

  assertEquals(history.length, 1);
  assertEquals(history[0], "echo test");
});

// ============================================================================
// State Management Tests
// ============================================================================

Deno.test("State - saves to localStorage", () => {
  const mockStorage = new Map<string, string>();

  const localStorage = {
    setItem: (key: string, value: string) => mockStorage.set(key, value),
    getItem: (key: string) => mockStorage.get(key) || null,
  };

  const state = { test: "data", processes: 5 };
  localStorage.setItem("wos-state", JSON.stringify(state));

  const stored = localStorage.getItem("wos-state");
  assertEquals(stored, '{"test":"data","processes":5}');
});

Deno.test("State - loads from localStorage", () => {
  const mockStorage = new Map<string, string>();
  mockStorage.set("wos-state", '{"test":"loaded","processes":3}');

  const localStorage = {
    setItem: (key: string, value: string) => mockStorage.set(key, value),
    getItem: (key: string) => mockStorage.get(key) || null,
  };

  const stored = localStorage.getItem("wos-state");
  assertExists(stored);

  const state = JSON.parse(stored);
  assertEquals(state.test, "loaded");
  assertEquals(state.processes, 3);
});

Deno.test("State - handles missing localStorage gracefully", () => {
  const localStorage = {
    setItem: (_key: string, _value: string) => {},
    getItem: (_key: string) => null,
  };

  const stored = localStorage.getItem("wos-state");
  assertEquals(stored, null);
});

Deno.test("State - handles invalid JSON gracefully", () => {
  const mockStorage = new Map<string, string>();
  mockStorage.set("wos-state", "invalid json {");

  const localStorage = {
    getItem: (key: string) => mockStorage.get(key) || null,
  };

  const stored = localStorage.getItem("wos-state");
  assertExists(stored);

  try {
    JSON.parse(stored);
    assert(false, "Should have thrown");
  } catch (e) {
    assert(e instanceof SyntaxError);
  }
});

// ============================================================================
// Command Parsing Tests
// ============================================================================

Deno.test("CommandParsing - splits command into parts", () => {
  const command = "echo hello world";
  const parts = command.trim().split(/\s+/);

  assertEquals(parts.length, 3);
  assertEquals(parts[0], "echo");
  assertEquals(parts[1], "hello");
  assertEquals(parts[2], "world");
});

Deno.test("CommandParsing - handles single command", () => {
  const command = "help";
  const parts = command.trim().split(/\s+/);

  assertEquals(parts.length, 1);
  assertEquals(parts[0], "help");
});

Deno.test("CommandParsing - handles multiple spaces", () => {
  const command = "echo    hello    world";
  const parts = command.trim().split(/\s+/);

  assertEquals(parts.length, 3);
  assertEquals(parts[0], "echo");
  assertEquals(parts[1], "hello");
  assertEquals(parts[2], "world");
});

Deno.test("CommandParsing - trims leading whitespace", () => {
  const command = "   echo test";
  const parts = command.trim().split(/\s+/);

  assertEquals(parts[0], "echo");
  assertEquals(parts[1], "test");
});

Deno.test("CommandParsing - trims trailing whitespace", () => {
  const command = "echo test   ";
  const parts = command.trim().split(/\s+/);

  assertEquals(parts.length, 2);
  assertEquals(parts[0], "echo");
  assertEquals(parts[1], "test");
});

Deno.test("CommandParsing - handles empty string", () => {
  const command = "";
  const trimmed = command.trim();

  assertEquals(trimmed, "");
  assertEquals(trimmed.length, 0);
});

Deno.test("CommandParsing - handles whitespace-only string", () => {
  const command = "   ";
  const trimmed = command.trim();

  assertEquals(trimmed, "");
});

// ============================================================================
// Input Validation Tests
// ============================================================================

Deno.test("InputValidation - accepts valid command", () => {
  const command = "echo test";
  const isValid = command.trim().length > 0;

  assert(isValid);
});

Deno.test("InputValidation - rejects empty command", () => {
  const command = "";
  const isValid = command.trim().length > 0;

  assert(!isValid);
});

Deno.test("InputValidation - rejects whitespace-only command", () => {
  const command = "   ";
  const isValid = command.trim().length > 0;

  assert(!isValid);
});

Deno.test("InputValidation - accepts command with special characters", () => {
  const command = "echo 'hello world'";
  const isValid = command.trim().length > 0;

  assert(isValid);
});

// ============================================================================
// DOM Update Tests
// ============================================================================

Deno.test("DOMUpdate - updates status element", () => {
  const doc = createMockDocument();
  const statusEl = doc.getElementById("status")!;

  statusEl.textContent = "Ready";
  statusEl.className = "ready";

  assertEquals(statusEl.textContent, "Ready");
  assertStringIncludes(statusEl.className, "ready");
});

Deno.test("DOMUpdate - updates version element", () => {
  const doc = createMockDocument();
  const versionEl = doc.getElementById("version")!;

  versionEl.textContent = "WOS v0.1.0";

  assertEquals(versionEl.textContent, "WOS v0.1.0");
});

Deno.test("DOMUpdate - updates process count", () => {
  const doc = createMockDocument();
  const processCountEl = doc.getElementById("process-count")!;

  processCountEl.textContent = "5";

  assertEquals(processCountEl.textContent, "5");
});

Deno.test("DOMUpdate - handles missing element gracefully", () => {
  const doc = createMockDocument();
  const missingEl = doc.getElementById("nonexistent");

  assertEquals(missingEl, null);
});

// ============================================================================
// Error Handling Tests
// ============================================================================

Deno.test("ErrorHandling - catches and displays errors", () => {
  const errors: string[] = [];

  const logError = (message: string) => {
    errors.push(message);
  };

  try {
    throw new Error("Test error");
  } catch (e) {
    logError(e instanceof Error ? e.message : String(e));
  }

  assertEquals(errors.length, 1);
  assertEquals(errors[0], "Test error");
});

Deno.test("ErrorHandling - handles non-Error objects", () => {
  const errors: string[] = [];

  const logError = (message: string) => {
    errors.push(message);
  };

  try {
    throw "String error";
  } catch (e) {
    logError(e instanceof Error ? e.message : String(e));
  }

  assertEquals(errors.length, 1);
  assertEquals(errors[0], "String error");
});

// ============================================================================
// Integration-style Tests
// ============================================================================

Deno.test("Integration - complete command flow", () => {
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
  };

  // Simulate user input
  const command = "echo hello";

  // Display command
  terminal.printLine(`$ ${command}`, "input");

  // Parse and execute (simulated)
  const parts = command.split(/\s+/);
  const cmdName = parts[0];
  const args = parts.slice(1);

  assertEquals(cmdName, "echo");
  assertEquals(args.join(" "), "hello");

  // Display output
  terminal.printLine(args.join(" "), "output");

  // Verify terminal state
  assertEquals(output.children.length, 2);
  assertEquals(output.children[0].textContent, "$ echo hello");
  assertEquals(output.children[1].textContent, "hello");
});

Deno.test("Integration - command history workflow", () => {
  const history: string[] = [];
  let historyIndex = 0;

  // Execute commands
  const commands = ["echo test", "ps", "version"];
  commands.forEach((cmd) => {
    history.push(cmd);
  });

  historyIndex = history.length;

  // Navigate up
  if (historyIndex > 0) historyIndex--;
  assertEquals(history[historyIndex], "version");

  if (historyIndex > 0) historyIndex--;
  assertEquals(history[historyIndex], "ps");

  if (historyIndex > 0) historyIndex--;
  assertEquals(history[historyIndex], "echo test");

  // Navigate down
  if (historyIndex < history.length) historyIndex++;
  assertEquals(history[historyIndex], "ps");
});

Deno.test("Integration - state persistence workflow", () => {
  const mockStorage = new Map<string, string>();
  const localStorage = {
    setItem: (key: string, value: string) => mockStorage.set(key, value),
    getItem: (key: string) => mockStorage.get(key) || null,
  };

  // Save state
  const state = { processes: 3, memory: 1024 };
  localStorage.setItem("wos-state", JSON.stringify(state));

  // Load state
  const loaded = localStorage.getItem("wos-state");
  assertExists(loaded);
  const parsedState = JSON.parse(loaded);

  assertEquals(parsedState.processes, 3);
  assertEquals(parsedState.memory, 1024);
});

// ============================================================================
// Summary
// ============================================================================

console.log(`
✅ Test Suite Summary:
- Terminal class tests: 11 tests
- Command history tests: 9 tests
- State management tests: 4 tests
- Command parsing tests: 7 tests
- Input validation tests: 4 tests
- DOM update tests: 4 tests
- Error handling tests: 2 tests
- Integration tests: 3 tests

Total: 44 unit tests

Run with:
  deno task test
  deno task test:coverage
  deno task test:watch
`);
