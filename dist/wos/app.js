// WOS Terminal Application
// Integrates WebAssembly kernel with HTML terminal interface

import init, { wos_version, WosWasm } from './wos.js';

class Terminal {
  constructor() {
    this.output = document.getElementById('terminal-output');
    this.input = document.getElementById('terminal-input');
    this.terminalElement = document.getElementById('terminal');
    this.history = [];
    this.historyIndex = -1;
    this.wos = null;

    this.setupEventListeners();
    this.printWelcome();
  }

  setupEventListeners() {
    // Enter key - execute command
    this.input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter') {
        e.preventDefault();
        this.executeCommand(this.input.value.trim());
        this.input.value = '';
      } // Arrow up - previous command
      else if (e.key === 'ArrowUp') {
        e.preventDefault();
        if (this.history.length > 0) {
          if (this.historyIndex === -1) {
            this.historyIndex = this.history.length - 1;
          } else if (this.historyIndex > 0) {
            this.historyIndex--;
          }
          this.input.value = this.history[this.historyIndex];
        }
      } // Arrow down - next command
      else if (e.key === 'ArrowDown') {
        e.preventDefault();
        if (this.historyIndex !== -1) {
          this.historyIndex++;
          if (this.historyIndex >= this.history.length) {
            this.historyIndex = -1;
            this.input.value = '';
          } else {
            this.input.value = this.history[this.historyIndex];
          }
        }
      } // Ctrl+L - clear terminal
      else if (e.ctrlKey && (e.key === 'l' || e.key === 'L')) {
        e.preventDefault();
        this.clear();
      }
    });

    // Button controls
    document.getElementById('btn-clear').addEventListener('click', () => this.clear());
    document.getElementById('btn-reset').addEventListener('click', () => this.reset());
    document.getElementById('btn-save').addEventListener('click', () => this.saveState());
    document.getElementById('btn-load').addEventListener('click', () => this.loadState());

    // Quality export buttons
    document.getElementById('btn-export-json').addEventListener(
      'click',
      () => this.exportQualityJson(),
    );
    document.getElementById('btn-export-html').addEventListener(
      'click',
      () => this.exportQualityHtml(),
    );
    document.getElementById('btn-export-md').addEventListener(
      'click',
      () => this.exportQualityMarkdown(),
    );
    document.getElementById('btn-export-sarif').addEventListener(
      'click',
      () => this.exportQualitySarif(),
    );

    // Keep input focused
    this.terminalElement.addEventListener('click', () => {
      this.input.focus();
    });
  }

  printWelcome() {
    this.printLine('WOS - WebAssembly Operating System', 'success');
    this.printLine('Educational microkernel v0.1.0', 'output');
    this.printLine('', 'output');
    this.printLine('Type "help" for available commands', 'output');
    this.printLine('', 'output');
  }

  printLine(text, className = 'output') {
    const line = document.createElement('div');
    line.className = `terminal-line ${className}`;
    line.textContent = text;
    this.output.appendChild(line);
    this.scrollToBottom();
  }

  printCommand(cmd) {
    this.printLine(`wos$ ${cmd}`, 'command');
  }

  scrollToBottom() {
    this.terminalElement.scrollTop = this.terminalElement.scrollHeight;
  }

  clear() {
    this.output.innerHTML = '';
  }

  reset() {
    if (!this.wos) return;

    try {
      this.wos.reset();
      this.updateSystemInfo();
      this.printLine('System reset successfully', 'success');
    } catch (error) {
      this.printLine(`Reset error: ${error}`, 'error');
    }
  }

  saveState() {
    if (!this.wos) {
      this.printLine('WASM not initialized', 'error');
      return;
    }

    try {
      const state = this.wos.getState();
      localStorage.setItem('wos-state', state);
      this.printLine('State saved to localStorage', 'success');
    } catch (error) {
      this.printLine(`Save error: ${error}`, 'error');
    }
  }

  loadState() {
    if (!this.wos) {
      this.printLine('WASM not initialized', 'error');
      return;
    }

    try {
      const state = localStorage.getItem('wos-state');
      if (!state) {
        this.printLine('No saved state found', 'error');
        return;
      }

      this.wos.setState(state);
      this.updateSystemInfo();
      this.printLine('State loaded from localStorage', 'success');
    } catch (error) {
      this.printLine(`Load error: ${error}`, 'error');
    }
  }

  executeCommand(cmd) {
    if (!cmd) return;

    // Add to history
    if (this.history.length === 0 || this.history[this.history.length - 1] !== cmd) {
      this.history.push(cmd);
    }
    this.historyIndex = -1;

    // Print command
    this.printCommand(cmd);

    // Handle built-in commands
    if (cmd === 'help') {
      this.printHelp();
      return;
    }

    if (cmd === 'clear' || cmd === 'cls') {
      this.clear();
      return;
    }

    if (cmd === 'history') {
      this.printHistory();
      return;
    }

    if (cmd === 'version') {
      this.printVersion();
      return;
    }

    // Execute via WASM if available
    if (this.wos) {
      try {
        const result = this.wos.executeCommand(cmd);
        this.printLine(result, 'output');
        this.updateSystemInfo();
      } catch (error) {
        this.printLine(`Error: ${error}`, 'error');
      }
    } else {
      this.printLine('WASM not initialized - command not executed', 'error');
    }
  }

  printHelp() {
    this.printLine('Available commands:', 'output');
    this.printLine('  help      - Show this help message', 'output');
    this.printLine('  clear     - Clear terminal', 'output');
    this.printLine('  history   - Show command history', 'output');
    this.printLine('  version   - Show OS version', 'output');
    this.printLine('', 'output');
    this.printLine('Keyboard shortcuts:', 'output');
    this.printLine('  ↑/↓       - Navigate command history', 'output');
    this.printLine('  Ctrl+L    - Clear terminal', 'output');
    this.printLine('', 'output');
  }

  printHistory() {
    if (this.history.length === 0) {
      this.printLine('No command history', 'output');
      return;
    }

    this.history.forEach((cmd, i) => {
      this.printLine(`  ${i + 1}  ${cmd}`, 'output');
    });
  }

  printVersion() {
    if (this.wos) {
      const version = wos_version();
      this.printLine(version, 'output');
    } else {
      this.printLine('WOS Version: Not loaded', 'error');
    }
  }

  updateSystemInfo() {
    if (!this.wos) return;

    try {
      const processCount = this.wos.processCount();
      document.getElementById('process-count').textContent = processCount;
    } catch (error) {
      console.error('Error updating system info:', error);
    }
  }

  updateQualityMetrics() {
    if (!this.wos || !this.wos.getQualityMetrics) return;

    try {
      const metricsJson = this.wos.getQualityMetrics();
      const metrics = JSON.parse(metricsJson);

      // Update TDG grade with color coding
      const gradeElement = document.getElementById('tdg-grade');
      gradeElement.textContent = metrics.tdg_grade;
      gradeElement.className = 'grade-value grade-' +
        metrics.tdg_grade.toLowerCase().replace('+', '-plus');

      // Update TDG score
      document.getElementById('tdg-score').textContent = `${metrics.tdg_score.toFixed(1)}%`;

      // Update test count
      const testCountElement = document.getElementById('test-count');
      testCountElement.textContent = metrics.test_count;
      testCountElement.className = 'metric-value good';

      // Update coverage
      const coverageElement = document.getElementById('coverage');
      coverageElement.textContent = `${metrics.coverage.toFixed(1)}%`;
      coverageElement.className = metrics.coverage >= 85.0
        ? 'metric-value good'
        : 'metric-value warning';

      // Update complexity
      const complexityElement = document.getElementById('complexity');
      complexityElement.textContent = `${metrics.max_complexity} / ${
        metrics.avg_complexity.toFixed(1)
      }`;
      complexityElement.className = metrics.max_complexity <= 20
        ? 'metric-value good'
        : 'metric-value warning';

      // Update SATD count
      const satdElement = document.getElementById('satd-count');
      satdElement.textContent = metrics.satd_count;
      satdElement.className = metrics.satd_count === 0 ? 'metric-value good' : 'metric-value error';
    } catch (error) {
      console.error('Error updating quality metrics:', error);
    }
  }

  exportQualityJson() {
    if (!this.wos || !this.wos.getQualityMetrics) {
      this.printLine('WASM not initialized', 'error');
      return;
    }

    try {
      const metricsJson = this.wos.getQualityMetrics();
      this.downloadFile('wos-quality-metrics.json', metricsJson, 'application/json');
      this.printLine('Quality metrics exported to JSON', 'success');
    } catch (error) {
      this.printLine(`Export error: ${error}`, 'error');
    }
  }

  exportQualityHtml() {
    if (!this.wos || !this.wos.exportQualityHtml) {
      this.printLine('WASM not initialized', 'error');
      return;
    }

    try {
      const html = this.wos.exportQualityHtml();
      this.downloadFile('wos-quality-report.html', html, 'text/html');
      this.printLine('Quality report exported to HTML', 'success');
    } catch (error) {
      this.printLine(`Export error: ${error}`, 'error');
    }
  }

  exportQualityMarkdown() {
    if (!this.wos || !this.wos.exportQualityMarkdown) {
      this.printLine('WASM not initialized', 'error');
      return;
    }

    try {
      const markdown = this.wos.exportQualityMarkdown();
      this.downloadFile('wos-quality-report.md', markdown, 'text/markdown');
      this.printLine('Quality report exported to Markdown', 'success');
    } catch (error) {
      this.printLine(`Export error: ${error}`, 'error');
    }
  }

  exportQualitySarif() {
    if (!this.wos || !this.wos.exportQualitySarif) {
      this.printLine('WASM not initialized', 'error');
      return;
    }

    try {
      const sarif = this.wos.exportQualitySarif();
      this.downloadFile('wos-quality-report.sarif', sarif, 'application/json');
      this.printLine('Quality report exported to SARIF format', 'success');
    } catch (error) {
      this.printLine(`Export error: ${error}`, 'error');
    }
  }

  downloadFile(filename, content, mimeType) {
    const blob = new Blob([content], { type: mimeType });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = filename;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  setWOS(wos) {
    this.wos = wos;
    this.updateSystemInfo();
    this.updateQualityMetrics();
  }
}

// Initialize application
async function initApp() {
  const terminal = new Terminal();
  const statusElement = document.getElementById('status');
  const versionElement = document.getElementById('version');

  try {
    statusElement.innerHTML = '<span class="loading"></span> Loading WASM...';

    // Initialize WASM module
    await init();

    // Create WOS instance
    const wos = new WosWasm();

    terminal.setWOS(wos);

    statusElement.textContent = 'Ready';
    statusElement.className = '';

    // Get and display version
    const version = wos_version();
    versionElement.textContent = version;

    terminal.printLine('WASM kernel loaded successfully', 'success');
    terminal.printLine(version, 'output');
    terminal.printLine('', 'output');
    terminal.printLine('Type "help" for available commands', 'output');
    terminal.printLine('', 'output');
  } catch (error) {
    console.error('Initialization error:', error);
    statusElement.textContent = 'Error';
    statusElement.className = 'error';
    terminal.printLine(`Failed to initialize WASM: ${error}`, 'error');
    terminal.printLine('', 'output');
    terminal.printLine('This may happen if:', 'output');
    terminal.printLine('- WASM files are not present', 'output');
    terminal.printLine('- WASM is not supported in your browser', 'output');
    terminal.printLine('- Files are being served from file:// instead of http://', 'output');
  }
}

// Start application when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}
