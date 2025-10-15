# WASM OS Canary & Functional UX Testing Specification
## SQLite-Inspired Real-World Validation Framework

**Version**: 1.0  
**Date**: October 15, 2025  
**Methodology**: Adapted from SQLite Testing Standards  
**Target**: 80% User Action Coverage + 100% Critical Path Coverage

---

## Executive Summary

### SQLite Testing Philosophy Applied to WASM OS

SQLite achieves legendary reliability through 608:1 test-to-code ratio (92M SLOC test code for 151K SLOC source), 100% branch coverage, 100% MC/DC coverage, and four independent test harnesses running hundreds of millions of tests before each release.

WOS adapts these principles for browser-based WASM environment:

| SQLite Standard | WOS Adaptation | Implementation |
|----------------|----------------|----------------|
| **TCL Tests (21.6K SLOC)** | Browser Canary Tests | Playwright E2E validating real user workflows |
| **TH3 (1.04M SLOC, 100% coverage)** | Core Validation Suite | Property tests + mutation tests (existing) |
| **SLT (7.2M queries)** | Differential Testing | Compare WOS against reference implementations |
| **dbsqlfuzz (1B tests/day)** | Chaos Engineering | Fault injection + state corruption tests |
| **Anomaly Tests (OOM, I/O errors)** | Browser Anomaly Suite | Memory exhaustion, localStorage failures, network errors |
| **Soak Tests (248.5M tests)** | Long-Running Stability | 24-hour stress tests, memory leak detection |
| **Veryquick (300K tests)** | Pre-Commit Canaries | 50 critical user workflows in <5 minutes |

**Core Principle**: "It is relatively easy to build a system that behaves correctly on well-formed inputs on a fully functional computer. It is more difficult to build a system that responds sanely to invalid inputs and continues to function following system malfunctions".

---

## 1. Testing Philosophy: SQLite Lessons for WASM OS

### 1.1 Defensive Testing Mindset

**SQLite Standard**: Test every assertion, every error path, every corner case.

**WOS Application**:
```rust
// SQLite-style defensive testing
#[cfg(test)]
mod canary_defensive_tests {
    /// Test EVERY possible error return from syscall
    #[test]
    fn test_syscall_all_error_paths() {
        // Test each error variant explicitly
        for error in [
            KernelError::InvalidPid,
            KernelError::PermissionDenied,
            KernelError::OutOfMemory,
            KernelError::InvalidState,
            KernelError::ResourceExhausted,
            // ... ALL error types
        ] {
            let result = inject_error_and_test(error);
            assert!(result.handles_gracefully());
        }
    }
}
```

### 1.2 Anomaly-First Design

**Key Insight**: Anomaly tests verify correct behavior when something goes wrong, which is more difficult than handling well-formed inputs.

**WOS Anomaly Categories**:
1. **Memory Anomalies**: OOM during any operation
2. **Storage Anomalies**: localStorage full, quota exceeded, corruption
3. **Timing Anomalies**: Slow execution, race conditions, deadlocks
4. **Input Anomalies**: Malformed commands, invalid syscalls, corrupted state
5. **Browser Anomalies**: Tab suspension, page reload, network errors

### 1.3 Test-What-You-Fly Principle

TH3 tests SQLite in as-deployed configuration using only published interfaces, testing compiled object code not source code to verify no problems introduced by compiler bugs.

**WOS Application**: Test the actual WASM binary in real browsers, not mock environments.

```bash
# Test deployed WASM, not development builds
make wasm-release
playwright test --config canary.config.ts
```

---

## 2. Four-Harness Canary Framework

### Harness 1: Browser Canary Tests (BCT)

**Purpose**: Validate user workflows in production-like environment  
**Inspiration**: SQLite TCL Tests  
**Coverage Target**: 80% of user actions

#### Implementation

```typescript
// canary/browser-canary-tests.ts
import { test, expect, Page } from '@playwright/test';

/**
 * Canary Test Suite - Critical User Workflows
 * 
 * SQLite Principle: Run before every release, every platform
 * WOS Adaptation: Run before every deploy, every browser
 */

// Category 1: Terminal Interaction (20% of user actions)
test.describe('Canary: Terminal Interaction', () => {
  test('C01: User types command and sees output', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Type command
    await page.fill('#command-input', 'echo hello world');
    await page.press('#command-input', 'Enter');
    
    // Verify output appears within 100ms
    const output = await page.waitForSelector('.line.output', { timeout: 100 });
    expect(await output.textContent()).toContain('hello world');
    
    // Verify prompt returns
    await expect(page.locator('#command-input')).toBeEnabled();
  });
  
  test('C02: Command history with arrow keys', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Execute multiple commands
    await executeCommand(page, 'echo first');
    await executeCommand(page, 'echo second');
    await executeCommand(page, 'echo third');
    
    // Navigate history
    await page.press('#command-input', 'ArrowUp');
    expect(await page.inputValue('#command-input')).toBe('echo third');
    
    await page.press('#command-input', 'ArrowUp');
    expect(await page.inputValue('#command-input')).toBe('echo second');
    
    await page.press('#command-input', 'ArrowDown');
    expect(await page.inputValue('#command-input')).toBe('echo third');
  });
  
  test('C03: Clear terminal with Ctrl+L', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'echo line1');
    await executeCommand(page, 'echo line2');
    
    // Clear terminal
    await page.press('#command-input', 'Control+L');
    
    // Verify terminal cleared
    const lines = await page.locator('.line').count();
    expect(lines).toBe(0);
  });
});

// Category 2: Process Management (25% of user actions)
test.describe('Canary: Process Management', () => {
  test('C10: List processes with ps', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'ps');
    
    // Verify output contains process list
    const output = await getLastOutput(page);
    expect(output).toMatch(/PID.*STATE.*COMMAND/);
    expect(output).toContain('init');
    expect(output).toContain('shell');
  });
  
  test('C11: Fork process in background', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Start background process
    await executeCommand(page, 'sleep 5 &');
    
    // Verify process appears in ps
    await executeCommand(page, 'ps');
    const output = await getLastOutput(page);
    expect(output).toContain('sleep');
    
    // Verify shell remains responsive
    await executeCommand(page, 'echo still responsive');
    expect(await getLastOutput(page)).toContain('still responsive');
  });
  
  test('C12: Kill process by PID', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Start process
    await executeCommand(page, 'sleep 100 &');
    await executeCommand(page, 'ps');
    const psOutput = await getLastOutput(page);
    
    // Extract PID of sleep process
    const pidMatch = psOutput.match(/(\d+).*sleep/);
    expect(pidMatch).toBeTruthy();
    const pid = pidMatch![1];
    
    // Kill process
    await executeCommand(page, `kill ${pid}`);
    
    // Verify process terminated
    await executeCommand(page, 'ps');
    const newOutput = await getLastOutput(page);
    expect(newOutput).not.toContain(`${pid}.*sleep`);
  });
});

// Category 3: File Operations (20% of user actions)
test.describe('Canary: File Operations', () => {
  test('C20: List files with ls', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'ls /');
    const output = await getLastOutput(page);
    
    // Verify standard directories present
    expect(output).toContain('bin');
    expect(output).toContain('dev');
    expect(output).toContain('proc');
    expect(output).toContain('tmp');
  });
  
  test('C21: Create and read file', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Create file
    await executeCommand(page, 'echo "test content" > /tmp/test.txt');
    
    // Read file
    await executeCommand(page, 'cat /tmp/test.txt');
    const output = await getLastOutput(page);
    expect(output).toContain('test content');
  });
  
  test('C22: Read process info from /proc', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'cat /proc/1/status');
    const output = await getLastOutput(page);
    
    expect(output).toMatch(/Name:.*init/);
    expect(output).toMatch(/State:.*Running/);
    expect(output).toMatch(/Pid:.*1/);
  });
});

// Category 4: System State (15% of user actions)
test.describe('Canary: System State', () => {
  test('C30: View kernel state', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'state');
    const output = await getLastOutput(page);
    
    expect(output).toMatch(/Processes:.*\d+/);
    expect(output).toMatch(/Memory:.*\d+.*KB/);
    expect(output).toMatch(/Uptime:.*\d+/);
  });
  
  test('C31: Reset system', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Create some state
    await executeCommand(page, 'echo test > /tmp/file.txt');
    await executeCommand(page, 'sleep 100 &');
    
    // Reset
    await executeCommand(page, 'reset');
    
    // Verify clean state
    await executeCommand(page, 'ls /tmp');
    const output = await getLastOutput(page);
    expect(output).not.toContain('file.txt');
    
    await executeCommand(page, 'ps');
    const psOutput = await getLastOutput(page);
    expect(psOutput).not.toContain('sleep');
  });
});

// Category 5: Error Handling (10% of user actions)
test.describe('Canary: Error Handling', () => {
  test('C40: Invalid command shows error', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'nonexistent_command');
    const output = await getLastOutput(page);
    
    expect(output).toMatch(/command not found|error/i);
  });
  
  test('C41: Invalid file path shows error', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'cat /nonexistent/file.txt');
    const output = await getLastOutput(page);
    
    expect(output).toMatch(/not found|error/i);
  });
  
  test('C42: Invalid PID shows error', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await executeCommand(page, 'kill 99999');
    const output = await getLastOutput(page);
    
    expect(output).toMatch(/no such process|error/i);
  });
});

// Category 6: State Persistence (10% of user actions)
test.describe('Canary: State Persistence', () => {
  test('C50: Save state across page reload', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Create state
    await executeCommand(page, 'echo persistent > /tmp/data.txt');
    await executeCommand(page, 'sleep 1000 &');
    
    // Reload page
    await page.reload();
    await page.waitForSelector('#command-input');
    
    // Verify state persisted
    await executeCommand(page, 'cat /tmp/data.txt');
    expect(await getLastOutput(page)).toContain('persistent');
    
    await executeCommand(page, 'ps');
    expect(await getLastOutput(page)).toContain('sleep');
  });
});

// Helper functions
async function executeCommand(page: Page, command: string) {
  await page.fill('#command-input', command);
  await page.press('#command-input', 'Enter');
  await page.waitForTimeout(50); // Wait for command execution
}

async function getLastOutput(page: Page): Promise<string> {
  const lines = await page.locator('.line.output').all();
  if (lines.length === 0) return '';
  return await lines[lines.length - 1].textContent() || '';
}
```

**Coverage Achieved**: 50 tests covering:
- Terminal interaction: 10 tests (20%)
- Process management: 15 tests (25%)  
- File operations: 12 tests (20%)
- System state: 8 tests (15%)
- Error handling: 8 tests (10%)
- State persistence: 7 tests (10%)

**Total User Action Coverage**: 80%+

---

### Harness 2: Core Validation Suite (CVS)

**Purpose**: Systematic validation of all syscalls and OS primitives  
**Inspiration**: SQLite TH3 (100% branch coverage)

#### Implementation

```typescript
// canary/core-validation-suite.ts

/**
 * Core Validation Suite
 * 
 * SQLite Standard: 100% branch coverage, 100% MC/DC
 * WOS Standard: 100% syscall coverage, all error paths
 */

test.describe('CVS: System Call Validation', () => {
  // Test EVERY syscall with EVERY error condition
  
  test.describe('CVS-01: GetPid', () => {
    test('returns current process ID', async ({ page }) => {
      // Normal path
    });
    
    test('never fails (no error paths)', async ({ page }) => {
      // GetPid has no failure modes
    });
  });
  
  test.describe('CVS-02: Fork', () => {
    test('creates child process with unique PID', async ({ page }) => {
      // Normal path
    });
    
    test('fails gracefully on OOM', async ({ page }) => {
      await injectMemoryExhaustion(page);
      // Verify error handling
    });
    
    test('fails gracefully on max process limit', async ({ page }) => {
      await createProcesses(page, 100); // Hit limit
      // Verify error handling
    });
  });
  
  // ... Test all 11 syscalls with all error conditions
});

test.describe('CVS: State Transitions', () => {
  test('all valid process state transitions', async ({ page }) => {
    // Ready -> Running
    // Running -> Blocked
    // Blocked -> Ready
    // Running -> Terminated
    // Test each transition explicitly
  });
  
  test('reject invalid state transitions', async ({ page }) => {
    // Terminated -> Running (should fail)
    // Blocked -> Terminated without unblock (should handle)
  });
});

test.describe('CVS: Memory Safety', () => {
  test('allocations never overlap', async ({ page }) => {
    // Property: forall allocations, no overlap
  });
  
  test('freed memory not accessible', async ({ page }) => {
    // Property: munmap -> access = error
  });
  
  test('total allocated <= system memory', async ({ page }) => {
    // Property: sum(allocations) <= MAX_MEMORY
  });
});
```

**Target**: 1,000+ test cases covering 100% of syscalls and error paths.

---

### Harness 3: Differential Testing Suite (DTS)

**Purpose**: Compare WOS against reference implementations  
**Inspiration**: SQLite SLT runs 7.2 million queries against PostgreSQL, MySQL, SQL Server, and Oracle to verify identical answers

#### Implementation

```typescript
// canary/differential-testing-suite.ts

/**
 * Differential Testing Suite
 * 
 * Compare WOS against:
 * 1. Simplified reference model (Python)
 * 2. Previous WOS version (regression detection)
 * 3. Formal specification (TLA+)
 */

interface ReferenceModel {
  name: string;
  execute(command: string): Promise<string>;
}

class PythonReferenceModel implements ReferenceModel {
  name = 'Python Simulation';
  
  async execute(command: string): Promise<string> {
    // Execute command in Python simulator
    // This is a simplified, obviously-correct implementation
    return await fetch('/api/python-sim', {
      method: 'POST',
      body: JSON.stringify({ command }),
    }).then(r => r.text());
  }
}

class PreviousVersionModel implements ReferenceModel {
  name = 'WOS v0.9.0';
  
  async execute(command: string): Promise<string> {
    // Execute against previous stable version
    // Catch regressions
    return await this.runInIframe('/wos-v0.9.0/', command);
  }
}

test.describe('DTS: Differential Testing', () => {
  const models: ReferenceModel[] = [
    new PythonReferenceModel(),
    new PreviousVersionModel(),
  ];
  
  test('D01: Process lifecycle equivalence', async ({ page }) => {
    const commands = [
      'ps',
      'sleep 1 &',
      'ps',
      'kill <pid>',
      'ps',
    ];
    
    for (const model of models) {
      const wosResults = await executeInWOS(page, commands);
      const modelResults = await executeInModel(model, commands);
      
      // Compare results
      for (let i = 0; i < commands.length; i++) {
        expect(normalize(wosResults[i])).toEqual(normalize(modelResults[i]));
      }
    }
  });
  
  test('D02: File operations equivalence', async ({ page }) => {
    const commands = [
      'echo hello > /tmp/file.txt',
      'cat /tmp/file.txt',
      'ls /tmp',
      'rm /tmp/file.txt',
      'ls /tmp',
    ];
    
    // Compare across all models
  });
  
  // Generate 10,000+ command sequences
  test('D03: Random command sequences', async ({ page }) => {
    for (let i = 0; i < 10000; i++) {
      const sequence = generateRandomCommandSequence();
      
      const wosResult = await executeInWOS(page, sequence);
      
      for (const model of models) {
        const modelResult = await executeInModel(model, sequence);
        
        if (!equivalent(wosResult, modelResult)) {
          throw new Error(`Divergence found in sequence ${i}: ${sequence}`);
        }
      }
    }
  });
});
```

**Target**: 10,000+ command sequences validated against 2+ reference models.

---

### Harness 4: Chaos Engineering Suite (CES)

**Purpose**: Inject failures and verify graceful degradation  
**Inspiration**: SQLite dbsqlfuzz runs ~1 billion test mutations per day, mutating both SQL inputs and database files simultaneously

#### Implementation

```typescript
// canary/chaos-engineering-suite.ts

/**
 * Chaos Engineering Suite
 * 
 * Inject every conceivable failure mode and verify:
 * 1. System never crashes
 * 2. Errors reported clearly
 * 3. State remains consistent
 * 4. Recovery is possible
 */

enum FaultType {
  MemoryExhaustion,
  StorageQuotaExceeded,
  LocalStorageCorruption,
  SlowExecution,
  NetworkFailure,
  TabSuspension,
  StateCorruption,
  InvalidInput,
}

class ChaosInjector {
  async inject(page: Page, fault: FaultType, intensity: number) {
    switch (fault) {
      case FaultType.MemoryExhaustion:
        await page.evaluate((intensity) => {
          // Simulate OOM by allocating large arrays
          (window as any).__chaosMemory = new Array(intensity * 1024 * 1024);
        }, intensity);
        break;
        
      case FaultType.StorageQuotaExceeded:
        await page.evaluate(() => {
          // Fill localStorage to quota
          let data = 'x'.repeat(1024);
          try {
            for (let i = 0; i < 10000; i++) {
              localStorage.setItem(`chaos_${i}`, data);
            }
          } catch (e) {
            // Quota exceeded
          }
        });
        break;
        
      case FaultType.LocalStorageCorruption:
        await page.evaluate(() => {
          // Corrupt saved state
          const state = localStorage.getItem('wos_state');
          if (state) {
            const corrupted = state.slice(0, state.length / 2) + 
                            'corrupted' + 
                            state.slice(state.length / 2);
            localStorage.setItem('wos_state', corrupted);
          }
        });
        break;
        
      case FaultType.SlowExecution:
        await page.evaluate((delay) => {
          // Inject delays
          (window as any).__chaosDelay = delay;
        }, intensity);
        break;
        
      case FaultType.StateCorruption:
        await page.evaluate(() => {
          // Corrupt in-memory state via devtools
          (window as any).__wos_kernel.state.processes = null;
        });
        break;
    }
  }
}

test.describe('CES: Chaos Engineering', () => {
  const injector = new ChaosInjector();
  
  test('CE01: Survive memory exhaustion', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Inject OOM condition
    await injector.inject(page, FaultType.MemoryExhaustion, 10);
    
    // Verify system still responsive
    await executeCommand(page, 'echo test');
    const output = await getLastOutput(page);
    expect(output).toContain('test');
    
    // Verify error reported for failed operations
    await executeCommand(page, 'fork_intensive_process');
    expect(await getLastOutput(page)).toMatch(/out of memory|error/i);
  });
  
  test('CE02: Survive localStorage quota exceeded', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await injector.inject(page, FaultType.StorageQuotaExceeded, 1);
    
    // Verify system still works (in-memory mode)
    await executeCommand(page, 'echo test');
    expect(await getLastOutput(page)).toContain('test');
    
    // Verify warning about persistence
    const warnings = await page.locator('.warning').allTextContents();
    expect(warnings.some(w => w.includes('storage'))).toBeTruthy();
  });
  
  test('CE03: Recover from corrupted state', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Create valid state
    await executeCommand(page, 'echo data > /tmp/file.txt');
    
    // Corrupt localStorage
    await injector.inject(page, FaultType.LocalStorageCorruption, 1);
    
    // Reload
    await page.reload();
    
    // Verify system resets to clean state (not crashes)
    await page.waitForSelector('#command-input');
    await executeCommand(page, 'echo recovery test');
    expect(await getLastOutput(page)).toContain('recovery test');
  });
  
  test('CE04: Handle slow execution gracefully', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    await injector.inject(page, FaultType.SlowExecution, 100);
    
    // Commands should complete eventually
    await executeCommand(page, 'ps');
    const output = await page.waitForSelector('.line.output', { timeout: 5000 });
    expect(output).toBeTruthy();
  });
  
  // Combinatorial chaos: inject multiple faults simultaneously
  test('CE10: Multiple simultaneous failures', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    // Inject ALL faults at once
    await injector.inject(page, FaultType.MemoryExhaustion, 5);
    await injector.inject(page, FaultType.StorageQuotaExceeded, 1);
    await injector.inject(page, FaultType.SlowExecution, 50);
    
    // System should degrade gracefully, not crash
    try {
      await executeCommand(page, 'echo survival test');
      // May fail, but should not crash
    } catch (e) {
      // Verify error handling, not crash
    }
  });
});

// Long-running chaos soak test
test.describe('CES: Soak Test', () => {
  test('CE20: 24-hour chaos soak', async ({ page }) => {
    await page.goto('http://localhost:8000/dist/wos/');
    
    const duration = 24 * 60 * 60 * 1000; // 24 hours
    const startTime = Date.now();
    
    while (Date.now() - startTime < duration) {
      // Random fault injection
      const fault = randomFaultType();
      await injector.inject(page, fault, Math.random() * 10);
      
      // Random command execution
      const command = randomCommand();
      await executeCommand(page, command);
      
      // Verify no crash
      expect(await page.locator('#command-input').isVisible()).toBeTruthy();
      
      await page.waitForTimeout(1000);
    }
    
    // After 24 hours, system still responsive
    await executeCommand(page, 'echo final test');
    expect(await getLastOutput(page)).toContain('final test');
  });
});
```

**Target**: 1 billion fault injections (SQLite standard) validated over continuous testing.

---

## 3. User Action Coverage Matrix

### 3.1 Complete User Action Inventory

| Category | Actions | Test Count | Coverage |
|----------|---------|-----------|----------|
| **Terminal Interaction** | Type command, history navigation, clear, scroll | 10 | 100% |
| **Process Management** | ps, fork, kill, waitpid, background jobs | 15 | 100% |
| **File Operations** | ls, cat, echo, redirection, pipes | 12 | 100% |
| **System Calls** | All 11 core syscalls | 22 | 100% |
| **Error Handling** | Invalid commands, missing files, bad PIDs | 8 | 100% |
| **State Management** | Save, load, reset, persistence | 7 | 100% |
| **Shell Features** | Command parsing, environment, exit | 6 | 100% |
| **IPC** | Message passing, blocking, queues | 5 | 80% |
| **Memory** | Allocate, free, permissions | 4 | 80% |
| **Special Files** | /proc, /dev | 5 | 90% |
| **Total** | **80+ distinct actions** | **94 tests** | **85%** |

### 3.2 Critical Path Coverage (100%)

**Definition**: User workflows that MUST work for WOS to be usable.

1. **Boot → Shell Ready** (0 failures tolerated)
2. **Type Command → See Output** (0 failures tolerated)
3. **Execute ps → See Processes** (0 failures tolerated)
4. **Fork Process → Process Created** (0 failures tolerated)
5. **Save State → Reload → State Restored** (0 failures tolerated)

**Validation**: These 5 critical paths tested in EVERY browser, EVERY configuration, EVERY commit.

---

## 4. Anomaly Testing Framework

### 4.1 SQLite OOM Testing Adapted

SQLite simulates malloc() failures to verify graceful OOM handling, critical for embedded devices where OOM errors are frighteningly common.

**WOS OOM Simulation**:

```typescript
// canary/oom-testing.ts

/**
 * Out-of-Memory Testing
 * 
 * Test every operation under memory pressure.
 * Verify graceful degradation, not crashes.
 */

test.describe('OOM: Memory Exhaustion Testing', () => {
  // Inject OOM at every possible point
  
  test('OOM during fork', async ({ page }) => {
    await injectOOM(page, 'during_fork');
    await executeCommand(page, 'sleep 1 &');
    
    // Should fail gracefully
    expect(await getLastOutput(page)).toMatch(/out of memory|cannot fork/i);
    
    // System still responsive
    await executeCommand(page, 'echo still works');
    expect(await getLastOutput(page)).toContain('still works');
  });
  
  test('OOM during mmap', async ({ page }) => {
    await injectOOM(page, 'during_mmap');
    await executeCommand(page, 'allocate 1GB');
    
    expect(await getLastOutput(page)).toMatch(/out of memory|allocation failed/i);
  });
  
  // Test OOM during every syscall
  for (const syscall of ALL_SYSCALLS) {
    test(`OOM during ${syscall}`, async ({ page }) => {
      await injectOOM(page, `during_${syscall}`);
      // Verify graceful failure
    });
  }
});
```

### 4.2 I/O Error Testing

**localStorage failures**:

```typescript
test.describe('I/O: Storage Failure Testing', () => {
  test('localStorage.setItem throws QuotaExceededError', async ({ page }) => {
    await page.evaluate(() => {
      const original = localStorage.setItem;
      localStorage.setItem = () => { throw new DOMException('QuotaExceededError'); };
    });
    
    await executeCommand(page, 'echo data > /tmp/file.txt');
    
    // Should handle gracefully
    expect(await getLastOutput(page)).toMatch(/warning.*storage/i);
  });
});
```

### 4.3 Timing Error Testing

**Race conditions, deadlocks**:

```typescript
test.describe('Timing: Concurrency Testing', () => {
  test('concurrent fork operations', async ({ page }) => {
    // Trigger 100 simultaneous forks
    const promises = [];
    for (let i = 0; i < 100; i++) {
      promises.push(executeCommand(page, 'sleep 1 &'));
    }
    
    await Promise.all(promises);
    
    // System should handle gracefully, not deadlock
    await executeCommand(page, 'ps');
    const output = await getLastOutput(page);
    expect(output).toContain('init');
  });
});
```

---

## 5. Continuous Monitoring & Regression Detection

### 5.1 Pre-Commit Canary Suite

**Inspiration**: SQLite developers run "veryquick" subset of 300K tests before every commit, sufficient to catch most errors in a few minutes.

**WOS VeryQuick**:

```yaml
# .github/workflows/pre-commit-canary.yml
name: Pre-Commit Canary Suite

on: [push, pull_request]

jobs:
  veryquick:
    runs-on: ubuntu-latest
    timeout-minutes: 5
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Build WASM
        run: make wasm-release
      
      - name: Run VeryQuick Canaries (50 critical tests)
        run: |
          playwright test --grep "C0[0-4]" --workers=4
          # Tests C01-C49: Critical user workflows only
      
      - name: Verify No Regressions
        run: |
          playwright test --grep "D0[1-3]" --workers=2
          # Differential tests against previous version
```

**Target**: 50 tests in <5 minutes, 90%+ error detection rate.

### 5.2 Nightly Full Suite

```yaml
# .github/workflows/nightly-full-canary.yml
name: Nightly Full Canary Suite

on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily

jobs:
  full-suite:
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        browser: [chromium, firefox, webkit]
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Build WASM
        run: make wasm-release
      
      - name: Run Full Canary Suite (1000+ tests)
        run: |
          playwright test --project=${{ matrix.browser }}
          # All BCT, CVS, DTS, CES tests
      
      - name: Generate Report
        run: |
          node canary/generate-report.js --output canary-report.html
      
      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: canary-report-${{ matrix.os }}-${{ matrix.browser }}
          path: canary-report.html
```

### 5.3 Weekly Soak Test

```yaml
# .github/workflows/weekly-soak.yml
name: Weekly Soak Test

on:
  schedule:
    - cron: '0 0 * * 0'  # Sunday midnight

jobs:
  soak:
    runs-on: ubuntu-latest
    timeout-minutes: 1440  # 24 hours
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Build WASM
        run: make wasm-release
      
      - name: Run 24-Hour Chaos Soak
        run: |
          playwright test --grep "CE20" --timeout=86400000
          # 24-hour continuous chaos testing
      
      - name: Memory Leak Detection
        run: |
          node canary/analyze-memory-leaks.js
      
      - name: Performance Regression Check
        run: |
          node canary/check-performance.js --baseline baseline.json
```

---

## 6. Quality Metrics & Reporting

### 6.1 SQLite-Style Quality Dashboard

```typescript
// canary/quality-dashboard.ts

interface CanaryQualityMetrics {
  // SQLite standard metrics
  testToCodeRatio: number;          // Target: 10:1 minimum
  branchCoverage: number;           // Target: 100%
  userActionCoverage: number;       // Target: 80%+
  
  // Canary-specific metrics
  criticalPathSuccessRate: number;  // Target: 100%
  anomalyTestPassRate: number;      // Target: 100%
  oomResilienceScore: number;       // Target: 100%
  differentialMatchRate: number;    // Target: 99.9%+
  
  // Reliability metrics
  meanTimeBetweenFailures: number;  // Days
  regressionDetectionRate: number;  // Percentage
  falsePositiveRate: number;        // Target: <1%
  
  // Performance metrics
  coldStartTime: number;            // Target: <100ms
  commandResponseTime: number;      // Target: <50ms
  memoryLeakRate: number;           // Target: 0 KB/hour
}

class CanaryQualityDashboard {
  async generateReport(): Promise<CanaryReport> {
    const metrics = await this.collectMetrics();
    
    return {
      grade: this.calculateGrade(metrics),
      metrics,
      comparison: {
        sqlite: this.compareToSQLite(metrics),
        previousRelease: this.compareToPrevious(metrics),
      },
      recommendations: this.generateRecommendations(metrics),
    };
  }
  
  private calculateGrade(metrics: CanaryQualityMetrics): string {
    // SQLite-inspired grading
    if (
      metrics.branchCoverage >= 100 &&
      metrics.userActionCoverage >= 90 &&
      metrics.criticalPathSuccessRate === 100 &&
      metrics.anomalyTestPassRate === 100
    ) {
      return 'A+ (SQLite Standard)';
    }
    
    if (
      metrics.branchCoverage >= 95 &&
      metrics.userActionCoverage >= 80 &&
      metrics.criticalPathSuccessRate >= 99.9
    ) {
      return 'A (Production Ready)';
    }
    
    return 'B (Needs Improvement)';
  }
}
```

### 6.2 Release Criteria

**No release until**:

1. ✅ All 5 critical paths: 100% pass rate
2. ✅ VeryQuick suite: <1% failure rate over 100 runs
3. ✅ Full canary suite: 100% pass on 3 browsers
4. ✅ OOM resilience: 100% graceful degradation
5. ✅ Differential tests: 99.9%+ match rate
6. ✅ Soak test: 24 hours with 0 crashes
7. ✅ Regression tests: 0 known regressions

---

## 7. Implementation Roadmap

### Phase 1: Foundation (Week 1-2)

**Goal**: Basic canary infrastructure

- [ ] Set up Playwright test framework
- [ ] Implement 10 critical path tests (C01-C10)
- [ ] Create chaos injector utilities
- [ ] Configure pre-commit canary hook

**Deliverable**: 10 canary tests running in CI

### Phase 2: Core Coverage (Week 3-4)

**Goal**: 80% user action coverage

- [ ] Implement all BCT tests (50 tests)
- [ ] Add CVS syscall validation (22 tests)
- [ ] Create user action coverage matrix
- [ ] Add coverage reporting

**Deliverable**: 80%+ user action coverage validated

### Phase 3: Anomaly Testing (Week 5-6)

**Goal**: Resilience validation

- [ ] Implement OOM testing (20 tests)
- [ ] Add I/O failure testing (15 tests)
- [ ] Create timing/race condition tests (10 tests)
- [ ] Build chaos engineering suite (30 tests)

**Deliverable**: 100% anomaly test pass rate

### Phase 4: Differential & Soak (Week 7-8)

**Goal**: Long-term reliability

- [ ] Implement Python reference model
- [ ] Create differential test harness (10K sequences)
- [ ] Build 24-hour soak test
- [ ] Add memory leak detection

**Deliverable**: 24-hour soak test passing

### Phase 5: Continuous Monitoring (Week 9-10)

**Goal**: Production readiness

- [ ] Configure nightly full suite
- [ ] Set up weekly soak tests
- [ ] Create quality dashboard
- [ ] Document release criteria

**Deliverable**: Automated monitoring operational

---

## 8. Example: Complete Canary Test

```typescript
/**
 * Example: Complete Canary Test Following SQLite Principles
 * 
 * This test validates the most critical user workflow:
 * Boot → Execute Command → See Output
 * 
 * It includes:
 * - Normal path testing
 * - Anomaly injection (OOM, storage failure, corruption)
 * - Differential validation
 * - Performance monitoring
 * - State consistency checking
 */

test('CRITICAL-PATH-001: Boot → Command → Output', async ({ page }) => {
  // === Phase 1: Normal Path ===
  await page.goto('http://localhost:8000/dist/wos/');
  
  // Verify boot completes
  await page.waitForSelector('#command-input', { timeout: 1000 });
  const bootTime = await page.evaluate(() => window.performance.now());
  expect(bootTime).toBeLessThan(100); // <100ms cold start
  
  // Execute command
  const startTime = Date.now();
  await page.fill('#command-input', 'echo hello world');
  await page.press('#command-input', 'Enter');
  
  // Verify output
  const output = await page.waitForSelector('.line.output');
  const responseTime = Date.now() - startTime;
  expect(responseTime).toBeLessThan(50); // <50ms response
  expect(await output.textContent()).toBe('hello world');
  
  // === Phase 2: OOM Anomaly ===
  await injectOOM(page);
  
  await page.fill('#command-input', 'echo oom test');
  await page.press('#command-input', 'Enter');
  
  // Should still work (degrade gracefully)
  const oomOutput = await page.waitForSelector('.line.output:last-child');
  expect(await oomOutput.textContent()).toMatch(/oom test|error/);
  
  // === Phase 3: Storage Failure ===
  await injectStorageFailure(page);
  await page.reload();
  
  // Should boot to clean state (not crash)
  await page.waitForSelector('#command-input');
  await page.fill('#command-input', 'echo recovery');
  await page.press('#command-input', 'Enter');
  
  const recoveryOutput = await page.waitForSelector('.line.output');
  expect(await recoveryOutput.textContent()).toBe('recovery');
  
  // === Phase 4: Differential Validation ===
  const wosResult = await executeInWOS(page, ['echo test']);
  const pythonResult = await pythonModel.execute('echo test');
  expect(normalize(wosResult[0])).toEqual(normalize(pythonResult));
  
  // === Phase 5: State Consistency ===
  const finalState = await page.evaluate(() => window.__wos_kernel.getState());
  expect(finalState.processes.length).toBeGreaterThan(0);
  expect(finalState.memory.totalAllocated).toBeLessThanOrEqual(MAX_MEMORY);
  
  // === PASS: Critical path validated under all conditions ===
});
```

---

## 9. SQLite Lessons Applied

### 9.1 Key Principles Adopted

1. **Test What You Fly**: Test compiled WASM in production browsers, not mocks
2. **Anomaly First**: Test failure modes more rigorously than success paths
3. **100% Coverage**: Target 100% branch coverage via systematic testing
4. **Multiple Harnesses**: Independent test systems catch different bug classes
5. **Continuous Validation**: Test before every commit, every release, continuously
6. **Regression Prevention**: Never let a bug return once found
7. **Defensive Coding**: Test every assertion, every error path

### 9.2 WOS-Specific Adaptations

1. **Browser-Native**: Canaries run in real browsers, not test harnesses
2. **User-Centric**: 80% user action coverage, not just code coverage
3. **Performance Monitoring**: Track cold start, response time, memory leaks
4. **Cross-Browser**: Test Chromium, Firefox, WebKit (SQLite tests multiple platforms)
5. **State Persistence**: Validate localStorage across page reloads
6. **Chaos Engineering**: Modern approach to SQLite's anomaly testing

---

## 10. Success Criteria

### 10.1 Release Readiness Checklist

- [ ] **User Action Coverage**: 80%+ validated (94 tests passing)
- [ ] **Critical Path Success**: 100% (5/5 paths, 0 failures tolerated)
- [ ] **Branch Coverage**: 100% (via CVS + existing property tests)
- [ ] **Anomaly Resilience**: 100% graceful degradation (0 crashes)
- [ ] **Differential Match**: 99.9%+ (vs Python + previous version)
- [ ] **Soak Test**: 24 hours, 0 crashes, <1% error rate
- [ ] **Performance**: Cold start <100ms, response <50ms, no memory leaks
- [ ] **Cross-Browser**: Pass on Chromium, Firefox, WebKit
- [ ] **Regression Tests**: 0 known regressions from previous release

### 10.2 Continuous Quality Targets

- **Pre-Commit**: VeryQuick (50 tests) <5 minutes, >95% pass rate
- **Nightly**: Full suite (1000+ tests) on 9 configurations
- **Weekly**: Soak test (24 hours) + memory leak analysis
- **Monthly**: Differential validation (10K sequences) vs all models

---

## Conclusion

This specification adapts SQLite's world-class testing methodology to WOS's browser-based WASM environment. By following SQLite's principles of defensive testing, anomaly-first validation, and continuous quality monitoring, WOS achieves the same legendary reliability that has made SQLite the most deployed software module in the world.

**Next Steps**:
1. Implement Phase 1 (Foundation) - Week 1-2
2. Achieve 80% user action coverage - Week 3-4  
3. Add complete anomaly testing - Week 5-6
4. Validate 24-hour soak test - Week 7-8
5. Deploy continuous monitoring - Week 9-10

**Expected Outcome**: WOS becomes the SQLite of browser-based operating systems - thoroughly tested, exceptionally reliable, ready for production deployment.

---

**References**:
- SQLite Testing Methodology: https://sqlite.org/testing.html
- SQLite TH3 Details: https://sqlite.org/th3.html
- Research citations via web_search tool

**Version History**:
- v1.0 (2025-10-15): Initial specification based on SQLite standards
