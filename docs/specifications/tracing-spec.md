# WOS Application Tracing Specification

**Version**: 1.0.0
**Status**: Draft
**Author**: Claude Code
**Date**: 2025-10-20

## 1. Executive Summary

This specification defines a comprehensive tracing system for the WOS (WebAssembly Operating System) browser application. The tracing system provides runtime visibility into application initialization, WASM loading, state management, and user interactions, with minimal performance overhead when disabled.

## 2. Goals

### 2.1 Primary Goals
1. **Zero-cost when disabled**: Tracing should have negligible performance impact in production
2. **Comprehensive coverage**: Trace all critical application lifecycle events
3. **Easy debugging**: Enable/disable tracing via simple configuration
4. **Structured logging**: Consistent, parseable log format
5. **Production-safe**: Safe to enable in production for debugging

### 2.2 Non-Goals
- Real-time performance profiling (use browser DevTools)
- User behavior analytics (this is educational software)
- Error tracking/monitoring service integration

## 3. Architecture

### 3.1 Trace Levels

```typescript
enum TraceLevel {
  NONE = 0,    // No tracing
  ERROR = 1,   // Errors only
  WARN = 2,    // Warnings and errors
  INFO = 3,    // Informational messages (default for debugging)
  DEBUG = 4,   // Detailed debugging information
  TRACE = 5    // Very verbose, trace every function call
}
```

### 3.2 Trace Categories

```typescript
enum TraceCategory {
  INIT = 'INIT',           // Application initialization
  WASM = 'WASM',           // WASM loading and initialization
  CONFIG = 'CONFIG',       // Configuration management
  PANEL = 'PANEL',         // Panel management
  TERMINAL = 'TERMINAL',   // Terminal operations
  VIM = 'VIM',             // Vim editor
  PROCESS = 'PROCESS',     // Process management (from WASM)
  MEMORY = 'MEMORY',       // Memory operations
  SYSCALL = 'SYSCALL',     // System calls
  FILE = 'FILE',           // File operations
  EVENT = 'EVENT',         // User events (clicks, keyboard)
  RENDER = 'RENDER'        // DOM rendering/updates
}
```

### 3.3 Configuration

Tracing is controlled via:

1. **LocalStorage** (persistent across sessions):
   ```javascript
   localStorage.setItem('wos-trace-level', 'DEBUG');
   localStorage.setItem('wos-trace-categories', 'INIT,WASM,CONFIG');
   ```

2. **URL Parameters** (temporary, for single session):
   ```
   http://localhost:8001/wos/index.html?trace=DEBUG&categories=INIT,WASM
   ```

3. **Console Commands** (runtime):
   ```javascript
   WOS.trace.setLevel('DEBUG');
   WOS.trace.enableCategory('WASM');
   WOS.trace.disableCategory('EVENT');
   ```

## 4. Implementation

### 4.1 Tracer Class

```javascript
class Tracer {
  constructor() {
    this.level = TraceLevel.NONE;
    this.enabledCategories = new Set();
    this.startTime = performance.now();
    this.loadConfig();
  }

  loadConfig() {
    // Load from URL parameters first (highest priority)
    const urlParams = new URLSearchParams(window.location.search);
    if (urlParams.has('trace')) {
      this.level = TraceLevel[urlParams.get('trace').toUpperCase()] || TraceLevel.NONE;
    } else {
      // Fall back to localStorage
      const savedLevel = localStorage.getItem('wos-trace-level');
      if (savedLevel) {
        this.level = TraceLevel[savedLevel] || TraceLevel.NONE;
      }
    }

    // Load enabled categories
    const categories = urlParams.get('categories') ||
                      localStorage.getItem('wos-trace-categories') ||
                      '';
    if (categories) {
      categories.split(',').forEach(cat => {
        this.enabledCategories.add(cat.trim().toUpperCase());
      });
    } else if (this.level > TraceLevel.NONE) {
      // If tracing enabled but no categories specified, enable all
      Object.keys(TraceCategory).forEach(cat => {
        this.enabledCategories.add(cat);
      });
    }
  }

  shouldTrace(level, category) {
    if (this.level < level) return false;
    if (this.enabledCategories.size > 0 && !this.enabledCategories.has(category)) return false;
    return true;
  }

  log(level, category, message, data = null) {
    if (!this.shouldTrace(level, category)) return;

    const timestamp = (performance.now() - this.startTime).toFixed(2);
    const levelName = Object.keys(TraceLevel).find(k => TraceLevel[k] === level);
    const prefix = `[${timestamp}ms] [${category}] [${levelName}]`;

    if (data) {
      console.log(prefix, message, data);
    } else {
      console.log(prefix, message);
    }
  }

  // Convenience methods
  error(category, message, data) { this.log(TraceLevel.ERROR, category, message, data); }
  warn(category, message, data) { this.log(TraceLevel.WARN, category, message, data); }
  info(category, message, data) { this.log(TraceLevel.INFO, category, message, data); }
  debug(category, message, data) { this.log(TraceLevel.DEBUG, category, message, data); }
  trace(category, message, data) { this.log(TraceLevel.TRACE, category, message, data); }

  // Runtime configuration
  setLevel(level) {
    this.level = typeof level === 'string' ? TraceLevel[level.toUpperCase()] : level;
    localStorage.setItem('wos-trace-level', Object.keys(TraceLevel).find(k => TraceLevel[k] === this.level));
  }

  enableCategory(category) {
    this.enabledCategories.add(category.toUpperCase());
    localStorage.setItem('wos-trace-categories', Array.from(this.enabledCategories).join(','));
  }

  disableCategory(category) {
    this.enabledCategories.delete(category.toUpperCase());
    localStorage.setItem('wos-trace-categories', Array.from(this.enabledCategories).join(','));
  }

  reset() {
    this.level = TraceLevel.NONE;
    this.enabledCategories.clear();
    localStorage.removeItem('wos-trace-level');
    localStorage.removeItem('wos-trace-categories');
  }
}

// Global tracer instance
const tracer = new Tracer();
```

### 4.2 Integration Points

#### 4.2.1 Application Initialization

```javascript
async function initApp() {
  tracer.info('INIT', 'Application initialization started');
  const statusElement = document.getElementById('status');
  const versionElement = document.getElementById('version');

  try {
    tracer.debug('INIT', 'Setting status to Loading WASM...');
    statusElement.innerHTML = '<span class="loading"></span> Loading WASM...';

    tracer.info('WASM', 'Calling init()');
    const initStart = performance.now();
    await init();
    const initDuration = performance.now() - initStart;
    tracer.info('WASM', `init() completed in ${initDuration.toFixed(2)}ms`);

    tracer.debug('CONFIG', 'Creating ConfigManager');
    const configManager = new ConfigManager();

    tracer.debug('PANEL', 'Creating PanelManager');
    const panelManager = new PanelManager(configManager);

    tracer.debug('TERMINAL', 'Creating Terminal');
    const terminal = new Terminal(configManager);

    tracer.debug('INIT', 'Exposing terminal to window');
    window.terminalInstance = terminal;

    tracer.info('WASM', 'Creating WOS instance');
    const wos = new WosWasm();

    tracer.debug('TERMINAL', 'Setting WOS instance');
    terminal.setWOS(wos);

    tracer.debug('INIT', 'Setting status to Ready');
    statusElement.textContent = 'Ready';
    statusElement.className = '';

    tracer.debug('WASM', 'Getting version');
    const version = wos_version();
    tracer.info('INIT', `WOS version: ${version}`);
    versionElement.textContent = version;

    tracer.info('INIT', 'Application initialization complete');

  } catch (error) {
    tracer.error('INIT', 'Initialization failed', error);
    console.error('Initialization error:', error);
    statusElement.textContent = 'Error';
    statusElement.className = 'error';

    // Show error in UI
    const errorMsg = document.createElement('div');
    errorMsg.className = 'error-message';
    errorMsg.innerHTML = `
      <p>Failed to initialize WASM: ${error}</p>
      <p>This may happen if:</p>
      <ul>
        <li>WASM files are not present</li>
        <li>WASM is not supported in your browser</li>
        <li>Files are being served from file:// instead of http://</li>
      </ul>
    `;
    document.getElementById('terminal-output')?.appendChild(errorMsg);
  }
}
```

#### 4.2.2 ConfigManager

```javascript
class ConfigManager {
  constructor() {
    tracer.debug('CONFIG', 'ConfigManager constructor called');
    this.config = null;
    this.loadConfig();
  }

  loadConfig() {
    tracer.debug('CONFIG', 'Loading configuration');
    const savedConfig = localStorage.getItem('wos-config');
    if (savedConfig) {
      tracer.debug('CONFIG', 'Found saved config in localStorage');
      try {
        this.config = JSON.parse(savedConfig);
        tracer.info('CONFIG', 'Loaded config from localStorage', this.config);
      } catch (error) {
        tracer.warn('CONFIG', 'Error loading saved config, using default', error);
        this.loadDefaultConfig();
      }
    } else {
      tracer.debug('CONFIG', 'No saved config, using default');
      this.loadDefaultConfig();
    }
    this.applyConfig();
  }

  loadDefaultConfig() {
    tracer.debug('CONFIG', 'Loading default configuration');
    this.config = {
      version: "0.1.0",
      environment: "browser",
      ui: {
        theme: "dark",
        mode: "interactive",
        panels: {
          terminal: { visible: true, collapsed: false },
          process_list: { visible: true, collapsed: false },
          memory_map: { visible: true, collapsed: false },
          system_call_trace: { visible: true, collapsed: false },
          files: { visible: true, collapsed: false },
          system_info: { visible: true, collapsed: false },
          system_monitor: { visible: true, collapsed: false }
        }
      }
    };
    tracer.trace('CONFIG', 'Default config created', this.config);
  }

  applyConfig() {
    tracer.debug('CONFIG', 'Applying configuration');
    if (!this.config || !this.config.ui) {
      tracer.warn('CONFIG', 'No config or UI config to apply');
      return;
    }

    const theme = this.config.ui.theme || 'auto';
    tracer.debug('CONFIG', `Applying theme: ${theme}`);

    if (theme === 'dark') {
      document.body.classList.add('theme-dark');
      document.body.classList.remove('theme-light');
    } else if (theme === 'light') {
      document.body.classList.add('theme-light');
      document.body.classList.remove('theme-dark');
    } else {
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
      tracer.debug('CONFIG', `Auto theme detected: ${prefersDark ? 'dark' : 'light'}`);
      document.body.classList.toggle('theme-dark', prefersDark);
      document.body.classList.toggle('theme-light', !prefersDark);
    }
  }
}
```

#### 4.2.3 PanelManager

```javascript
class PanelManager {
  constructor(configManager) {
    tracer.debug('PANEL', 'PanelManager constructor called');
    this.configManager = configManager;
    this.panels = {};
    this.initializePanels();
    this.setupEventListeners();
  }

  initializePanels() {
    tracer.debug('PANEL', 'Initializing panels');
    const config = this.configManager.getConfig();
    if (!config || !config.ui || !config.ui.panels) {
      tracer.warn('PANEL', 'No panel configuration available');
      return;
    }

    const savedState = localStorage.getItem('wos_panel_state');
    const panelState = savedState ? JSON.parse(savedState) : {};
    tracer.debug('PANEL', 'Loaded panel state from localStorage', panelState);

    const panelElements = document.querySelectorAll('[data-panel]');
    tracer.info('PANEL', `Found ${panelElements.length} panels in DOM`);

    panelElements.forEach(panelEl => {
      const panelName = panelEl.dataset.panel;
      tracer.trace('PANEL', `Initializing panel: ${panelName}`);

      const panelConfig = config.ui.panels[panelName];
      if (panelConfig) {
        if (panelState[panelName]) {
          panelConfig.visible = panelState[panelName].visible;
          panelConfig.collapsed = panelState[panelName].collapsed;
          tracer.trace('PANEL', `Panel ${panelName} state restored`, panelState[panelName]);
        }

        this.panels[panelName] = {
          element: panelEl,
          config: panelConfig
        };

        if (panelConfig.visible === false) {
          panelEl.style.display = 'none';
          tracer.debug('PANEL', `Panel ${panelName} hidden`);
        }

        if (panelConfig.collapsed) {
          this.collapsePanel(panelName);
        }
      }
    });

    tracer.info('PANEL', `Initialized ${Object.keys(this.panels).length} panels`);
  }

  collapsePanel(panelName) {
    tracer.debug('PANEL', `Collapsing panel: ${panelName}`);
    const panel = this.panels[panelName];
    if (!panel) {
      tracer.warn('PANEL', `Panel not found: ${panelName}`);
      return;
    }

    const content = panel.element.querySelector('.panel-content');
    if (content) {
      content.style.display = 'none';
      panel.element.classList.add('collapsed');
      panel.config.collapsed = true;
      this.savePanelState();
      tracer.trace('PANEL', `Panel ${panelName} collapsed`);
    }
  }

  expandPanel(panelName) {
    tracer.debug('PANEL', `Expanding panel: ${panelName}`);
    const panel = this.panels[panelName];
    if (!panel) {
      tracer.warn('PANEL', `Panel not found: ${panelName}`);
      return;
    }

    const content = panel.element.querySelector('.panel-content');
    if (content) {
      content.style.display = '';
      panel.element.classList.remove('collapsed');
      panel.config.collapsed = false;
      this.savePanelState();
      tracer.trace('PANEL', `Panel ${panelName} expanded`);
    }
  }

  savePanelState() {
    const state = {};
    Object.keys(this.panels).forEach(name => {
      state[name] = {
        visible: this.panels[name].config.visible,
        collapsed: this.panels[name].config.collapsed
      };
    });
    localStorage.setItem('wos_panel_state', JSON.stringify(state));
    tracer.trace('PANEL', 'Panel state saved', state);
  }
}
```

## 5. Usage Examples

### 5.1 Enable Full Tracing for Debugging

```bash
# Via URL
http://localhost:8001/wos/index.html?trace=DEBUG

# Via console
localStorage.setItem('wos-trace-level', 'DEBUG');
location.reload();
```

### 5.2 Trace Only WASM and Config

```bash
# Via URL
http://localhost:8001/wos/index.html?trace=DEBUG&categories=WASM,CONFIG

# Via console
localStorage.setItem('wos-trace-level', 'DEBUG');
localStorage.setItem('wos-trace-categories', 'WASM,CONFIG');
location.reload();
```

### 5.3 Trace Specific Issue

For the current E2E test failure investigation:

```bash
http://localhost:8001/wos/index.html?trace=DEBUG&categories=INIT,WASM,CONFIG
```

Expected output:
```
[0.00ms] [INIT] [INFO] Application initialization started
[0.15ms] [INIT] [DEBUG] Setting status to Loading WASM...
[0.18ms] [WASM] [INFO] Calling init()
[245.32ms] [WASM] [INFO] init() completed in 245.14ms
[245.45ms] [CONFIG] [DEBUG] ConfigManager constructor called
[245.48ms] [CONFIG] [DEBUG] Loading configuration
[245.51ms] [CONFIG] [DEBUG] No saved config, using default
[245.54ms] [CONFIG] [DEBUG] Loading default configuration
[245.58ms] [CONFIG] [TRACE] Default config created {version: "0.1.0", ...}
[245.62ms] [CONFIG] [DEBUG] Applying configuration
[245.65ms] [CONFIG] [DEBUG] Applying theme: dark
[246.01ms] [PANEL] [DEBUG] PanelManager constructor called
...
[350.12ms] [INIT] [DEBUG] Setting status to Ready
[350.25ms] [INIT] [INFO] WOS version: 0.1.0-alpha
[350.28ms] [INIT] [INFO] Application initialization complete
```

### 5.4 Disable Tracing

```bash
# Via console
localStorage.removeItem('wos-trace-level');
localStorage.removeItem('wos-trace-categories');
location.reload();

# Or use tracer API
tracer.reset();
location.reload();
```

## 6. Performance Considerations

### 6.1 Zero-Cost When Disabled

When `tracer.level = TraceLevel.NONE`, the `shouldTrace()` check returns immediately:

```javascript
shouldTrace(level, category) {
  if (this.level < level) return false;  // Fast path: single integer comparison
  // ... rest of function never executes
}
```

This ensures minimal overhead when tracing is disabled (production default).

### 6.2 Structured Data

For complex data structures, use the `data` parameter instead of string concatenation:

```javascript
// GOOD: Data serialization only happens if tracing is enabled
tracer.debug('CONFIG', 'Config loaded', this.config);

// BAD: JSON.stringify happens even when tracing is disabled
tracer.debug('CONFIG', `Config loaded: ${JSON.stringify(this.config)}`);
```

## 7. Testing

### 7.1 Manual Testing Checklist

- [ ] Verify tracing disabled by default (no console output)
- [ ] Enable via URL parameter and verify output
- [ ] Enable via localStorage and verify persistence across reload
- [ ] Test each trace level (ERROR, WARN, INFO, DEBUG, TRACE)
- [ ] Test category filtering
- [ ] Test runtime configuration via console
- [ ] Verify performance with tracing disabled
- [ ] Verify all initialization steps are traced

### 7.2 E2E Test Integration

Playwright tests can enable tracing by navigating to:

```typescript
await page.goto('index.html?trace=DEBUG&categories=INIT,WASM');
```

Console output can be captured:

```typescript
page.on('console', msg => {
  if (msg.text().includes('[INIT]') || msg.text().includes('[WASM]')) {
    console.log(msg.text());
  }
});
```

## 8. Future Enhancements

### 8.1 Performance Metrics

```javascript
tracer.measure('WASM_INIT', startTime, endTime);
tracer.reportMetrics(); // Print performance summary
```

### 8.2 Trace Export

```javascript
tracer.export('json'); // Export trace log as JSON for analysis
tracer.export('har');  // Export as HAR format for browser tools
```

### 8.3 Trace Filtering

```javascript
// Filter by time range
tracer.filter({ startTime: 0, endTime: 1000 });

// Filter by message pattern
tracer.filter({ pattern: /init|wasm/i });
```

## 9. References

- [Console API](https://developer.mozilla.org/en-US/docs/Web/API/Console)
- [Performance API](https://developer.mozilla.org/en-US/docs/Web/API/Performance)
- [User Timing API](https://developer.mozilla.org/en-US/docs/Web/API/User_Timing_API)

## 10. Appendix: Complete Example

See `dist/wos/app.js` for the complete implementation integrating this tracing specification into the WOS application.
