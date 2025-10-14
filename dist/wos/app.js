// WOS Terminal Application
// Integrates WebAssembly kernel with HTML terminal interface

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
            }
            // Arrow up - previous command
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
            }
            // Arrow down - next command
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
            }
            // Ctrl+L - clear terminal
            else if (e.ctrlKey && e.key === 'l') {
                e.preventDefault();
                this.clear();
            }
        });

        // Button controls
        document.getElementById('btn-clear').addEventListener('click', () => this.clear());
        document.getElementById('btn-reset').addEventListener('click', () => this.reset());
        document.getElementById('btn-save').addEventListener('click', () => this.saveState());
        document.getElementById('btn-load').addEventListener('click', () => this.loadState());

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
        this.printWelcome();
    }

    async reset() {
        if (!this.wos) return;

        try {
            this.wos.reset();
            this.updateSystemInfo();
            this.printLine('System reset successfully', 'success');
        } catch (error) {
            this.printLine(`Reset error: ${error}`, 'error');
        }
    }

    async saveState() {
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

    async loadState() {
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

    async executeCommand(cmd) {
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
            const version = this.wos.version || 'Unknown';
            this.printLine(`WOS Version: ${version}`, 'output');
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

    setWOS(wos) {
        this.wos = wos;
        this.updateSystemInfo();
    }
}

// Initialize application
async function init() {
    const terminal = new Terminal();
    const statusElement = document.getElementById('status');
    const versionElement = document.getElementById('version');

    try {
        statusElement.innerHTML = '<span class="loading"></span> Loading WASM...';

        // For now, we'll create a mock WOS object until the WASM is properly built
        // In the full implementation, this would load the actual WASM module
        const mockWOS = {
            version: '0.1.0',
            processCount: () => 0,
            reset: () => {},
            getState: () => JSON.stringify({ processes: {} }),
            setState: (state) => {},
            executeCommand: (cmd) => `Command executed: ${cmd}\n(WASM integration pending)`
        };

        terminal.setWOS(mockWOS);

        statusElement.textContent = 'Ready';
        statusElement.className = '';
        versionElement.textContent = mockWOS.version;

        terminal.printLine('WASM kernel loaded successfully', 'success');
        terminal.printLine('Note: Full WASM integration pending - using mock interface', 'output');
        terminal.printLine('', 'output');

    } catch (error) {
        console.error('Initialization error:', error);
        statusElement.textContent = 'Error';
        statusElement.className = 'error';
        terminal.printLine(`Failed to initialize: ${error}`, 'error');
    }
}

// Start application when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
