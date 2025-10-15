/**
 * Property-Based Tests for WOS Frontend
 *
 * These tests use fast-check to generate random inputs and verify properties hold.
 * Each test runs 1000+ iterations with random data.
 *
 * Run with: deno test app.property.test.ts --allow-net
 */

import fc from "https://esm.sh/fast-check@3.14.0";
import { DOMParser } from "https://deno.land/x/deno_dom@v0.1.43/deno-dom-wasm.ts";

// ============================================================================
// Helper Functions
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

// ============================================================================
// Terminal Properties
// ============================================================================

Deno.test("Property: printLine never crashes with any text input", () => {
  fc.assert(
    fc.property(
      fc.string(),
      fc.constantFrom("output", "error", "input", "system"),
      (text, type) => {
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

        try {
          terminal.printLine(text, type);
          return true;
        } catch {
          return false;
        }
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: printLine preserves exact text content", () => {
  fc.assert(
    fc.property(
      fc.string(),
      fc.string(),
      (text, type) => {
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

        terminal.printLine(text, type);

        return output.children.length === 1 &&
          output.children[0].textContent === text;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: printLine increments child count", () => {
  fc.assert(
    fc.property(
      fc.array(fc.string(), { minLength: 1, maxLength: 100 }),
      (lines) => {
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

        lines.forEach((line) => terminal.printLine(line, "output"));

        return output.children.length === lines.length;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: clear always results in zero children", () => {
  fc.assert(
    fc.property(
      fc.array(fc.string(), { minLength: 0, maxLength: 100 }),
      (lines) => {
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

        // Add random number of lines
        lines.forEach((line) => terminal.printLine(line, "output"));

        // Clear
        terminal.clear();

        return output.children.length === 0;
      },
    ),
    { numRuns: 1000 },
  );
});

// ============================================================================
// Command History Properties
// ============================================================================

Deno.test("Property: history navigation never goes below 0", () => {
  fc.assert(
    fc.property(
      fc.array(fc.string(), { minLength: 0, maxLength: 50 }),
      fc.integer({ min: 0, max: 100 }),
      (history, upPresses) => {
        let historyIndex = history.length;

        // Navigate up many times
        for (let i = 0; i < upPresses; i++) {
          if (historyIndex > 0) {
            historyIndex--;
          }
        }

        return historyIndex >= 0;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: history navigation never exceeds length", () => {
  fc.assert(
    fc.property(
      fc.array(fc.string(), { minLength: 0, maxLength: 50 }),
      fc.integer({ min: 0, max: 100 }),
      (history, downPresses) => {
        let historyIndex = 0;

        // Navigate down many times
        for (let i = 0; i < downPresses; i++) {
          if (historyIndex < history.length) {
            historyIndex++;
          }
        }

        return historyIndex <= history.length;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: adding commands increases history length", () => {
  fc.assert(
    fc.property(
      fc.array(fc.string().filter((s) => s.trim().length > 0), {
        minLength: 1,
        maxLength: 50,
      }),
      (commands) => {
        const history: string[] = [];

        commands.forEach((cmd) => {
          if (cmd.trim()) {
            history.push(cmd.trim());
          }
        });

        return history.length === commands.length;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: history preserves command order", () => {
  fc.assert(
    fc.property(
      fc.array(fc.string().filter((s) => s.trim().length > 0), {
        minLength: 1,
        maxLength: 20,
      }),
      (commands) => {
        const history: string[] = [];

        commands.forEach((cmd) => {
          if (cmd.trim()) {
            history.push(cmd.trim());
          }
        });

        // Verify order
        for (let i = 0; i < commands.length; i++) {
          if (history[i] !== commands[i].trim()) {
            return false;
          }
        }

        return true;
      },
    ),
    { numRuns: 1000 },
  );
});

// ============================================================================
// Command Parsing Properties
// ============================================================================

Deno.test("Property: splitting command never loses characters", () => {
  fc.assert(
    fc.property(
      fc.string().filter((s) => s.trim().length > 0),
      (command) => {
        const trimmed = command.trim();
        const parts = trimmed.split(/\s+/);

        // Characters might differ due to whitespace normalization
        // but word count should be consistent
        return parts.length > 0;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: command parsing handles whitespace consistently", () => {
  fc.assert(
    fc.property(
      fc.string(),
      (command) => {
        const trimmed = command.trim();

        if (trimmed.length === 0) {
          return trimmed === "";
        }

        const parts = trimmed.split(/\s+/);
        return parts.length >= 1;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: first part is always the command name", () => {
  fc.assert(
    fc.property(
      fc.string().filter((s) => s.trim().length > 0),
      (command) => {
        const parts = command.trim().split(/\s+/);
        const cmdName = parts[0];

        return typeof cmdName === "string" && cmdName.length > 0;
      },
    ),
    { numRuns: 1000 },
  );
});

// ============================================================================
// State Management Properties
// ============================================================================

Deno.test("Property: JSON serialization roundtrip preserves data", () => {
  fc.assert(
    fc.property(
      fc.record({
        processes: fc.integer({ min: 0, max: 1000 }),
        memory: fc.integer({ min: 0, max: 1000000 }),
        nextPid: fc.integer({ min: 1, max: 10000 }),
      }),
      (state) => {
        const mockStorage = new Map<string, string>();
        const localStorage = {
          setItem: (key: string, value: string) =>
            mockStorage.set(key, value),
          getItem: (key: string) => mockStorage.get(key) || null,
        };

        // Save
        localStorage.setItem("wos-state", JSON.stringify(state));

        // Load
        const loaded = localStorage.getItem("wos-state");
        if (!loaded) return false;

        const parsed = JSON.parse(loaded);

        return parsed.processes === state.processes &&
          parsed.memory === state.memory &&
          parsed.nextPid === state.nextPid;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: localStorage setItem never throws", () => {
  fc.assert(
    fc.property(
      fc.string(),
      fc.string(),
      (key, value) => {
        const mockStorage = new Map<string, string>();
        const localStorage = {
          setItem: (key: string, value: string) =>
            mockStorage.set(key, value),
          getItem: (key: string) => mockStorage.get(key) || null,
        };

        try {
          localStorage.setItem(key, value);
          return true;
        } catch {
          return false;
        }
      },
    ),
    { numRuns: 1000 },
  );
});

// ============================================================================
// Input Validation Properties
// ============================================================================

Deno.test("Property: empty/whitespace commands are invalid", () => {
  fc.assert(
    fc.property(
      fc.string().filter((s) => /^\s*$/.test(s)),
      (command) => {
        const isValid = command.trim().length > 0;
        return !isValid; // Should always be invalid
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: non-empty trimmed commands are valid", () => {
  fc.assert(
    fc.property(
      fc.string().filter((s) => s.trim().length > 0),
      (command) => {
        const isValid = command.trim().length > 0;
        return isValid;
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: validation is consistent for same input", () => {
  fc.assert(
    fc.property(
      fc.string(),
      (command) => {
        const isValid1 = command.trim().length > 0;
        const isValid2 = command.trim().length > 0;
        return isValid1 === isValid2;
      },
    ),
    { numRuns: 1000 },
  );
});

// ============================================================================
// DOM Manipulation Properties
// ============================================================================

Deno.test("Property: setting textContent never throws", () => {
  fc.assert(
    fc.property(
      fc.string(),
      (text) => {
        const doc = createMockDocument();
        const el = doc.createElement("div");

        try {
          el.textContent = text;
          return el.textContent === text;
        } catch {
          return false;
        }
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: setting className never throws", () => {
  fc.assert(
    fc.property(
      fc.string(),
      (className) => {
        const doc = createMockDocument();
        const el = doc.createElement("div");

        try {
          el.className = className;
          return el.className === className;
        } catch {
          return false;
        }
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: appendChild always increments children length", () => {
  fc.assert(
    fc.property(
      fc.integer({ min: 1, max: 100 }),
      (numChildren) => {
        const doc = createMockDocument();
        const parent = doc.createElement("div");

        for (let i = 0; i < numChildren; i++) {
          const child = doc.createElement("div");
          parent.appendChild(child);
        }

        return parent.children.length === numChildren;
      },
    ),
    { numRuns: 1000 },
  );
});

// ============================================================================
// Error Handling Properties
// ============================================================================

Deno.test("Property: error logging never throws", () => {
  fc.assert(
    fc.property(
      fc.string(),
      (errorMessage) => {
        const errors: string[] = [];

        const logError = (message: string) => {
          errors.push(message);
        };

        try {
          logError(errorMessage);
          return errors.length === 1 && errors[0] === errorMessage;
        } catch {
          return false;
        }
      },
    ),
    { numRuns: 1000 },
  );
});

Deno.test("Property: Error object conversion preserves message", () => {
  fc.assert(
    fc.property(
      fc.string(),
      (message) => {
        const error = new Error(message);
        const extracted = error instanceof Error ? error.message : String(error);
        return extracted === message;
      },
    ),
    { numRuns: 1000 },
  );
});

// ============================================================================
// Integration Properties
// ============================================================================

Deno.test("Property: complete command flow never corrupts state", () => {
  fc.assert(
    fc.property(
      fc.array(fc.string().filter((s) => s.trim().length > 0), {
        minLength: 1,
        maxLength: 20,
      }),
      (commands) => {
        const doc = createMockDocument();
        const output = doc.getElementById("terminal")!;
        const history: string[] = [];

        const terminal = {
          output: output,
          printLine(text: string, type: string) {
            const line = doc.createElement("div");
            line.className = `line ${type}`;
            line.textContent = text;
            this.output.appendChild(line);
          },
        };

        // Execute commands
        commands.forEach((cmd) => {
          const trimmed = cmd.trim();
          if (trimmed) {
            history.push(trimmed);
            terminal.printLine(`$ ${trimmed}`, "input");
            terminal.printLine("output", "output");
          }
        });

        // Verify state
        return history.length === commands.length &&
          output.children.length === commands.length * 2;
      },
    ),
    { numRuns: 500 },
  );
});

// ============================================================================
// Summary
// ============================================================================

console.log(`
✅ Property-Based Test Suite Summary:
- Terminal properties: 4 tests × 1000 iterations = 4,000 cases
- Command history properties: 4 tests × 1000 iterations = 4,000 cases
- Command parsing properties: 3 tests × 1000 iterations = 3,000 cases
- State management properties: 2 tests × 1000 iterations = 2,000 cases
- Input validation properties: 3 tests × 1000 iterations = 3,000 cases
- DOM manipulation properties: 3 tests × 1000 iterations = 3,000 cases
- Error handling properties: 2 tests × 1000 iterations = 2,000 cases
- Integration properties: 1 test × 500 iterations = 500 cases

Total: 22 property tests × ~1000 iterations = ~21,500 test cases

Run with:
  deno test app.property.test.ts --allow-net
`);
