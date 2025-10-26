// WOS Terminal Application
// Integrates WebAssembly kernel with HTML terminal interface

import init, {
  wos_version,
  WosWasm,
  getDefaultConfig,
  loadConfigFromYaml,
  loadConfigFromYamlWithFallback
} from './wos.js';

// Tracing System - for debugging and performance analysis
const TraceLevel = {
  NONE: 0,
  ERROR: 1,
  WARN: 2,
  INFO: 3,
  DEBUG: 4,
  TRACE: 5
};

const TraceCategory = {
  INIT: 'INIT',
  WASM: 'WASM',
  CONFIG: 'CONFIG',
  PANEL: 'PANEL',
  TERMINAL: 'TERMINAL',
  VIM: 'VIM',
  PROCESS: 'PROCESS',
  MEMORY: 'MEMORY',
  SYSCALL: 'SYSCALL',
  FILE: 'FILE',
  EVENT: 'EVENT',
  RENDER: 'RENDER'
};

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
      const traceLevelStr = urlParams.get('trace').toUpperCase();
      this.level = TraceLevel[traceLevelStr] || TraceLevel.NONE;
    } else {
      // Fall back to localStorage
      const savedLevel = localStorage.getItem('wos-trace-level');
      if (savedLevel) {
        this.level = TraceLevel[savedLevel] || TraceLevel.NONE;
      }
    }

    // Load categories from URL parameters
    if (urlParams.has('categories')) {
      const categories = urlParams.get('categories').split(',');
      categories.forEach(cat => this.enabledCategories.add(cat.toUpperCase()));
    } else {
      // Fall back to localStorage
      const savedCategories = localStorage.getItem('wos-trace-categories');
      if (savedCategories) {
        const categories = savedCategories.split(',');
        categories.forEach(cat => this.enabledCategories.add(cat.toUpperCase()));
      }
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

  error(category, message, data) { this.log(TraceLevel.ERROR, category, message, data); }
  warn(category, message, data) { this.log(TraceLevel.WARN, category, message, data); }
  info(category, message, data) { this.log(TraceLevel.INFO, category, message, data); }
  debug(category, message, data) { this.log(TraceLevel.DEBUG, category, message, data); }
  trace(category, message, data) { this.log(TraceLevel.TRACE, category, message, data); }

  setLevel(level) {
    this.level = TraceLevel[level] || TraceLevel.NONE;
    localStorage.setItem('wos-trace-level', level);
  }

  setCategories(categories) {
    this.enabledCategories.clear();
    categories.forEach(cat => this.enabledCategories.add(cat.toUpperCase()));
    localStorage.setItem('wos-trace-categories', categories.join(','));
  }

  clear() {
    this.level = TraceLevel.NONE;
    this.enabledCategories.clear();
    localStorage.removeItem('wos-trace-level');
    localStorage.removeItem('wos-trace-categories');
  }
}

// Global tracer instance
const tracer = new Tracer();
window.tracer = tracer; // Expose for console access

// Monaco Editor Integration
let monacoEditor = null;
let currentEditingFile = null;
let currentWosInstance = null;  // Store wos instance for save functionality

function initMonacoEditor(callback) {
  tracer.debug('MONACO', 'initMonacoEditor called');

  if (typeof window.require === 'undefined') {
    tracer.error('MONACO', 'RequireJS not available');
    if (callback) callback(new Error('RequireJS not available'));
    return;
  }

  window.require(['vs/editor/editor.main'], function(monaco) {
    tracer.info('MONACO', 'Monaco editor loaded successfully');
    window.monaco = monaco;
    if (callback) callback(null);
  }, function(err) {
    tracer.error('MONACO', 'Failed to load Monaco editor', err);
    if (callback) callback(err);
  });
}

function getLanguageFromFilename(filename) {
  const ext = filename.split('.').pop().toLowerCase();
  const languageMap = {
    'rs': 'rust',
    'sh': 'shell',
    'bash': 'shell',
    'md': 'markdown',
    'yaml': 'yaml',
    'yml': 'yaml',
    'json': 'json',
    'js': 'javascript',
    'ts': 'typescript',
    'html': 'html',
    'css': 'css',
    'txt': 'plaintext',
  };
  return languageMap[ext] || 'plaintext';
}

function openMonacoEditor(filename, content, wosInstance) {
  tracer.info('MONACO', 'Opening Monaco editor for file: ' + filename);

  const container = document.getElementById('monaco-editor-container');
  if (!container) {
    tracer.error('MONACO', 'Monaco editor container not found');
    return;
  }

  // Store wos instance globally for save functionality
  currentWosInstance = wosInstance;

  initMonacoEditor(function(err) {
    if (err) {
      tracer.error('MONACO', 'Failed to initialize Monaco editor', err);
      return;
    }

    if (!monacoEditor) {
      monacoEditor = window.monaco.editor.create(container, {
        value: content,
        language: getLanguageFromFilename(filename),
        theme: 'vs-dark',
        automaticLayout: true,
        minimap: { enabled: true },
        fontSize: 16,
        lineNumbers: 'on',
        scrollBeyondLastLine: false,
        wordWrap: 'on',
        accessibilitySupport: 'on',
        ariaLabel: 'Editing file: ' + filename,
        multiCursorModifier: 'ctrlCmd',
        quickSuggestions: true,
        suggestOnTriggerCharacters: true,
      });

      window.monacoEditor = monacoEditor;

      // Add Escape key handler to close editor - uses currentWosInstance
      monacoEditor.addCommand(window.monaco.KeyCode.Escape, function() {
        closeMonacoEditor(true);
      });

      tracer.debug('MONACO', 'Monaco editor instance created with Escape key handler');
    } else {
      monacoEditor.setValue(content);
      window.monaco.editor.setModelLanguage(monacoEditor.getModel(), getLanguageFromFilename(filename));
      tracer.debug('MONACO', 'Monaco editor updated with new content');
    }

    currentEditingFile = filename;
    container.style.display = 'block';
    monacoEditor.focus();
    tracer.info('MONACO', 'Monaco editor opened and focused');
  });
}

function closeMonacoEditor(save) {
  tracer.info('MONACO', 'Closing Monaco editor, save=' + save);

  const container = document.getElementById('monaco-editor-container');
  if (!container) {
    tracer.error('MONACO', 'Monaco editor container not found');
    return;
  }

  if (save && monacoEditor && currentEditingFile && currentWosInstance) {
    const content = monacoEditor.getValue();
    try {
      // Write file using echo command with proper escaping
      const escapedContent = content.replace(/\\/g, '\\\\').replace(/"/g, '\\"').replace(/\$/g, '\\$').replace(/`/g, '\\`');
      currentWosInstance.executeCommand(`echo "${escapedContent}" > ${currentEditingFile}`);
      tracer.info('MONACO', 'File saved: ' + currentEditingFile);
    } catch (err) {
      tracer.error('MONACO', 'Failed to save file', err);
    }
  }

  container.style.display = 'none';
  currentEditingFile = null;

  const terminalInput = document.getElementById('terminal-input');
  if (terminalInput) {
    terminalInput.focus();
  }
  tracer.debug('MONACO', 'Monaco editor closed, focus returned to terminal');
}

class ConfigManager {
  constructor() {
    tracer.debug('CONFIG', 'ConfigManager constructor called');
    this.config = null;
    this.loadConfig();
    tracer.debug('CONFIG', 'ConfigManager initialized', this.config);
  }

  loadConfig() {
    tracer.debug('CONFIG', 'Loading configuration');
    const savedConfig = localStorage.getItem('wos-config');
    if (savedConfig) {
      tracer.debug('CONFIG', 'Found saved config in localStorage');
      try {
        // Don't call WASM functions - just parse JSON directly
        this.config = JSON.parse(savedConfig);
        tracer.info('CONFIG', 'Loaded saved configuration');
      } catch (error) {
        tracer.error('CONFIG', 'Error loading saved config, using default', error);
        console.error('Error loading saved config, using default:', error);
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
    // Don't call WASM functions before init() - use hardcoded default
    this.config = {
      version: "0.1.0",
      environment: "browser",
      ui: {
        theme: "dark",
        mode: "interactive",
        panels: {
          terminal: { visible: true, collapsed: false },
          process_list: { visible: true, collapsed: true }, // WOS-306: Collapsed for progressive disclosure
          memory_map: { visible: true, collapsed: true }, // WOS-306: Collapsed for progressive disclosure
          syscall_trace: { visible: true, collapsed: true }, // WOS-306: Collapsed for progressive disclosure
          filesystem: { visible: true, collapsed: true }, // WOS-306: Collapsed for progressive disclosure
          system_monitor: { visible: true, collapsed: true }, // WOS-306: Collapsed for progressive disclosure
          system_monitor_detailed: { visible: true, collapsed: true }, // WOS-306: Collapsed by default for progressive disclosure
          time_travel_debugger: { visible: true, collapsed: true }, // WOS-306: Collapsed by default for progressive disclosure
          help: { visible: true, collapsed: true } // WOS-306: Collapsed for progressive disclosure
        }
      }
    };
    tracer.info('CONFIG', 'Default configuration loaded');
  }

  saveConfig(yamlConfig) {
    tracer.debug('CONFIG', 'Saving configuration');
    localStorage.setItem('wos-config', yamlConfig);
    this.config = JSON.parse(loadConfigFromYamlWithFallback(yamlConfig));
    this.applyConfig();
    tracer.info('CONFIG', 'Configuration saved');
  }

  applyConfig() {
    tracer.debug('CONFIG', 'Applying configuration');
    if (!this.config || !this.config.ui) {
      tracer.warn('CONFIG', 'Cannot apply config - config or ui is null');
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
      document.body.classList.toggle('theme-dark', prefersDark);
      document.body.classList.toggle('theme-light', !prefersDark);
    }
    tracer.debug('CONFIG', 'Configuration applied');
  }

  getConfig() {
    return this.config;
  }
}

class PanelManager {
  constructor(configManager) {
    this.configManager = configManager;
    this.panels = {};
    this.initializePanels();
    this.setupEventListeners();
  }

  initializePanels() {
    const config = this.configManager.getConfig();
    if (!config || !config.ui || !config.ui.panels) return;

    // WOS-306: Load saved panel state from localStorage (progressive disclosure)
    const savedState = localStorage.getItem('wos-layout-preferences');
    const panelState = savedState ? JSON.parse(savedState) : {};

    // Get all panels with data-panel attribute
    const panelElements = document.querySelectorAll('[data-panel]');
    panelElements.forEach(panelEl => {
      const panelName = panelEl.dataset.panel;
      const panelConfig = config.ui.panels[panelName] || { visible: true, collapsed: false };

      // WOS-306: Override config with saved state if available
      if (panelState[panelName]) {
        panelConfig.collapsed = (panelState[panelName] === 'collapsed');
      }

      this.panels[panelName] = {
        element: panelEl,
        config: panelConfig
      };

      // TOOLBAR PATTERN: Don't apply collapsed class or create tabs
      // Visibility is controlled by toolbar via .active class
      // (Old accordion code removed)
    });
  }

  setupEventListeners() {
    // Add click listeners to all collapse buttons (legacy accordion support)
    document.querySelectorAll('.btn-collapse').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const panel = e.target.closest('[data-panel]');
        if (panel) {
          const panelName = panel.dataset.panel;
          this.toggleCollapse(panelName);
        }
      });
    });

    // TOOLBAR PATTERN: Add click listeners to toolbar icons
    document.querySelectorAll('.toolbar-icon').forEach(btn => {
      btn.addEventListener('click', (e) => {
        const panelName = btn.dataset.panelToggle;
        if (panelName) {
          this.activatePanel(panelName);
        }
      });
    });

    // Initialize: Show only the active panel (learning_objectives by default)
    this.initializeToolbarPanels();

    // Setup resize handle for terminal
    this.setupResizeHandle();
  }

  initializeToolbarPanels() {
    // Hide all panels first
    Object.keys(this.panels).forEach(panelName => {
      this.panels[panelName].element.classList.remove('active');
    });

    // Show the default panel (help)
    const defaultPanel = 'help';
    if (this.panels[defaultPanel]) {
      this.panels[defaultPanel].element.classList.add('active');
    }
  }

  activatePanel(panelName) {
    // TOGGLE BEHAVIOR: If clicking already-active panel, hide it (give space to terminal)
    const currentlyActive = this.panels[panelName]?.element.classList.contains('active');

    if (currentlyActive) {
      // Hide the active panel
      this.panels[panelName].element.classList.remove('active');

      // Deactivate all toolbar buttons
      document.querySelectorAll('.toolbar-icon').forEach(btn => {
        btn.classList.remove('active');
      });

      // Save state: no panel active
      localStorage.setItem('wos-active-panel', 'none');
      return;
    }

    // Hide all panels
    Object.keys(this.panels).forEach(name => {
      this.panels[name].element.classList.remove('active');
    });

    // Show selected panel
    if (this.panels[panelName]) {
      this.panels[panelName].element.classList.add('active');
    }

    // Update toolbar button active states
    document.querySelectorAll('.toolbar-icon').forEach(btn => {
      if (btn.dataset.panelToggle === panelName) {
        btn.classList.add('active');
      } else {
        btn.classList.remove('active');
      }
    });

    // Save state to localStorage
    localStorage.setItem('wos-active-panel', panelName);
  }

  setupResizeHandle() {
    const resizeHandle = document.querySelector('.resize-handle');
    const terminalContainer = document.querySelector('.terminal-container');

    if (!resizeHandle || !terminalContainer) return;

    let isResizing = false;
    let startY = 0;
    let startHeight = 0;

    const onMouseDown = (e) => {
      isResizing = true;
      startY = e.clientY;
      startHeight = terminalContainer.offsetHeight;

      // Prevent text selection during drag
      e.preventDefault();

      // Add visual feedback
      document.body.style.cursor = 'nwse-resize';
      resizeHandle.style.opacity = '1';
    };

    const onMouseMove = (e) => {
      if (!isResizing) return;

      const deltaY = e.clientY - startY;
      const newHeight = startHeight + deltaY;

      // Enforce min/max constraints (100px - 600px)
      const minHeight = 100;
      const maxHeight = 600;
      const clampedHeight = Math.max(minHeight, Math.min(maxHeight, newHeight));

      // Update terminal height
      terminalContainer.style.flex = `0 0 ${clampedHeight}px`;

      // Save to localStorage for persistence
      localStorage.setItem('wos-terminal-height', clampedHeight);
    };

    const onMouseUp = () => {
      if (!isResizing) return;

      isResizing = false;

      // Remove visual feedback
      document.body.style.cursor = '';
      resizeHandle.style.opacity = '';
    };

    // Add event listeners
    resizeHandle.addEventListener('mousedown', onMouseDown);
    document.addEventListener('mousemove', onMouseMove);
    document.addEventListener('mouseup', onMouseUp);

    // Restore saved height from localStorage
    const savedHeight = localStorage.getItem('wos-terminal-height');
    if (savedHeight) {
      terminalContainer.style.flex = `0 0 ${savedHeight}px`;
    }
  }

  toggleCollapse(panelName) {
    const panel = this.panels[panelName];
    if (!panel) return;

    const isCollapsed = panel.element.classList.contains('collapsed');
    if (isCollapsed) {
      this.expandPanel(panelName);
    } else {
      this.collapsePanel(panelName);
    }
  }

  collapsePanel(panelName) {
    const panel = this.panels[panelName];
    if (!panel) return;

    panel.element.classList.add('collapsed');

    // WOS-306: Use CSS transitions instead of display:none for smooth animations
    // The CSS handles hiding via opacity: 0 and max-height: 0

    // WOS-306: Create tab for collapsed panel
    this.createPanelTab(panelName, panel.element);

    // Update config and persist to localStorage
    panel.config.collapsed = true;
    this.savePanelState();

    // Icon rotation is handled by CSS (.collapsed .btn-collapse svg)
  }

  expandPanel(panelName) {
    const panel = this.panels[panelName];
    if (!panel) return;

    // ACCORDION BEHAVIOR: Collapse all other panels first
    for (const [otherName, otherPanel] of Object.entries(this.panels)) {
      if (otherName !== panelName && !otherPanel.element.classList.contains('collapsed')) {
        this.collapsePanel(otherName);
      }
    }

    panel.element.classList.remove('collapsed');

    // WOS-306: Use CSS transitions instead of display for smooth animations
    // The CSS handles showing via opacity: 1 and max-height: auto

    // WOS-306: Remove tab when panel is expanded
    this.removePanelTab(panelName, panel.element);

    // Update config and persist to localStorage
    panel.config.collapsed = false;
    this.savePanelState();

    // Icon rotation is handled by CSS (.btn-collapse svg)
  }

  showPanel(panelName) {
    const panel = this.panels[panelName];
    if (!panel) return;

    panel.element.style.display = '';
    panel.config.visible = true;

    // WOS-FILE-EDIT-01: Update file list when filesystem panel opens
    if (panelName === 'filesystem' && this.terminal) {
      this.terminal.updateFilesystemList();
    }
  }

  hidePanel(panelName) {
    const panel = this.panels[panelName];
    if (!panel) return;

    panel.element.style.display = 'none';
    panel.config.visible = false;
  }

  // WOS-306: Create tab element for collapsed panel (progressive disclosure)
  createPanelTab(panelName, panelElement) {
    // Check if tab already exists
    const existingTab = panelElement.querySelector('.panel-tab');
    if (existingTab) return;

    // Get panel title from header
    const header = panelElement.querySelector('.file-panel-header h3');
    const panelTitle = header ? header.textContent : panelName;

    // Create tab element (WOS-308: role="button" for semantic correctness)
    const tab = document.createElement('div');
    tab.className = 'panel-tab';
    tab.setAttribute('role', 'button');
    tab.setAttribute('aria-expanded', 'false');
    tab.setAttribute('aria-label', `Expand ${panelTitle} panel`);
    tab.setAttribute('tabindex', '0'); // WOS-308: Make keyboard accessible
    tab.textContent = panelTitle;

    // Add click listener to expand panel
    tab.addEventListener('click', () => {
      this.expandPanel(panelName);
    });

    // Add keyboard support (Enter/Space to expand)
    tab.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' || e.key === ' ') {
        e.preventDefault();
        this.expandPanel(panelName);
      }
    });

    // Make tab focusable
    tab.setAttribute('tabindex', '0');

    // Insert tab at the beginning of the panel
    panelElement.insertBefore(tab, panelElement.firstChild);
  }

  // WOS-306: Remove tab element when panel is expanded
  removePanelTab(panelName, panelElement) {
    const tab = panelElement.querySelector('.panel-tab');
    if (tab) {
      tab.remove();
    }
  }

  savePanelState() {
    // WOS-306: Save panel state to localStorage (progressive disclosure)
    const panelState = {};
    for (const [name, panel] of Object.entries(this.panels)) {
      // Store simple 'collapsed' or 'expanded' string
      panelState[name] = panel.config.collapsed === true ? 'collapsed' : 'expanded';
    }
    localStorage.setItem('wos-layout-preferences', JSON.stringify(panelState));
  }
}

class FileManager {
  constructor(wos) {
    this.wos = wos;
    this.files = new Map(); // fileName -> {name, content, size, modified}
    this.selectedFile = null;

    this.setupEventListeners();
    this.refreshFileList();

    // WOS-FILE-EDIT-01: Auto-refresh file list every 600ms
    this.startFileSystemObserver();
  }

  startFileSystemObserver() {
    // WOS-FILE-EDIT-01: Disabled polling - Terminal.updateFilesystemList() handles this
    // setInterval(() => {
    //   this.refreshFileList();
    // }, 600);
  }

  setupEventListeners() {
    // Upload button
    document.getElementById('btn-upload').addEventListener('click', () => {
      document.getElementById('file-upload-input').click();
    });

    // File input change
    document.getElementById('file-upload-input').addEventListener('change', (e) => {
      this.handleFileUpload(e.target.files);
    });

    // New file button
    document.getElementById('btn-new-file').addEventListener('click', () => {
      this.createNewFile();
    });

    // Refresh button
    document.getElementById('btn-refresh').addEventListener('click', () => {
      this.refreshFileList();
    });

    // Action buttons
    document.getElementById('btn-edit').addEventListener('click', () => {
      if (this.selectedFile) {
        this.openVimEditor(this.selectedFile);
      }
    });

    document.getElementById('btn-download').addEventListener('click', () => {
      if (this.selectedFile) {
        this.downloadFile(this.selectedFile);
      }
    });

    document.getElementById('btn-delete').addEventListener('click', () => {
      if (this.selectedFile) {
        this.deleteFile(this.selectedFile);
      }
    });
  }

  async handleFileUpload(files) {
    for (const file of files) {
      try {
        const content = await file.text();
        const fileData = {
          name: file.name,
          content: content,
          size: content.length,
          modified: new Date(file.lastModified).toLocaleString()
        };

        this.files.set(file.name, fileData);

        // Write to VFS via WOS
        if (this.wos) {
          const path = `/tmp/${file.name}`;
          const cmd = `echo "${content.replace(/"/g, '\\"').replace(/\n/g, '\\n')}" > ${path}`;
          this.wos.executeCommand(`touch ${path}`);
          // Store content in localStorage for now
          localStorage.setItem(`wos-file-${file.name}`, content);
        }
      } catch (error) {
        console.error(`Error uploading ${file.name}:`, error);
      }
    }

    this.refreshFileList();
    this.updateFileCount();
  }

  createNewFile() {
    const fileName = prompt('Enter new file name:', 'untitled.txt');
    if (!fileName) return;

    const fileData = {
      name: fileName,
      content: '',
      size: 0,
      modified: new Date().toLocaleString()
    };

    this.files.set(fileName, fileData);
    localStorage.setItem(`wos-file-${fileName}`, '');

    this.refreshFileList();
    this.updateFileCount();
    this.selectFile(fileName);
    this.openVimEditor(fileName);
  }

  refreshFileList() {
    // Track files before refresh to detect new ones
    const previousFiles = new Set(this.files.keys());

    // WOS-FILE-EDIT-01: Load files from WASM filesystem using ls command
    if (this.wos && this.wos.executeCommand) {
      try {
        // Use 'ls -1' to list one file per line
        const result = this.wos.executeCommand('ls -1');
        console.log('[FileManager] ls -1 result:', result);
        if (result && typeof result === 'string' && !result.includes('Error') && !result.includes('error')) {
          const lines = result.trim().split('\n');
          console.log('[FileManager] ls -1 lines:', lines);

          // Parse ls output to get file names (one per line format)
          for (let i = 0; i < lines.length; i++) {
            const fileName = lines[i].trim();
            if (!fileName || fileName === '.' || fileName === '..') continue;

            // Skip if it looks like a directory (ends with /)
            if (fileName.endsWith('/')) continue;

            console.log('[FileManager] Processing file:', fileName);

            // Try to read file content from WASM filesystem
            let content = '';
            let size = 0;
            try {
              const catResult = this.wos.executeCommand(`cat ${fileName}`) || '';
              // If cat returns error message, file is empty or unreadable
              if (catResult.includes('No such file') || catResult.includes('Error') || catResult.includes('cat:')) {
                content = '';
              } else {
                content = catResult;
              }
              size = content.length;
            } catch (e) {
              // File exists but can't be read, use empty content
              content = '';
              size = 0;
            }

            console.log('[FileManager] Adding file to map:', fileName, 'size:', size);
            this.files.set(fileName, {
              name: fileName,
              content: content,
              size: size,
              modified: 'From WASM filesystem'
            });
          }
        }
        console.log('[FileManager] Total files in map:', this.files.size);
      } catch (error) {
        console.error('Error reading WASM filesystem:', error);
      }
    }

    // Also load files from localStorage (for backward compatibility)
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key.startsWith('wos-file-')) {
        const fileName = key.replace('wos-file-', '');
        const content = localStorage.getItem(key);
        if (!this.files.has(fileName)) {
          this.files.set(fileName, {
            name: fileName,
            content: content,
            size: content.length,
            modified: 'Loaded from storage'
          });
        }
      }
    }

    // WOS-FILE-EDIT-01: Auto-select newly created files
    const newFiles = Array.from(this.files.keys()).filter(name => !previousFiles.has(name));
    if (newFiles.length > 0 && !this.selectedFile) {
      // Auto-select the first new file if nothing is currently selected
      this.selectFile(newFiles[0]);
    }

    this.renderFileList();
    this.updateFileCount();
  }

  renderFileList() {
    // WOS-FILE-EDIT-01: Terminal.updateFilesystemList() now handles rendering
    // FileManager only needs to update selection state, not render the list
    tracer.debug('FILE', 'FileManager.renderFileList() - delegating to Terminal');
    return;

    const container = fileList || browser;

    for (const [fileName, fileData] of this.files) {
      const item = document.createElement('div');
      item.className = 'file-item';
      if (this.selectedFile === fileName) {
        item.classList.add('selected');
      }

      item.innerHTML = `
        <div class="file-item-name">
          <svg class="file-item-icon" viewBox="0 0 24 24" fill="currentColor">
            <path d="M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z"/>
          </svg>
          <span>${fileName}</span>
        </div>
        <span class="file-item-size">${this.formatSize(fileData.size)}</span>
      `;

      item.addEventListener('click', () => this.selectFile(fileName));
      item.addEventListener('dblclick', () => this.openVimEditor(fileName));

      container.appendChild(item);
    }
  }

  selectFile(fileName) {
    this.selectedFile = fileName;
    this.renderFileList();
    this.updateFileDetails();

    // Enable action buttons
    document.getElementById('btn-edit').disabled = false;
    document.getElementById('btn-download').disabled = false;
    document.getElementById('btn-delete').disabled = false;
  }

  updateFileDetails() {
    const details = document.getElementById('file-details');

    if (!this.selectedFile) {
      details.innerHTML = '<p class="no-selection">No file selected</p>';
      return;
    }

    const fileData = this.files.get(this.selectedFile);
    details.innerHTML = `
      <p><strong>Name:</strong> ${fileData.name}</p>
      <p><strong>Size:</strong> ${this.formatSize(fileData.size)}</p>
      <p><strong>Modified:</strong> ${fileData.modified}</p>
      <p><strong>Lines:</strong> ${fileData.content.split('\n').length}</p>
    `;
  }

  updateFileCount() {
    document.getElementById('file-count').textContent = this.files.size;
  }

  formatSize(bytes) {
    if (bytes === 0) return '0 B';
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  openVimEditor(fileName) {
    // WOS-FILE-EDIT-01: Read file content from WASM filesystem (not just the Map)
    tracer.debug('FILE', 'openVimEditor called', { fileName });

    let content = '';

    // Try to get from WASM filesystem first
    if (this.wos) {
      try {
        const result = this.wos.executeCommand(`cat ${fileName}`);
        // Check if file exists (cat returns error if not found)
        if (result && !result.includes('No such file') && !result.includes('cat:')) {
          content = result;
          tracer.debug('FILE', 'Loaded content from WASM', { fileName, contentLength: content.length });
        } else {
          // File doesn't exist in WASM, might be new or from localStorage
          tracer.debug('FILE', 'File not in WASM, checking Map/localStorage', { fileName });
          const fileData = this.files.get(fileName);
          if (fileData) {
            content = fileData.content || '';
          }
        }
      } catch (error) {
        tracer.warn('FILE', 'Error reading from WASM', { fileName, error: error.message });
        const fileData = this.files.get(fileName);
        if (fileData) {
          content = fileData.content || '';
        }
      }
    } else {
      // No WASM available, use Map
      const fileData = this.files.get(fileName);
      if (fileData) {
        content = fileData.content || '';
      }
    }

    // Create Vim editor with save callback that writes to WASM
    const vim = new VimEditor(fileName, content, (newContent) => {
      tracer.info('FILE', 'Saving file to WASM filesystem', { fileName, contentLength: newContent.length });

      // Save to WASM filesystem using echo redirect
      if (this.wos) {
        try {
          // Escape content for shell (handle quotes, backslashes, etc.)
          const escapedContent = this.escapeForShell(newContent);
          const saveCommand = `echo "${escapedContent}" > ${fileName}`;
          this.wos.executeCommand(saveCommand);
          tracer.info('FILE', 'File saved to WASM successfully', { fileName });
        } catch (error) {
          tracer.error('FILE', 'Error saving to WASM', { fileName, error: error.message });
        }
      }

      // Also update local Map and localStorage for backward compatibility
      const fileData = this.files.get(fileName) || { name: fileName };
      fileData.content = newContent;
      fileData.size = newContent.length;
      fileData.modified = new Date().toLocaleString();
      this.files.set(fileName, fileData);
      localStorage.setItem(`wos-file-${fileName}`, newContent);

      // Refresh file list to show updated size
      this.refreshFileList();
    });

    vim.open();
  }

  // WOS-FILE-EDIT-01: Escape content for safe shell command execution
  escapeForShell(content) {
    // Replace backslashes first (must be first to avoid double-escaping)
    let escaped = content.replace(/\\/g, '\\\\');
    // Escape double quotes
    escaped = escaped.replace(/"/g, '\\"');
    // Escape dollar signs (variable expansion)
    escaped = escaped.replace(/\$/g, '\\$');
    // Escape backticks (command substitution)
    escaped = escaped.replace(/`/g, '\\`');
    // Replace newlines with \n literal
    escaped = escaped.replace(/\n/g, '\\n');
    return escaped;
  }

  downloadFile(fileName) {
    const fileData = this.files.get(fileName);
    if (!fileData) return;

    const blob = new Blob([fileData.content], { type: 'text/plain' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = fileName;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }

  deleteFile(fileName) {
    if (!confirm(`Delete ${fileName}?`)) return;

    this.files.delete(fileName);
    localStorage.removeItem(`wos-file-${fileName}`);

    if (this.selectedFile === fileName) {
      this.selectedFile = null;
      document.getElementById('btn-edit').disabled = true;
      document.getElementById('btn-download').disabled = true;
      document.getElementById('btn-delete').disabled = true;
    }

    this.refreshFileList();
    this.updateFileCount();
    this.updateFileDetails();
  }
}

class VimEditor {
  constructor(fileName, content, saveCallback) {
    this.fileName = fileName;
    this.lines = content.split('\n');
    if (this.lines.length === 0) this.lines = [''];
    this.saveCallback = saveCallback;

    this.cursorRow = 0;
    this.cursorCol = 0;
    this.mode = 'NORMAL'; // NORMAL, INSERT, COMMAND
    this.commandBuffer = '';
    this.modified = false;
    this.message = '';

    this.modal = document.getElementById('vim-modal');
    this.editor = document.getElementById('vim-editor');
    this.setupVimEventListeners();
  }

  setupVimEventListeners() {
    this.keyHandler = this.handleKeyPress.bind(this);

    document.getElementById('vim-close').addEventListener('click', () => {
      if (this.modified) {
        if (confirm('You have unsaved changes. Close anyway?')) {
          this.close();
        }
      } else {
        this.close();
      }
    });
  }

  open() {
    this.modal.classList.remove('hidden');
    this.render();
    // Use document-level event capture to ensure all keyboard events are caught
    document.addEventListener('keydown', this.keyHandler);
    // Also focus the editor for visual feedback
    this.editor.focus();
  }

  close() {
    this.modal.classList.add('hidden');
    document.removeEventListener('keydown', this.keyHandler);
  }

  handleKeyPress(e) {
    if (this.mode === 'NORMAL') {
      this.handleNormalMode(e);
    } else if (this.mode === 'INSERT') {
      this.handleInsertMode(e);
    } else if (this.mode === 'COMMAND') {
      this.handleCommandMode(e);
    }

    this.render();
  }

  handleNormalMode(e) {
    switch(e.key) {
      case 'h': // Move left
      case 'ArrowLeft':
        e.preventDefault();
        this.moveCursorLeft();
        break;
      case 'j': // Move down
      case 'ArrowDown':
        e.preventDefault();
        this.moveCursorDown();
        break;
      case 'k': // Move up
      case 'ArrowUp':
        e.preventDefault();
        this.moveCursorUp();
        break;
      case 'l': // Move right
      case 'ArrowRight':
        e.preventDefault();
        this.moveCursorRight();
        break;
      case 'i': // Insert mode
        e.preventDefault();
        this.mode = 'INSERT';
        this.message = '-- INSERT --';
        break;
      case 'a': // Insert after cursor
        e.preventDefault();
        this.moveCursorRight();
        this.mode = 'INSERT';
        this.message = '-- INSERT --';
        break;
      case 'o': // New line below
        e.preventDefault();
        this.lines.splice(this.cursorRow + 1, 0, '');
        this.cursorRow++;
        this.cursorCol = 0;
        this.mode = 'INSERT';
        this.modified = true;
        break;
      case 'O': // New line above
        e.preventDefault();
        this.lines.splice(this.cursorRow, 0, '');
        this.cursorCol = 0;
        this.mode = 'INSERT';
        this.modified = true;
        break;
      case 'x': // Delete character
        e.preventDefault();
        if (this.cursorCol < this.lines[this.cursorRow].length) {
          this.lines[this.cursorRow] =
            this.lines[this.cursorRow].slice(0, this.cursorCol) +
            this.lines[this.cursorRow].slice(this.cursorCol + 1);
          this.modified = true;
        }
        break;
      case ':': // Command mode
        e.preventDefault();
        this.mode = 'COMMAND';
        this.commandBuffer = ':';
        break;
      case 'Escape':
        e.preventDefault();
        this.message = '';
        break;
    }
  }

  handleInsertMode(e) {
    if (e.key === 'Escape') {
      e.preventDefault();
      this.mode = 'NORMAL';
      this.message = '';
      if (this.cursorCol > 0) this.cursorCol--;
      return;
    }

    if (e.key === 'Enter') {
      e.preventDefault();
      const currentLine = this.lines[this.cursorRow];
      const beforeCursor = currentLine.slice(0, this.cursorCol);
      const afterCursor = currentLine.slice(this.cursorCol);
      this.lines[this.cursorRow] = beforeCursor;
      this.lines.splice(this.cursorRow + 1, 0, afterCursor);
      this.cursorRow++;
      this.cursorCol = 0;
      this.modified = true;
      return;
    }

    if (e.key === 'Backspace') {
      e.preventDefault();
      if (this.cursorCol > 0) {
        this.lines[this.cursorRow] =
          this.lines[this.cursorRow].slice(0, this.cursorCol - 1) +
          this.lines[this.cursorRow].slice(this.cursorCol);
        this.cursorCol--;
        this.modified = true;
      } else if (this.cursorRow > 0) {
        const prevLine = this.lines[this.cursorRow - 1];
        this.lines[this.cursorRow - 1] = prevLine + this.lines[this.cursorRow];
        this.lines.splice(this.cursorRow, 1);
        this.cursorRow--;
        this.cursorCol = prevLine.length;
        this.modified = true;
      }
      return;
    }

    if (e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
      e.preventDefault();
      this.lines[this.cursorRow] =
        this.lines[this.cursorRow].slice(0, this.cursorCol) +
        e.key +
        this.lines[this.cursorRow].slice(this.cursorCol);
      this.cursorCol++;
      this.modified = true;
    }
  }

  handleCommandMode(e) {
    if (e.key === 'Escape') {
      e.preventDefault();
      this.mode = 'NORMAL';
      this.commandBuffer = '';
      this.message = '';
      return;
    }

    if (e.key === 'Enter') {
      e.preventDefault();
      this.executeCommand(this.commandBuffer);
      this.commandBuffer = '';
      this.mode = 'NORMAL';
      return;
    }

    if (e.key === 'Backspace') {
      e.preventDefault();
      if (this.commandBuffer.length > 1) {
        this.commandBuffer = this.commandBuffer.slice(0, -1);
      } else {
        this.mode = 'NORMAL';
        this.commandBuffer = '';
      }
      return;
    }

    if (e.key.length === 1 && !e.ctrlKey && !e.altKey && !e.metaKey) {
      e.preventDefault();
      this.commandBuffer += e.key;
    }
  }

  executeCommand(cmd) {
    cmd = cmd.slice(1); // Remove leading ':'

    // WOS-401: List of available vim commands for helpful error messages
    const AVAILABLE_COMMANDS = [':w', ':write', ':q', ':quit', ':q!', ':quit!', ':wq', ':x', ':help'];

    if (cmd === 'w' || cmd === 'write') {
      this.save();
      this.message = `"${this.fileName}" ${this.lines.length}L written`;
    } else if (cmd === 'q' || cmd === 'quit') {
      if (this.modified) {
        this.message = 'No write since last change (add ! to override)';
      } else {
        this.close();
      }
    } else if (cmd === 'q!' || cmd === 'quit!') {
      this.close();
    } else if (cmd === 'wq' || cmd === 'x') {
      this.save();
      this.close();
    } else if (cmd === 'help') {
      // WOS-402: Vim :help command
      this.message = `WOS Vim Commands:
:w, :write - Save file
:q, :quit - Quit (fails if modified)
:q!, :quit! - Quit without saving
:wq, :x - Save and quit
:help - Show this help
Press Escape to clear this message`;
    } else {
      // WOS-401: Improved error message with available command list
      this.message = `E492: Not an editor command: ${cmd}. Available commands: ${AVAILABLE_COMMANDS.join(', ')}`;
    }
  }

  save() {
    const content = this.lines.join('\n');
    this.saveCallback(content);
    this.modified = false;
  }

  moveCursorUp() {
    if (this.cursorRow > 0) {
      this.cursorRow--;
      this.cursorCol = Math.min(this.cursorCol, this.lines[this.cursorRow].length);
    }
  }

  moveCursorDown() {
    if (this.cursorRow < this.lines.length - 1) {
      this.cursorRow++;
      this.cursorCol = Math.min(this.cursorCol, this.lines[this.cursorRow].length);
    }
  }

  moveCursorLeft() {
    if (this.cursorCol > 0) {
      this.cursorCol--;
    }
  }

  moveCursorRight() {
    if (this.cursorCol < this.lines[this.cursorRow].length) {
      this.cursorCol++;
    }
  }

  render() {
    // Update header
    document.getElementById('vim-filename').textContent = this.fileName;
    document.getElementById('vim-modified').classList.toggle('hidden', !this.modified);

    const modeText = this.mode === 'NORMAL' ? '-- NORMAL --' :
                     this.mode === 'INSERT' ? '-- INSERT --' :
                     '-- COMMAND --';
    document.getElementById('vim-mode').textContent = modeText;
    document.getElementById('vim-position').textContent = `${this.cursorRow + 1},${this.cursorCol + 1}`;

    // Update command line
    document.getElementById('vim-command').textContent = this.commandBuffer;
    document.getElementById('vim-message').textContent = this.message;

    // Render editor content
    this.editor.innerHTML = '';

    this.lines.forEach((line, idx) => {
      const lineDiv = document.createElement('div');
      lineDiv.className = 'vim-line';

      if (idx === this.cursorRow) {
        const beforeCursor = line.slice(0, this.cursorCol);
        const atCursor = line[this.cursorCol] || ' ';
        const afterCursor = line.slice(this.cursorCol + 1);

        lineDiv.innerHTML =
          this.escapeHtml(beforeCursor) +
          `<span class="vim-cursor">${this.escapeHtml(atCursor)}</span>` +
          this.escapeHtml(afterCursor);
      } else {
        lineDiv.textContent = line || ' ';
      }

      this.editor.appendChild(lineDiv);
    });
  }

  escapeHtml(text) {
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }
}

// WOS-305: Comprehensive Help System Data Structure
// Provides detailed documentation for all commands with usage, examples, and related commands
const HELP_DATA = {
  help: {
    name: 'help',
    description: 'Display help information for commands',
    usage: 'help [command]',
    examples: [
      { command: 'help', description: 'Show all available commands' },
      { command: 'help ls', description: 'Show detailed help for ls command' },
      { command: 'help cat', description: 'Show detailed help for cat command' }
    ],
    options: [
      { flag: '[command]', description: 'Optional command name for detailed help' }
    ],
    related: ['version', 'config']
  },
  ls: {
    name: 'ls',
    description: 'List files and directories in the current directory',
    usage: 'ls [options] [path]',
    examples: [
      { command: 'ls', description: 'List files in current directory' },
      { command: 'ls /home', description: 'List files in /home directory' },
      { command: 'ls -l', description: 'List files with detailed information' }
    ],
    options: [
      { flag: '-l', description: 'Use long listing format' },
      { flag: '-a', description: 'Show hidden files' },
      { flag: '[path]', description: 'Directory path to list (default: current directory)' }
    ],
    related: ['cd', 'pwd', 'mkdir']
  },
  cat: {
    name: 'cat',
    description: 'Concatenate and display file contents',
    usage: 'cat <filename>',
    examples: [
      { command: 'cat file.txt', description: 'Display contents of file.txt' },
      { command: 'cat /etc/config', description: 'Display contents of /etc/config' }
    ],
    options: [
      { flag: '<filename>', description: 'File to display (required)' }
    ],
    related: ['echo', 'grep', 'wc']
  },
  echo: {
    name: 'echo',
    description: 'Print text to the terminal output',
    usage: 'echo <text>',
    examples: [
      { command: 'echo Hello World', description: 'Print "Hello World"' },
      { command: 'echo "Hello, WOS!"', description: 'Print quoted text' }
    ],
    options: [
      { flag: '<text>', description: 'Text to print (required)' }
    ],
    related: ['cat', 'printf']
  },
  cd: {
    name: 'cd',
    description: 'Change the current working directory',
    usage: 'cd <directory>',
    examples: [
      { command: 'cd /home', description: 'Change to /home directory' },
      { command: 'cd ..', description: 'Go up one directory level' },
      { command: 'cd ~', description: 'Go to home directory' }
    ],
    options: [
      { flag: '<directory>', description: 'Directory path to change to (required)' },
      { flag: '..', description: 'Parent directory' },
      { flag: '~', description: 'Home directory' }
    ],
    related: ['pwd', 'ls', 'mkdir']
  },
  pwd: {
    name: 'pwd',
    description: 'Print the current working directory path',
    usage: 'pwd',
    examples: [
      { command: 'pwd', description: 'Show current directory path' }
    ],
    options: [],
    related: ['cd', 'ls']
  },
  mkdir: {
    name: 'mkdir',
    description: 'Create a new directory',
    usage: 'mkdir <directory>',
    examples: [
      { command: 'mkdir mydir', description: 'Create directory named "mydir"' },
      { command: 'mkdir /home/docs', description: 'Create directory at /home/docs' }
    ],
    options: [
      { flag: '<directory>', description: 'Directory name or path to create (required)' }
    ],
    related: ['ls', 'cd', 'rm']
  },
  rm: {
    name: 'rm',
    description: 'Remove (delete) files or directories',
    usage: 'rm <file>',
    examples: [
      { command: 'rm file.txt', description: 'Delete file.txt' },
      { command: 'rm /tmp/temp.log', description: 'Delete /tmp/temp.log' }
    ],
    options: [
      { flag: '<file>', description: 'File to remove (required)' },
      { flag: '-r', description: 'Remove directories recursively' }
    ],
    related: ['touch', 'mkdir', 'ls']
  },
  touch: {
    name: 'touch',
    description: 'Create a new empty file or update file timestamp',
    usage: 'touch <filename>',
    examples: [
      { command: 'touch file.txt', description: 'Create empty file.txt' },
      { command: 'touch /home/notes.md', description: 'Create file at /home/notes.md' }
    ],
    options: [
      { flag: '<filename>', description: 'File to create or update (required)' }
    ],
    related: ['rm', 'cat', 'mkdir']
  },
  ps: {
    name: 'ps',
    description: 'List running processes in the system',
    usage: 'ps',
    examples: [
      { command: 'ps', description: 'Show all running processes' }
    ],
    options: [],
    related: ['kill', 'state']
  },
  grep: {
    name: 'grep',
    description: 'Search for patterns in file contents',
    usage: 'grep <pattern> <file>',
    examples: [
      { command: 'grep "hello" file.txt', description: 'Search for "hello" in file.txt' },
      { command: 'grep error log.txt', description: 'Search for "error" in log.txt' }
    ],
    options: [
      { flag: '<pattern>', description: 'Text pattern to search for (required)' },
      { flag: '<file>', description: 'File to search in (required)' }
    ],
    related: ['cat', 'wc']
  },
  wc: {
    name: 'wc',
    description: 'Count words, lines, and bytes in files',
    usage: 'wc <file>',
    examples: [
      { command: 'wc file.txt', description: 'Count lines, words, and bytes in file.txt' }
    ],
    options: [
      { flag: '<file>', description: 'File to analyze (required)' },
      { flag: '-l', description: 'Count only lines' },
      { flag: '-w', description: 'Count only words' },
      { flag: '-c', description: 'Count only bytes' }
    ],
    related: ['cat', 'grep']
  },
  vim: {
    name: 'vim',
    description: 'Open the Vim modal text editor',
    usage: 'vim [filename]',
    examples: [
      { command: 'vim', description: 'Open empty Vim editor' },
      { command: 'vim file.txt', description: 'Edit file.txt in Vim' }
    ],
    options: [
      { flag: '[filename]', description: 'Optional file to edit' }
    ],
    related: ['edit', 'cat']
  },
  edit: {
    name: 'edit',
    description: 'Open the Monaco code editor',
    usage: 'edit <filename>',
    examples: [
      { command: 'edit file.js', description: 'Edit file.js in Monaco editor' },
      { command: 'edit config.json', description: 'Edit config.json' }
    ],
    options: [
      { flag: '<filename>', description: 'File to edit (required)' }
    ],
    related: ['vim', 'cat']
  },
  bash: {
    name: 'bash',
    description: 'Execute a shell script file',
    usage: 'bash <script>',
    examples: [
      { command: 'bash script.sh', description: 'Execute script.sh' }
    ],
    options: [
      { flag: '<script>', description: 'Script file to execute (required)' }
    ],
    related: ['source']
  },
  source: {
    name: 'source',
    description: 'Execute script in current shell context',
    usage: 'source <script>',
    examples: [
      { command: 'source config.sh', description: 'Source config.sh in current shell' }
    ],
    options: [
      { flag: '<script>', description: 'Script file to source (required)' }
    ],
    related: ['bash']
  },
  version: {
    name: 'version',
    description: 'Show the WOS system version information',
    usage: 'version',
    examples: [
      { command: 'version', description: 'Display system version' }
    ],
    options: [],
    related: ['help', 'state']
  },
  state: {
    name: 'state',
    description: 'Display current kernel state and system information',
    usage: 'state',
    examples: [
      { command: 'state', description: 'Show kernel state' }
    ],
    options: [],
    related: ['ps', 'version']
  },
  reset: {
    name: 'reset',
    description: 'Reset the system to initial state',
    usage: 'reset',
    examples: [
      { command: 'reset', description: 'Reset system and clear all data' }
    ],
    options: [],
    related: ['clear']
  },
  clear: {
    name: 'clear',
    description: 'Clear the terminal screen',
    usage: 'clear',
    examples: [
      { command: 'clear', description: 'Clear terminal output' },
      { command: 'cls', description: 'Alias for clear' }
    ],
    options: [],
    related: ['reset']
  },
  history: {
    name: 'history',
    description: 'Show command history',
    usage: 'history',
    examples: [
      { command: 'history', description: 'Display all previous commands' }
    ],
    options: [],
    related: ['clear']
  },
  config: {
    name: 'config',
    description: 'Show current system configuration',
    usage: 'config',
    examples: [
      { command: 'config', description: 'Display configuration settings' }
    ],
    options: [],
    related: ['version', 'state']
  },
  theme: {
    name: 'theme',
    description: 'Change the terminal color theme',
    usage: 'theme <mode>',
    examples: [
      { command: 'theme dark', description: 'Switch to dark theme' },
      { command: 'theme light', description: 'Switch to light theme' },
      { command: 'theme auto', description: 'Use system theme preference' }
    ],
    options: [
      { flag: 'dark', description: 'Dark color scheme' },
      { flag: 'light', description: 'Light color scheme' },
      { flag: 'auto', description: 'Automatic based on system preference' }
    ],
    related: ['config']
  }
};

class Terminal {
  constructor(configManager) {
    this.output = document.getElementById('terminal-output');
    this.input = document.getElementById('terminal-input');
    this.terminalElement = document.getElementById('terminal');
    this.history = [];
    this.historyIndex = -1;
    this.wos = null;
    this.fileManager = null;
    this.configManager = configManager;

    this.setupEventListeners();
    this.printWelcome();
  }

  setupEventListeners() {
    tracer.debug('TERMINAL', 'Setting up event listeners');

    // Validate required elements exist
    if (!this.input) {
      tracer.error('TERMINAL', 'Input element not found');
      throw new Error('Terminal input element (#terminal-input) not found');
    }
    if (!this.terminalElement) {
      tracer.error('TERMINAL', 'Terminal element not found');
      throw new Error('Terminal element (#terminal) not found');
    }

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

    // Button controls - with null checks
    const btnClear = document.getElementById('btn-clear');
    const btnReset = document.getElementById('btn-reset');
    const btnSave = document.getElementById('btn-save');
    const btnLoad = document.getElementById('btn-load');
    const btnBenchmark = document.getElementById('btn-benchmark');

    if (!btnClear) tracer.warn('TERMINAL', 'Button not found: btn-clear');
    else btnClear.addEventListener('click', () => this.clear());

    if (!btnReset) tracer.warn('TERMINAL', 'Button not found: btn-reset');
    else btnReset.addEventListener('click', () => this.reset());

    if (!btnSave) tracer.warn('TERMINAL', 'Button not found: btn-save');
    else btnSave.addEventListener('click', () => this.saveState());

    if (!btnLoad) tracer.warn('TERMINAL', 'Button not found: btn-load');
    else btnLoad.addEventListener('click', () => this.loadState());

    if (!btnBenchmark) tracer.warn('TERMINAL', 'Button not found: btn-benchmark');
    else btnBenchmark.addEventListener('click', () => this.runBenchmark());

    // Keep input focused
    this.terminalElement.addEventListener('click', () => {
      this.input.focus();
    });

    tracer.debug('TERMINAL', 'Event listeners set up successfully');
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
    // WOS-400: Dynamic prompt showing current working directory
    if (!this.wos) {
      // Fallback if WASM not initialized
      this.printLine(`wos$ ${cmd}`, 'command');
      return;
    }

    try {
      const cwd = this.wos.getCurrentWorkingDirectory();
      const user = this.wos.getCurrentUser();
      // Format: user@wos:/path$ command
      this.printLine(`${user}@wos:${cwd}$ ${cmd}`, 'command');
    } catch (error) {
      // Fallback if getCurrentWorkingDirectory not available
      tracer.warn('TERMINAL', 'Failed to get CWD, using static prompt', error);
      this.printLine(`wos$ ${cmd}`, 'command');
    }
  }

  scrollToBottom() {
    this.terminalElement.scrollTop = this.terminalElement.scrollHeight;
  }

  clear() {
    this.output.innerHTML = '';
    this.printWelcome();
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

  runBenchmark() {
    if (!this.wos) {
      this.printLine('Error: WOS not initialized', 'error');
      return;
    }

    this.printLine('Running actual browser load test...', 'info');
    this.printLine('Executing CPU/memory intensive workloads', 'info');
    this.printLine('Watch System Monitor for real activity metrics', 'info');

    // Track actual workload metrics
    this.benchmarkRunning = true;
    this.benchmarkStartTime = performance.now();
    this.benchmarkWorkloadMetrics = {
      primeCalculations: 0,
      arrayOperations: 0,
      domManipulations: 0,
      wosCommands: 0,
      totalWorkloadTime: 0
    };

    const commands = [
      'echo "Process stress test iteration"',
      'ls',
      'ps',
      'cat /proc/1/status',
      'echo "Memory allocation test"',
      'ls -l',
      'echo "I/O operations test"',
      'ps',
      'echo "System call trace"',
      'ls'
    ];

    let commandIndex = 0;
    let iteration = 0;
    const maxIterations = 10;
    const delayBetweenCommands = 300; // ms

    const runNextCommand = () => {
      if (iteration >= maxIterations) {
        this.benchmarkRunning = false;
        const totalTime = performance.now() - this.benchmarkStartTime;
        this.printLine('Benchmark complete!', 'success');
        this.printLine(`Total execution time: ${totalTime.toFixed(2)}ms`, 'info');
        this.printLine(`Prime calculations: ${this.benchmarkWorkloadMetrics.primeCalculations}`, 'info');
        this.printLine(`Array operations: ${this.benchmarkWorkloadMetrics.arrayOperations}`, 'info');
        this.printLine(`DOM manipulations: ${this.benchmarkWorkloadMetrics.domManipulations}`, 'info');
        this.printLine(`WOS commands: ${this.benchmarkWorkloadMetrics.wosCommands}`, 'info');
        this.printLine('System monitor returning to normal levels', 'info');
        this.updateSystemInfo();
        return;
      }

      // Perform actual CPU-intensive workload: Prime number calculation
      const primeWorkloadStart = performance.now();
      const primeLimit = 5000 + (iteration * 1000); // Increasing load each iteration
      let primeCount = 0;

      const isPrime = (n) => {
        if (n <= 1) return false;
        if (n <= 3) return true;
        if (n % 2 === 0 || n % 3 === 0) return false;
        for (let i = 5; i * i <= n; i += 6) {
          if (n % i === 0 || n % (i + 2) === 0) return false;
        }
        return true;
      };

      for (let i = 0; i < primeLimit; i++) {
        if (isPrime(i)) primeCount++;
      }
      this.benchmarkWorkloadMetrics.primeCalculations += primeCount;
      const primeWorkloadTime = performance.now() - primeWorkloadStart;

      // Perform actual memory-intensive workload: Large array operations
      const arrayWorkloadStart = performance.now();
      const arraySize = 10000 + (iteration * 2000); // Growing arrays
      const testArray = Array.from({ length: arraySize }, () => Math.random());
      testArray.sort((a, b) => a - b); // CPU-intensive sorting
      const medianValue = testArray[Math.floor(arraySize / 2)];
      this.benchmarkWorkloadMetrics.arrayOperations += arraySize;
      const arrayWorkloadTime = performance.now() - arrayWorkloadStart;

      // Perform actual DOM manipulation workload
      const domWorkloadStart = performance.now();
      const container = document.createElement('div');
      container.style.display = 'none'; // Hidden but still processes
      for (let i = 0; i < 100; i++) {
        const element = document.createElement('div');
        element.textContent = `Load test element ${iteration}-${i}`;
        element.className = 'benchmark-element';
        container.appendChild(element);
      }
      document.body.appendChild(container);
      // Clean up immediately
      container.remove();
      this.benchmarkWorkloadMetrics.domManipulations += 100;
      const domWorkloadTime = performance.now() - domWorkloadStart;

      // Execute actual WOS command for system activity
      const cmd = commands[commandIndex];
      try {
        const result = this.wos.executeCommand(cmd);
        this.benchmarkWorkloadMetrics.wosCommands++;
      } catch (error) {
        // Some commands may fail, that's OK for benchmark purposes
      }

      // Track total workload time
      const totalWorkloadTime = primeWorkloadTime + arrayWorkloadTime + domWorkloadTime;
      this.benchmarkWorkloadMetrics.totalWorkloadTime += totalWorkloadTime;

      // Force update of system info with actual metrics
      this.updateSystemInfo();

      // Move to next command
      commandIndex = (commandIndex + 1) % commands.length;
      if (commandIndex === 0) {
        iteration++;
      }

      // Continue with next command
      if (iteration < maxIterations) {
        setTimeout(runNextCommand, delayBetweenCommands);
      } else {
        this.benchmarkRunning = false;
        const totalTime = performance.now() - this.benchmarkStartTime;
        this.printLine('Benchmark complete!', 'success');
        this.printLine(`Total execution time: ${totalTime.toFixed(2)}ms`, 'info');
        this.printLine(`Prime calculations: ${this.benchmarkWorkloadMetrics.primeCalculations}`, 'info');
        this.printLine(`Array operations: ${this.benchmarkWorkloadMetrics.arrayOperations}`, 'info');
        this.printLine(`DOM manipulations: ${this.benchmarkWorkloadMetrics.domManipulations}`, 'info');
        this.printLine(`WOS commands: ${this.benchmarkWorkloadMetrics.wosCommands}`, 'info');
        this.updateSystemInfo();
      }
    };

    // Start benchmark
    runNextCommand();
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
    // WOS-305: Enhanced help command with support for help <command>
    if (cmd.startsWith('help')) {
      const args = cmd.split(/\s+/).filter(arg => arg.length > 0); // Split on whitespace, filter empty
      if (args.length > 1 && args[1]) {
        this.printDetailedHelp(args[1]);
      } else {
        this.printHelp();
      }
      return; // IMPORTANT: Must return to prevent WASM from handling it
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

    if (cmd === 'config') {
      this.printConfig();
      return;
    }

    if (cmd === 'theme dark') {
      this.setTheme('dark');
      return;
    }

    if (cmd === 'theme light') {
      this.setTheme('light');
      return;
    }

    if (cmd === 'theme auto') {
      this.setTheme('auto');
      return;
    }

    // Handle vim command
    if (cmd.startsWith('vim ') || cmd === 'vim') {
      const fileName = cmd.split(' ')[1] || 'untitled.txt';
      this.openVim(fileName);
      return;
    }

    // Handle edit command (Monaco editor)
    if (cmd.startsWith('edit ') || cmd === 'edit') {
      const fileName = cmd.split(' ')[1];
      if (!fileName) {
        this.printLine('Usage: edit <filename>', 'error');
        return;
      }

      if (this.wos) {
        try {
          // Try to read the file using executeCommand
          const result = this.wos.executeCommand(`cat ${fileName}`);
          // Check if the result is an error message (contains "No such file")
          if (result && result.includes('No such file')) {
            openMonacoEditor(fileName, '', this.wos);
          } else {
            openMonacoEditor(fileName, result || '', this.wos);
          }
        } catch (error) {
          // File doesn't exist or other error - open with empty content
          openMonacoEditor(fileName, '', this.wos);
        }
      } else {
        this.printLine('WASM not initialized', 'error');
      }
      return;
    }

    // Execute via WASM if available
    if (this.wos) {
      try {
        const result = this.wos.executeCommand(cmd);
        this.printLine(result, 'output');

        // WOS-302: Log syscall trace for time-travel debugger
        if (this.wos._addTrace) {
          // Determine syscall type from command
          const syscallName = this.getSyscallTypeFromCommand(cmd);
          const isSuccess = !result.includes('Error') && !result.includes('error');
          const resultObj = isSuccess ? { Ok: result } : { Err: result };
          this.wos._addTrace({ [syscallName]: cmd }, resultObj, 1);
        }

        this.updateSystemInfo();
        // WOS-301: Update filesystem list after command execution (for file creation/deletion)
        this.updateFilesystemList();
      } catch (error) {
        this.printLine(`Error: ${error}`, 'error');

        // WOS-302: Log error trace
        if (this.wos && this.wos._addTrace) {
          this.wos._addTrace({ Error: cmd }, { Err: error.toString() }, 1);
        }
      }
    } else {
      this.printLine('WASM not initialized - command not executed', 'error');
    }
  }

  // WOS-302: Helper to determine syscall type from command string
  getSyscallTypeFromCommand(cmd) {
    const parts = cmd.trim().split(/\s+/);
    const command = parts[0];

    // Map command to syscall type
    const syscallMap = {
      'echo': 'Write',
      'cat': 'Read',
      'ls': 'Read',
      'ps': 'GetPid',
      'touch': 'Open',
      'rm': 'Close',
      'mkdir': 'Open',
      'cd': 'Read'
    };

    return syscallMap[command] || 'Unknown';
  }

  // WOS-305: Updated printHelp() to use HELP_DATA structure
  printHelp() {
    this.printLine('Available commands:', 'output');
    this.printLine('Type "help <command>" for detailed documentation', 'output');
    this.printLine('', 'output');

    // Print all commands from HELP_DATA
    Object.values(HELP_DATA).forEach(cmd => {
      const padding = ' '.repeat(Math.max(1, 15 - cmd.name.length));
      this.printLine(`  ${cmd.name}${padding}- ${cmd.description}`, 'output');
    });

    this.printLine('', 'output');
    this.printLine('Keyboard shortcuts:', 'output');
    this.printLine('  ↑/↓       - Navigate command history', 'output');
    this.printLine('  Ctrl+L    - Clear terminal', 'output');
    this.printLine('  F1        - Open help panel', 'output');
    this.printLine('', 'output');
  }

  // WOS-305: New method for detailed command help
  printDetailedHelp(commandName) {
    const helpData = HELP_DATA[commandName];

    if (!helpData) {
      this.printLine(`Unknown command: ${commandName}`, 'error');
      this.printLine('Type "help" to see available commands', 'output');
      return;
    }

    // Print command header
    this.printLine(`Command: ${helpData.name}`, 'success help-command');
    this.printLine('', 'output');

    // Print description
    this.printLine('DESCRIPTION:', 'output');
    this.printLine(`  ${helpData.description}`, 'output');
    this.printLine('', 'output');

    // Print usage
    this.printLine('USAGE:', 'output');
    this.printLine(`  ${helpData.usage}`, 'output');
    this.printLine('', 'output');

    // Print options if available
    if (helpData.options && helpData.options.length > 0) {
      this.printLine('OPTIONS:', 'output');
      helpData.options.forEach(opt => {
        const padding = ' '.repeat(Math.max(1, 20 - opt.flag.length));
        this.printLine(`  ${opt.flag}${padding}${opt.description}`, 'output');
      });
      this.printLine('', 'output');
    }

    // Print examples
    if (helpData.examples && helpData.examples.length > 0) {
      this.printLine('EXAMPLES:', 'output');
      helpData.examples.forEach(example => {
        this.printLine(`  $ ${example.command}`, 'command');
        this.printLine(`    ${example.description}`, 'output');
      });
      this.printLine('', 'output');
    }

    // Print related commands
    if (helpData.related && helpData.related.length > 0) {
      this.printLine('SEE ALSO:', 'output');
      this.printLine(`  ${helpData.related.join(', ')}`, 'output');
      this.printLine('', 'output');
    }
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

  printConfig() {
    if (!this.configManager) {
      this.printLine('Config manager not available', 'error');
      return;
    }

    const config = this.configManager.getConfig();
    if (!config) {
      this.printLine('No configuration loaded', 'error');
      return;
    }

    this.printLine('Current Configuration:', 'output');
    this.printLine('', 'output');
    this.printLine(`  Version: ${config.version}`, 'output');
    this.printLine(`  Environment: ${config.environment}`, 'output');

    if (config.ui) {
      this.printLine('', 'output');
      this.printLine('  UI Settings:', 'output');
      this.printLine(`    Mode: ${config.ui.mode}`, 'output');
      this.printLine(`    Theme: ${config.ui.theme}`, 'output');
      this.printLine(`    Terminal history size: ${config.ui.terminal?.history_size || 1000}`, 'output');
    }
    this.printLine('', 'output');
    this.printLine('Available commands:', 'output');
    this.printLine('  theme dark   - Switch to dark theme', 'output');
    this.printLine('  theme light  - Switch to light theme', 'output');
    this.printLine('  theme auto   - Auto theme based on system', 'output');
  }

  setTheme(theme) {
    try {
      if (!this.configManager) {
        this.printLine('Config manager not available', 'error');
        return;
      }

      // Update theme in config and save to localStorage
      const config = this.configManager.getConfig();
      if (!config || !config.ui) {
        this.printLine('Configuration error', 'error');
        return;
      }

      config.ui.theme = theme;
      localStorage.setItem('wos-config', JSON.stringify(config));

      // Apply the theme to the DOM
      this.configManager.applyConfig();

      this.printLine(`Theme set to: ${theme}`, 'success');
    } catch (error) {
      this.printLine(`Error setting theme: ${error.message}`, 'error');
    }
  }

  updateSystemInfo() {
    if (!this.wos) return;

    try {
      // Get basic system info
      const processCount = this.wos.processCount();
      document.getElementById('process-count').textContent = processCount;

      // Update System Monitor Panel
      this.updateSystemMonitor(processCount);
    } catch (error) {
      console.error('Error updating system info:', error);
    }
  }

  updateSystemMonitor(processCount) {
    if (!this.wos) return;

    // Get process count from WOS if not provided
    if (processCount === undefined || processCount === null) {
      processCount = this.wos.processCount ? this.wos.processCount() : 0;
    }

    // Track syscall count for rate calculation
    if (!this.syscallCount) {
      this.syscallCount = 0;
      this.lastSyscallTime = Date.now();
    }

    // Calculate actual CPU usage based on real browser workload metrics
    let activityBonus = 0;
    if (this.benchmarkRunning && this.benchmarkWorkloadMetrics) {
      // Calculate CPU bonus based on actual workload execution time
      // Measure recent workload in last update cycle (approximation)
      const recentWorkloadMs = this.benchmarkWorkloadMetrics.totalWorkloadTime;

      // Convert workload time to CPU percentage (heuristic: >50ms = high load)
      // This creates measurable variation based on ACTUAL computation
      if (recentWorkloadMs > 100) {
        activityBonus = 40; // Heavy load
      } else if (recentWorkloadMs > 50) {
        activityBonus = 30; // Moderate load
      } else if (recentWorkloadMs > 20) {
        activityBonus = 20; // Light load
      } else if (recentWorkloadMs > 0) {
        activityBonus = 10; // Minimal load
      }

      // Add variation based on array/prime operation counts (visible real activity)
      const operationDensity = this.benchmarkWorkloadMetrics.primeCalculations / 10000;
      activityBonus += Math.min(Math.floor(operationDensity * 20), 30);
    }

    // Calculate CPU usage based on process activity and actual workload
    const baseCpu = processCount > 0 ? Math.min(5 + (processCount * 8), 100) : 0;
    const cpuUsage = Math.min(baseCpu + activityBonus, 100);
    document.getElementById('monitor-cpu').innerHTML = `${cpuUsage}<span class="monitor-unit">%</span>`;
    document.getElementById('monitor-cpu-bar').style.width = `${cpuUsage}%`;

    // Show different status based on actual load
    let cpuInfo = processCount > 0 ? `Scheduler: Running (${processCount} proc)` : 'Scheduler: Idle';
    if (this.benchmarkRunning) {
      cpuInfo = `Benchmark: Active load (${cpuUsage}% CPU)`;
    }
    document.getElementById('monitor-cpu-info').textContent = cpuInfo;

    // Memory usage - get from memory panel or simulate
    const memTotal = 4096; // 4MB total (from memory panel)
    const memUsed = processCount * 128; // ~128KB per process (simulated)
    const memPercent = Math.min((memUsed / memTotal) * 100, 100).toFixed(1);
    document.getElementById('monitor-memory').innerHTML = `${memUsed}<span class="monitor-unit">KB</span>`;
    document.getElementById('monitor-memory-bar').style.width = `${memPercent}%`;
    document.getElementById('monitor-memory-info').textContent = `${memUsed} KB / ${memTotal} KB`;

    // Also update the old memory panel for consistency
    document.getElementById('mem-used').textContent = `${memUsed} KB`;
    document.getElementById('mem-free').textContent = `${memTotal - memUsed} KB`;
    document.getElementById('mem-percent').textContent = `${memPercent}%`;

    // Process count
    const runningProcs = Math.max(processCount - 1, 0); // Exclude init
    document.getElementById('monitor-processes').textContent = processCount;
    document.getElementById('monitor-process-info').textContent =
      `${runningProcs} running${processCount > 0 ? ', 1 init' : ''}`;

    // Syscall activity - track rate
    const now = Date.now();
    const timeDelta = (now - this.lastSyscallTime) / 1000; // seconds

    // Increment syscall count (simulated based on activity)
    if (processCount > 0) {
      this.syscallCount += processCount * 2; // Simulate 2 syscalls per process per update
    }

    const syscallRate = timeDelta > 0 ? Math.floor(this.syscallCount / timeDelta) : 0;
    document.getElementById('monitor-syscalls').textContent = this.syscallCount;
    document.getElementById('monitor-syscall-info').textContent = `${syscallRate} /sec`;

    // Reset rate calculation every 5 seconds
    if (timeDelta >= 5) {
      this.syscallCount = 0;
      this.lastSyscallTime = now;
    }

    // Update detailed visual panels
    this.updateProcessTable();
    this.updateMemoryView();
  }

  // WOS-301: Update process table with live process data
  updateProcessTable() {
    if (!this.wos) return;

    try {
      // Get process list using ps command
      const result = this.wos.executeCommand('ps');
      if (!result || typeof result !== 'string') return;

      const tbody = document.getElementById('process-table-body');
      if (!tbody) return;

      // WOS-301: Remember currently selected process to preserve selection across updates
      let selectedPid = null;
      const selectedRow = tbody.querySelector('tr.selected');
      if (selectedRow) {
        selectedPid = selectedRow.dataset.pid;
      }

      // Parse ps output to get process information
      // Expected format (tab-separated): PID\tSTATE\tPARENT
      const lines = result.trim().split('\n');
      const processes = [];

      // Skip header lines (PID STATE PARENT, separator line)
      let startIdx = 0;
      for (let i = 0; i < Math.min(3, lines.length); i++) {
        if (lines[i]?.includes('PID') || lines[i]?.includes('---')) {
          startIdx = i + 1;
        }
      }

      for (let i = startIdx; i < lines.length; i++) {
        const line = lines[i].trim();
        if (!line || line.includes('No processes')) continue;

        // Split by tabs or multiple spaces (handles both tab and space-separated output)
        const parts = line.split(/[\t\s]+/);
        if (parts.length >= 2) {
          // Extract PID, STATE, and PARENT
          const pid = parts[0]?.trim();
          const state = parts[1]?.trim() || 'Ready';
          const parent = parts[2]?.trim() || '-';

          // Determine command based on PID (1 is init, others unknown)
          const command = pid === '1' ? 'init' : 'process';

          if (pid) {
            processes.push({
              pid,
              state,
              parent,
              command,
              // Simulated data for now
              cpuTime: `${(Math.random() * 5).toFixed(2)}ms`,
              memory: `${Math.floor(128 + Math.random() * 128)}KB`
            });
          }
        }
      }

      // Clear existing rows
      tbody.innerHTML = '';

      if (processes.length === 0) {
        tbody.innerHTML = '<tr aria-label="No processes currently running"><td colspan="6" class="no-data">No processes running</td></tr>';
        return;
      }

      // Add process rows
      processes.forEach(proc => {
        const row = document.createElement('tr');
        const stateClass = `state-${proc.state.toLowerCase()}`;

        row.innerHTML = `
          <td>${proc.pid}</td>
          <td><span class="process-state ${stateClass}">${proc.state}</span></td>
          <td>${proc.parent}</td>
          <td>${proc.cpuTime}</td>
          <td>${proc.memory}</td>
          <td>${proc.command}</td>
        `;

        // Set attributes AFTER innerHTML to ensure they're not cleared
        row.setAttribute('tabindex', '0');
        row.setAttribute('aria-label', `Process ${proc.pid}, state ${proc.state}, parent ${proc.parent}`);
        row.dataset.pid = proc.pid;

        // WOS-301: Restore 'selected' class if this was the previously selected process
        if (proc.pid === selectedPid) {
          row.classList.add('selected');
        }

        // Add click handler for row selection
        row.addEventListener('click', (e) => {
          // Remove selection from other rows
          tbody.querySelectorAll('tr').forEach(r => r.classList.remove('selected'));
          // Add selection to clicked row
          row.classList.add('selected');
          // Highlight memory regions for this process
          this.highlightProcessMemory(proc.pid);
        });

        // Add keyboard navigation
        row.addEventListener('keydown', (e) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            row.click();
          }
        });

        tbody.appendChild(row);
      });
    } catch (error) {
      tracer.error('VISUAL_MONITOR', 'Failed to update process table', error);
    }
  }

  // WOS-301: Update memory view with graphical bars
  updateMemoryView() {
    if (!this.wos) return;

    try {
      // Get memory statistics
      const processCount = this.wos.processCount();
      const memTotal = 4096; // 4MB total
      const memUsed = processCount * 128; // ~128KB per process

      // Update memory summary
      document.getElementById('mem-total').textContent = `${memTotal} KB`;
      document.getElementById('mem-used').textContent = `${memUsed} KB`;
      document.getElementById('mem-free').textContent = `${memTotal - memUsed} KB`;
      const memPercent = ((memUsed / memTotal) * 100).toFixed(1);
      document.getElementById('mem-percent').textContent = `${memPercent}%`;

      // Update memory segment bars (simulated data)
      // Code segment: typically 100% allocated
      const codeUsage = 100;
      this.updateSegmentBar('code', codeUsage, 16, 16);

      // Data segment: varies with process count
      const dataUsage = Math.min((processCount * 4) / 16 * 100, 100);
      this.updateSegmentBar('data', dataUsage, processCount * 4, 16);

      // Heap: varies with memory usage
      const heapUsed = Math.floor(memUsed * 0.6); // 60% of used memory is heap
      const heapUsage = (heapUsed / 256) * 100;
      this.updateSegmentBar('heap', heapUsage, heapUsed / 1024, 256); // Convert KB to MB

      // Stack: smaller allocation
      const stackUsed = Math.floor(memUsed * 0.15); // 15% of used memory is stack
      const stackUsage = (stackUsed / (8 * 1024)) * 100; // 8MB max
      this.updateSegmentBar('stack', stackUsage, stackUsed / 1024, 8); // Convert KB to MB

      // Update page counts
      const pageSize = 4; // 4KB pages
      const totalPages = memTotal / pageSize;
      const allocatedPages = Math.floor(memUsed / pageSize);
      const freePages = totalPages - allocatedPages;

      document.getElementById('mem-free-pages').textContent = freePages;
      document.getElementById('mem-allocated-pages').textContent = allocatedPages;
      document.getElementById('mem-total-pages').textContent = totalPages;
    } catch (error) {
      tracer.error('VISUAL_MONITOR', 'Failed to update memory view', error);
    }
  }

  // Helper: Update a memory segment bar
  updateSegmentBar(segment, percentage, used, max) {
    const bar = document.querySelector(`.memory-segment-bar[data-segment="${segment}"]`);
    const sizeSpan = document.querySelector(`.segment-size[data-segment="${segment}"]`);

    if (bar) {
      const clampedPercent = Math.min(Math.max(percentage, 0), 100);
      bar.style.width = `${clampedPercent.toFixed(1)}%`;

      // Update tooltip
      const usedFormatted = used < 1 ? `${(used * 1024).toFixed(0)} KB` : `${used.toFixed(1)} MB`;
      const maxFormatted = max < 1 ? `${(max * 1024).toFixed(0)} KB` : `${max} MB`;
      bar.title = `${segment.charAt(0).toUpperCase() + segment.slice(1)}: ${usedFormatted} / ${maxFormatted} (${clampedPercent.toFixed(1)}%)`;
    }

    if (sizeSpan) {
      const usedFormatted = used < 1 ? `${(used * 1024).toFixed(0)} KB` : `${used.toFixed(1)} MB`;
      sizeSpan.textContent = usedFormatted;
    }
  }

  // Helper: Highlight memory regions for selected process
  highlightProcessMemory(pid) {
    // Remove previous highlights
    document.querySelectorAll('.memory-segment-bar').forEach(bar => {
      bar.classList.remove('highlighted');
    });

    // Highlight relevant segments (simulated - in real implementation would query actual process memory map)
    // For now, highlight heap and stack for all processes
    document.querySelectorAll('.memory-segment-bar[data-segment="heap"]').forEach(bar => {
      bar.classList.add('highlighted');
    });
    document.querySelectorAll('.memory-segment-bar[data-segment="stack"]').forEach(bar => {
      bar.classList.add('highlighted');
    });

    tracer.debug('VISUAL_MONITOR', `Highlighted memory regions for process ${pid}`);
  }

  // WOS-301: Update filesystem list from WOS filesystem
  updateFilesystemList() {
    if (!this.wos) {
      tracer.warn('FILE', 'WOS not initialized in updateFilesystemList');
      return;
    }

    try {
      // WOS-FILE-EDIT-01: Use simple 'ls' command (ls -la is broken in WOS)
      const result = this.wos.executeCommand('ls');
      tracer.debug('FILE', 'ls result:', { result });

      // WOS-FILE-EDIT-01: Empty string is valid (means no files), only reject null/undefined
      if (result === null || result === undefined || typeof result !== 'string') {
        tracer.warn('FILE', 'Invalid ls result, returning');
        return;
      }

      const fileList = document.getElementById('file-list');
      if (!fileList) {
        tracer.warn('FILE', 'file-list element not found');
        return;
      }

      // Parse ls output - simple format, one file per line
      const lines = result.trim().split('\n');
      tracer.debug('FILE', 'Parsed lines:', { lines });
      const files = [];

      for (let i = 0; i < lines.length; i++) {
        const fileName = lines[i].trim();
        if (!fileName || fileName === '.' || fileName === '..') continue;

        // Skip directories (end with /)
        const isDirectory = fileName.endsWith('/');
        if (isDirectory) continue;

        tracer.debug('FILE', 'Adding file:', { fileName });
        files.push({
          name: fileName,
          isDirectory: false,
          size: '0'
        });
      }

      tracer.info('FILE', `Total files: ${files.length}`);

      // Clear existing file list
      fileList.innerHTML = '';

      // WOS-FILE-EDIT-01: Disable edit buttons when no files exist
      if (files.length === 0) {
        if (this.fileManager) {
          this.fileManager.selectedFile = null;
          document.getElementById('btn-edit').disabled = true;
          document.getElementById('btn-download').disabled = true;
          document.getElementById('btn-delete').disabled = true;
        }
        return;
      }

      // Add file items
      let firstFileItem = null;
      files.forEach(file => {
        const item = document.createElement('div');
        item.className = file.isDirectory ? 'file-item dir' : 'file-item';

        // Track first non-directory file for auto-selection
        if (!file.isDirectory && !firstFileItem) {
          firstFileItem = { item, file };
        }

        // Use appropriate icon
        const iconPath = file.isDirectory
          ? 'M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z'
          : 'M14 2H6c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h12c1.1 0 2-.9 2-2V8l-6-6zm2 16H8v-2h8v2zm0-4H8v-2h8v2zm-3-5V3.5L18.5 9H13z';

        item.innerHTML = `
          <div class="file-item-name">
            <svg class="file-item-icon ${file.isDirectory ? 'dir-icon' : 'file-icon'}" viewBox="0 0 24 24" fill="currentColor">
              <path d="${iconPath}"/>
            </svg>
            <span>${file.name}</span>
          </div>
          <span class="file-item-size">${file.size}</span>
        `;

        // Add click handler
        item.addEventListener('click', () => {
          // Remove selection from other items
          fileList.querySelectorAll('.file-item').forEach(f => f.classList.remove('selected'));
          item.classList.add('selected');

          // WOS-FILE-EDIT-01: Notify FileManager of selection to enable edit button
          if (!file.isDirectory && this.fileManager) {
            this.fileManager.selectFile(file.name);
          }
        });

        fileList.appendChild(item);
      });

      // WOS-FILE-EDIT-01: Auto-select first file if no selection exists
      if (firstFileItem && this.fileManager) {
        const hasSelection = fileList.querySelector('.file-item.selected');
        if (!hasSelection) {
          firstFileItem.item.classList.add('selected');
          this.fileManager.selectFile(firstFileItem.file.name);
        }
      }
    } catch (error) {
      tracer.error('VISUAL_MONITOR', 'Failed to update filesystem list', error);
    }
  }

  openVim(fileName) {
    if (!this.wos) {
      this.printLine('WASM not initialized - cannot open vim', 'error');
      return;
    }

    // Try to read file content from WASM filesystem
    let content = '';
    try {
      const result = this.wos.executeCommand(`cat ${fileName}`);
      // Check if file exists (cat returns error message if not)
      // Error format: "cat: filename: No such file or directory"
      if (!result.includes('No such file or directory')) {
        content = result;
      }
    } catch (error) {
      // File doesn't exist, start with empty content
      content = '';
    }

    const vim = new VimEditor(fileName, content, (newContent) => {
      // Save callback - write file back to WASM filesystem
      try {
        // Write file using echo with double quotes
        // Must escape: backslashes, dollar signs, double quotes, backticks
        // This is because echo "..." interprets these characters
        // We want the LITERAL content written to the file
        const escapedContent = newContent
          .replace(/\\/g, '\\\\')   // Escape backslashes first (\ -> \\)
          .replace(/\$/g, '\\$')    // Escape dollar signs ($ -> \$)
          .replace(/"/g, '\\"')     // Escape double quotes (" -> \")
          .replace(/`/g, '\\`')     // Escape backticks (` -> \`)
          .replace(/\n/g, '\\n');   // Convert newlines to \n for echo

        // Normalize path to absolute (add leading / if missing)
        const normalizedPath = fileName.startsWith('/') ? fileName : `/${fileName}`;
        this.wos.executeCommand(`echo "${escapedContent}" > ${normalizedPath}`);
        this.printLine(`File saved: ${normalizedPath}`, 'success');
      } catch (error) {
        this.printLine(`Error saving file: ${error}`, 'error');
      }
    });

    vim.open();
  }

  setWOS(wos) {
    this.wos = wos;
    this.fileManager = new FileManager(wos);
    this.updateSystemInfo();

    // Start automatic polling for system monitor and process list updates (WOS-301: 100ms updates)
    if (this.updateInterval) {
      clearInterval(this.updateInterval);
    }
    this.updateInterval = setInterval(() => {
      this.updateSystemInfo();
      this.updateSystemMonitor(); // Update process table and memory view
      // WOS-FILE-EDIT-01: Disabled - FileManager now handles file list updates
      // this.updateFilesystemList(); // Update file list from WOS filesystem
    }, 100); // Update every 100ms for real-time monitoring
  }

  refreshProcessList() {
    if (!this.wos) return;

    // Directly update the process table instead of clicking refresh button
    this.updateProcessTable();
  }
}

/**
 * WOS-302: Time-Travel Debugger
 *
 * Omniscient debugger for WOS kernel with time-travel capabilities.
 * Integrates with KernelHistory (kernel/src/trace.rs) to provide:
 * - Scrubbable timeline for state navigation
 * - Event log with filtering
 * - State inspector with diff highlighting
 * - Playback controls
 * - Keyboard navigation
 */
class TimeTravelDebugger {
  constructor(wos) {
    this.wos = wos;

    // Restore position from localStorage
    const savedPosition = localStorage.getItem('wos-debugger-position');
    this.currentPosition = savedPosition ? parseInt(savedPosition, 10) : 0;

    this.maxPosition = 0;
    this.traces = [];
    this.isPlaying = false;
    this.playbackInterval = null;
    this.lastRenderedPosition = -1; // Track last rendered position for diff highlighting
    this.filters = {
      pid: '',
      syscallType: '',
      showSuccess: true,
      showFailure: true
    };

    tracer.debug('DEBUGGER', 'TimeTravelDebugger constructor called');
    this.initializeElements();
    this.attachEventListeners();
    this.loadHistory();

    // Update timeline every 100ms for responsive debugging
    setInterval(() => this.loadHistory(), 100);

    tracer.debug('DEBUGGER', 'TimeTravelDebugger initialized');
  }

  initializeElements() {
    this.elements = {
      slider: document.getElementById('timeline-slider'),
      positionIndicator: document.getElementById('timeline-position-indicator'),
      traceCount: document.getElementById('trace-count'),
      ticks: document.getElementById('timeline-ticks'),
      tooltip: document.getElementById('timeline-tooltip'),

      playBtn: document.getElementById('playback-play'),
      pauseBtn: document.getElementById('playback-pause'),
      stepBackBtn: document.getElementById('playback-step-back'),
      stepForwardBtn: document.getElementById('playback-step-forward'),

      eventLog: document.getElementById('event-log'),
      eventLogBody: document.getElementById('event-log-body'),
      eventLogCanvas: document.getElementById('event-log-canvas'),

      filterPid: document.getElementById('filter-pid'),
      filterSyscallType: document.getElementById('filter-syscall-type'),
      filterSuccess: document.getElementById('filter-success'),
      filterFailure: document.getElementById('filter-failure'),

      stateInspector: document.getElementById('state-inspector'),
      stateProcesses: document.getElementById('state-inspector-processes'),
      stateMemory: document.getElementById('state-inspector-memory'),
      stateFilesystem: document.getElementById('state-inspector-filesystem'),

      eventDetailsPanel: document.getElementById('event-details-panel'),
      eventDetailsContent: document.getElementById('event-details-content'),
      eventDetailsClose: document.getElementById('event-details-close'),

      exportBtn: document.getElementById('export-traces-json'),
      loadingIndicator: document.getElementById('state-loading-indicator')
    };
  }

  attachEventListeners() {
    // Timeline slider
    if (this.elements.slider) {
      this.elements.slider.addEventListener('input', () => this.onSliderChange());
      this.elements.slider.addEventListener('mousemove', (e) => this.onSliderHover(e));
      this.elements.slider.addEventListener('mouseleave', () => this.hideTooltip());

      // Keyboard navigation
      this.elements.slider.addEventListener('keydown', (e) => this.onKeyDown(e));
      this.elements.slider.focus();
    }

    // Playback controls
    if (this.elements.playBtn) {
      this.elements.playBtn.addEventListener('click', () => this.play());
    }
    if (this.elements.pauseBtn) {
      this.elements.pauseBtn.addEventListener('click', () => this.pause());
    }
    if (this.elements.stepBackBtn) {
      this.elements.stepBackBtn.addEventListener('click', () => this.stepBack());
    }
    if (this.elements.stepForwardBtn) {
      this.elements.stepForwardBtn.addEventListener('click', () => this.stepForward());
    }

    // Filters
    if (this.elements.filterPid) {
      this.elements.filterPid.addEventListener('input', () => {
        this.filters.pid = this.elements.filterPid.value;
        this.updateEventLog();
      });
    }
    if (this.elements.filterSyscallType) {
      this.elements.filterSyscallType.addEventListener('change', () => {
        this.filters.syscallType = this.elements.filterSyscallType.value;
        this.updateEventLog();
      });
    }
    if (this.elements.filterSuccess) {
      this.elements.filterSuccess.addEventListener('change', () => {
        this.filters.showSuccess = this.elements.filterSuccess.checked;
        this.updateEventLog();
      });
    }
    if (this.elements.filterFailure) {
      this.elements.filterFailure.addEventListener('change', () => {
        this.filters.showFailure = this.elements.filterFailure.checked;
        this.updateEventLog();
      });
    }

    // Event details close button
    if (this.elements.eventDetailsClose) {
      this.elements.eventDetailsClose.addEventListener('click', () => this.closeEventDetails());
    }

    // Export button
    if (this.elements.exportBtn) {
      this.elements.exportBtn.addEventListener('click', () => this.exportTracesJSON());
    }
  }

  loadHistory() {
    if (!this.wos) return;

    try {
      // Get kernel history from WASM backend
      const historyJson = this.wos.getKernelHistory ? this.wos.getKernelHistory() : null;

      if (historyJson) {
        this.traces = JSON.parse(historyJson);
        this.maxPosition = this.traces.length;
        this.currentPosition = Math.min(this.currentPosition, this.maxPosition);

        // Sync WASM position with debugger position
        if (this.wos && this.wos.jumpToPosition) {
          this.wos.jumpToPosition(this.currentPosition);
        }

        this.updateTimeline();
        this.updateEventLog();

        // Only update state inspector if position changed (preserves diff highlighting)
        if (this.currentPosition !== this.lastRenderedPosition) {
          this.updateStateInspector();
          this.lastRenderedPosition = this.currentPosition;
        }

        this.updateTraceCount();
      }
    } catch (error) {
      tracer.error('DEBUGGER', 'Failed to load history', error);
    }
  }

  updateTimeline() {
    if (!this.elements.slider) return;

    this.elements.slider.max = Math.max(0, this.maxPosition - 1);
    this.elements.slider.value = this.currentPosition;

    // Update position indicator
    const currentTrace = this.traces[this.currentPosition];
    const timestamp = currentTrace ? currentTrace.timestamp_us / 1000 : 0;
    if (this.elements.positionIndicator) {
      this.elements.positionIndicator.textContent = `${timestamp.toFixed(2)}ms`;
    }

    // Update tick marks
    this.updateTickMarks();

    // Update button states
    this.updateButtonStates();
  }

  updateTickMarks() {
    if (!this.elements.ticks) return;

    this.elements.ticks.innerHTML = '';

    // Add tick mark every 10% of timeline
    const tickCount = Math.min(10, this.maxPosition);
    for (let i = 0; i <= tickCount; i++) {
      const position = Math.floor((i / tickCount) * this.maxPosition);
      const trace = this.traces[position];

      if (trace) {
        const tick = document.createElement('div');
        tick.className = 'tick-mark';
        tick.style.left = `${(position / this.maxPosition) * 100}%`;
        tick.title = `${trace.timestamp_us / 1000}ms`;
        this.elements.ticks.appendChild(tick);
      }
    }
  }

  updateEventLog() {
    if (!this.elements.eventLogBody) return;

    // Filter traces up to current position
    const visibleTraces = this.traces.slice(0, this.currentPosition + 1);
    const filteredTraces = this.filterTraces(visibleTraces);

    // Switch to canvas rendering if > 1000 events
    if (filteredTraces.length > 1000) {
      this.renderEventLogCanvas(filteredTraces);
    } else {
      this.renderEventLogTable(filteredTraces);
    }
  }

  filterTraces(traces) {
    return traces.filter(trace => {
      // Filter by PID
      if (this.filters.pid && trace.calling_pid.toString() !== this.filters.pid) {
        return false;
      }

      // Filter by syscall type
      if (this.filters.syscallType) {
        const syscallName = this.getSyscallName(trace.syscall);
        if (!syscallName.includes(this.filters.syscallType)) {
          return false;
        }
      }

      // Filter by success/failure
      const isSuccess = trace.result && trace.result.Ok !== undefined;
      if (isSuccess && !this.filters.showSuccess) {
        return false;
      }
      if (!isSuccess && !this.filters.showFailure) {
        return false;
      }

      return true;
    });
  }

  renderEventLogTable(traces) {
    if (this.elements.eventLogCanvas) {
      this.elements.eventLogCanvas.classList.add('hidden');
    }

    this.elements.eventLogBody.innerHTML = '';

    if (traces.length === 0) {
      const row = document.createElement('tr');
      row.className = 'no-data';
      row.innerHTML = '<td colspan="4">No events match the current filters</td>';
      this.elements.eventLogBody.appendChild(row);
      return;
    }

    traces.forEach((trace, index) => {
      const row = document.createElement('tr');
      row.className = 'event-item';

      if (index === this.currentPosition) {
        row.classList.add('selected');
      }

      const isSuccess = trace.result && trace.result.Ok !== undefined;
      const resultIcon = isSuccess ? '✓' : '✗';
      const resultClass = isSuccess ? 'success' : 'failure';

      row.innerHTML = `
        <td class="event-time">${(trace.timestamp_us / 1000).toFixed(2)}ms</td>
        <td class="event-pid">${trace.calling_pid}</td>
        <td class="event-syscall">${this.getSyscallName(trace.syscall)}</td>
        <td class="event-result ${resultClass}">${resultIcon}</td>
      `;

      row.addEventListener('click', () => this.showEventDetails(trace));

      this.elements.eventLogBody.appendChild(row);
    });
  }

  renderEventLogCanvas(traces) {
    // Show canvas, hide table
    if (this.elements.eventLogCanvas) {
      this.elements.eventLogCanvas.classList.remove('hidden');
    }

    this.elements.eventLogBody.innerHTML = '';

    // Canvas rendering for performance (implementation simplified for now)
    tracer.debug('DEBUGGER', `Using canvas rendering for ${traces.length} events`);
  }

  getSyscallName(syscall) {
    if (typeof syscall === 'string') {
      return syscall;
    }

    // Handle different syscall formats
    if (syscall.Write) return 'Write';
    if (syscall.Read) return 'Read';
    if (syscall.Fork) return 'Fork';
    if (syscall.Exec) return 'Exec';
    if (syscall.Exit) return 'Exit';
    if (syscall.Open) return 'Open';
    if (syscall.Close) return 'Close';
    if (syscall.GetPid) return 'GetPid';
    if (syscall.Kill) return 'Kill';

    return JSON.stringify(syscall);
  }

  updateStateInspector() {
    if (!this.wos || !this.wos.getCurrentState) return;

    try {
      this.showLoading(true);

      const stateJson = this.wos.getCurrentState();
      const state = JSON.parse(stateJson);

      // Store previous state for diff highlighting
      // Use deep copy to avoid reference issues
      const prevState = this.previousState;
      this.previousState = JSON.parse(JSON.stringify(state));

      this.renderProcessState(state.processes || {}, prevState?.processes);
      this.renderMemoryState(state.memory || {}, prevState?.memory);
      this.renderFilesystemState(state.filesystem || {}, prevState?.filesystem);

      this.showLoading(false);
    } catch (error) {
      tracer.error('DEBUGGER', 'Failed to update state inspector', error);
      this.showLoading(false);
    }
  }

  renderProcessState(processes, prevProcesses) {
    if (!this.elements.stateProcesses) return;

    this.elements.stateProcesses.innerHTML = '';

    const processArray = Object.entries(processes);
    if (processArray.length === 0) {
      this.elements.stateProcesses.innerHTML = '<p class="no-data">No processes in current state</p>';
      return;
    }

    processArray.forEach(([pid, process]) => {
      const item = document.createElement('div');
      item.className = 'process-state-item';
      item.setAttribute('aria-expanded', 'false');

      const header = document.createElement('div');
      header.className = 'state-item-header';
      header.textContent = `Process ${pid}`;

      // Check if this process is new or changed
      const prevProcess = prevProcesses?.[pid];
      const isNew = !prevProcess;
      const isChanged = prevProcess && JSON.stringify(process) !== JSON.stringify(prevProcess);

      const details = document.createElement('div');
      details.className = 'process-details hidden';

      // Add diff highlighting to changed fields
      const stateClass = (isNew || (prevProcess && process.state !== prevProcess.state)) ? 'state-diff' : '';
      const memoryClass = (isNew || (prevProcess && process.memory !== prevProcess.memory)) ? 'state-diff' : '';

      details.innerHTML = `
        <p><strong>PID:</strong> ${pid}</p>
        <p class="${stateClass}"><strong>State:</strong> ${process.state || 'Unknown'}</p>
        <p><strong>Parent:</strong> ${process.parent_pid || '-'}</p>
        <p class="${memoryClass}"><strong>Memory:</strong> ${this.formatMemorySize(process.memory || 0)}</p>
      `;

      // Add click handler to item (not just header) for accessibility
      item.addEventListener('click', () => {
        const isExpanded = item.getAttribute('aria-expanded') === 'true';
        item.setAttribute('aria-expanded', String(!isExpanded));
        details.classList.toggle('hidden');
      });

      item.appendChild(header);
      item.appendChild(details);
      this.elements.stateProcesses.appendChild(item);
    });
  }

  renderMemoryState(memory, prevMemory) {
    if (!this.elements.stateMemory) return;

    // Add diff highlighting to changed memory values
    const usedClass = (prevMemory && memory.used !== prevMemory.used) ? 'state-diff' : '';
    const freeClass = (prevMemory && memory.free !== prevMemory.free) ? 'state-diff' : '';

    this.elements.stateMemory.innerHTML = `
      <p><strong>Total:</strong> ${this.formatMemorySize(memory.total || 0)}</p>
      <p class="${usedClass}"><strong>Used:</strong> ${this.formatMemorySize(memory.used || 0)}</p>
      <p class="${freeClass}"><strong>Free:</strong> ${this.formatMemorySize(memory.free || 0)}</p>
    `;
  }

  renderFilesystemState(filesystem, prevFilesystem) {
    if (!this.elements.stateFilesystem) return;

    this.elements.stateFilesystem.innerHTML = '';

    const files = filesystem.files || [];
    if (files.length === 0) {
      this.elements.stateFilesystem.innerHTML = '<p class="no-data">No files in current state</p>';
      return;
    }

    const prevFiles = prevFilesystem?.files || [];

    files.forEach(file => {
      const item = document.createElement('div');
      const filePath = file.path || file;

      // Highlight new files
      const isNew = !prevFiles.includes(filePath) && !prevFiles.find(f => (f.path || f) === filePath);
      item.className = isNew ? 'file-tree-item state-diff' : 'file-tree-item';
      item.textContent = filePath;
      this.elements.stateFilesystem.appendChild(item);
    });
  }

  formatMemorySize(bytes) {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }

  showEventDetails(trace) {
    if (!this.elements.eventDetailsPanel || !this.elements.eventDetailsContent) return;

    const isSuccess = trace.result && trace.result.Ok !== undefined;
    const resultData = isSuccess ? trace.result.Ok : trace.result.Err;

    this.elements.eventDetailsContent.innerHTML = `
      <h4>Trace #${trace.trace_id}</h4>
      <p><strong>Timestamp:</strong> ${(trace.timestamp_us / 1000).toFixed(2)}ms</p>
      <p><strong>PID:</strong> ${trace.calling_pid}</p>
      <p><strong>Syscall:</strong> ${this.getSyscallName(trace.syscall)}</p>
      <p><strong>Result:</strong> ${isSuccess ? 'Success' : 'Failure'}</p>
      <h5>Input:</h5>
      <pre>${JSON.stringify(trace.syscall, null, 2)}</pre>
      <h5>Output:</h5>
      <pre>${JSON.stringify(resultData, null, 2)}</pre>
    `;

    this.elements.eventDetailsPanel.classList.remove('hidden');
  }

  closeEventDetails() {
    if (this.elements.eventDetailsPanel) {
      this.elements.eventDetailsPanel.classList.add('hidden');
    }
  }

  showLoading(show) {
    if (this.elements.loadingIndicator) {
      this.elements.loadingIndicator.classList.toggle('hidden', !show);
    }
  }

  updateTraceCount() {
    if (this.elements.traceCount) {
      this.elements.traceCount.textContent = `${this.traces.length} events`;
    }
  }

  updateButtonStates() {
    if (this.elements.stepBackBtn) {
      this.elements.stepBackBtn.disabled = this.currentPosition === 0;
    }
    if (this.elements.stepForwardBtn) {
      this.elements.stepForwardBtn.disabled = this.currentPosition >= this.maxPosition - 1;
    }
  }

  onSliderChange() {
    this.currentPosition = parseInt(this.elements.slider.value);

    // Save position to localStorage
    localStorage.setItem('wos-debugger-position', String(this.currentPosition));

    // Jump to position in kernel history
    if (this.wos && this.wos.jumpToPosition) {
      this.wos.jumpToPosition(this.currentPosition);
    }

    this.updateTimeline();
    this.updateEventLog();
    this.updateStateInspector();
    this.lastRenderedPosition = this.currentPosition;
  }

  onSliderHover(e) {
    if (!this.elements.tooltip || !this.elements.slider) return;

    const rect = this.elements.slider.getBoundingClientRect();
    const x = e.clientX - rect.left;
    const percentage = x / rect.width;
    const position = Math.floor(percentage * this.maxPosition);
    const trace = this.traces[position];

    if (trace) {
      this.elements.tooltip.textContent = `${(trace.timestamp_us / 1000).toFixed(2)}ms`;
      this.elements.tooltip.style.left = `${x}px`;
      this.elements.tooltip.classList.remove('hidden');
    }
  }

  hideTooltip() {
    if (this.elements.tooltip) {
      this.elements.tooltip.classList.add('hidden');
    }
  }

  onKeyDown(e) {
    switch (e.key) {
      case 'ArrowLeft':
        e.preventDefault();
        this.stepBack();
        break;
      case 'ArrowRight':
        e.preventDefault();
        this.stepForward();
        break;
      case 'Home':
        e.preventDefault();
        this.jumpTo(0);
        break;
      case 'End':
        e.preventDefault();
        this.jumpTo(this.maxPosition);
        break;
      case ' ':
        e.preventDefault();
        if (this.isPlaying) {
          this.pause();
        } else {
          this.play();
        }
        break;
    }
  }

  play() {
    if (this.isPlaying) return;

    this.isPlaying = true;
    this.playbackInterval = setInterval(() => {
      if (this.currentPosition < this.maxPosition - 1) {
        this.stepForward();
      } else {
        this.pause();
      }
    }, 100); // Advance 10 events per second
  }

  pause() {
    this.isPlaying = false;
    if (this.playbackInterval) {
      clearInterval(this.playbackInterval);
      this.playbackInterval = null;
    }
  }

  stepBack() {
    if (this.currentPosition > 0) {
      this.jumpTo(this.currentPosition - 1);
    }
  }

  stepForward() {
    if (this.currentPosition < this.maxPosition - 1) {
      this.jumpTo(this.currentPosition + 1);
    }
  }

  jumpTo(position) {
    this.currentPosition = Math.max(0, Math.min(position, this.maxPosition - 1));
    this.elements.slider.value = this.currentPosition;

    if (this.wos && this.wos.jumpToPosition) {
      this.wos.jumpToPosition(this.currentPosition);
    }

    this.updateTimeline();
    this.updateEventLog();
    this.updateStateInspector();
  }

  exportTracesJSON() {
    const jsonData = JSON.stringify(this.traces, null, 2);
    const blob = new Blob([jsonData], { type: 'application/json' });
    const url = URL.createObjectURL(blob);

    const a = document.createElement('a');
    a.href = url;
    a.download = `wos-traces-${new Date().toISOString()}.json`;
    document.body.appendChild(a);
    a.click();
    document.body.removeChild(a);
    URL.revokeObjectURL(url);
  }
}



// WOS-305: Contextual Help Panel
class HelpPanel {
  constructor() {
    this.helpCommandList = document.getElementById('help-command-list');
    this.searchInput = document.getElementById('help-search');
    this.searchResultCount = document.getElementById('search-result-count');
    this.helpPanel = document.getElementById('panel-help');
    this.btnHelp = document.getElementById('btn-help');

    this.populateCommandList();
    this.setupEventListeners();
  }

  populateCommandList() {
    if (!this.helpCommandList) return;

    this.helpCommandList.innerHTML = '';

    Object.values(HELP_DATA).forEach(cmdData => {
      const item = document.createElement('div');
      item.className = 'help-command-item';
      item.setAttribute('role', 'listitem');
      item.setAttribute('data-command-name', cmdData.name);
      item.setAttribute('data-command-description', cmdData.description);
      item.setAttribute('tabindex', '0');

      const header = document.createElement('div');
      header.className = 'help-command-header';
      header.innerHTML = `
        <span class="help-command-name">${cmdData.name}</span>
        <span class="help-command-description">${cmdData.description}</span>
      `;

      const details = document.createElement('div');
      details.className = 'help-command-details hidden';
      details.innerHTML = `
        <div class="help-section">
          <strong>Usage:</strong> <code>${cmdData.usage}</code>
        </div>
        ${cmdData.options && cmdData.options.length > 0 ? `
          <div class="help-section">
            <strong>Options:</strong>
            <ul class="help-options-list">
              ${cmdData.options.map(opt => `<li><code>${opt.flag}</code> - ${opt.description}</li>`).join('')}
            </ul>
          </div>
        ` : ''}
        ${cmdData.examples && cmdData.examples.length > 0 ? `
          <div class="help-section">
            <strong>Examples:</strong>
            <ul class="help-examples-list">
              ${cmdData.examples.map(ex => `<li><code>${ex.command}</code> - ${ex.description}</li>`).join('')}
            </ul>
          </div>
        ` : ''}
        ${cmdData.related && cmdData.related.length > 0 ? `
          <div class="help-section">
            <strong>See also:</strong> ${cmdData.related.join(', ')}
          </div>
        ` : ''}
      `;

      // Initialize aria-expanded
      item.setAttribute('aria-expanded', 'false');

      item.appendChild(header);
      item.appendChild(details);
      this.helpCommandList.appendChild(item);

      // Click anywhere on item to expand
      item.addEventListener('click', () => this.toggleCommandDetails(item));
      item.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' || e.key === ' ') {
          e.preventDefault();
          this.toggleCommandDetails(item);
        }
      });
    });

    this.updateSearchResults(Object.keys(HELP_DATA).length);
  }

  toggleCommandDetails(item) {
    const details = item.querySelector('.help-command-details');
    const isHidden = details.classList.contains('hidden');

    // Close all other expanded items and update their aria-expanded
    this.helpCommandList.querySelectorAll('.help-command-item').forEach(otherItem => {
      if (otherItem !== item) {
        const otherDetails = otherItem.querySelector('.help-command-details');
        if (otherDetails) {
          otherDetails.classList.add('hidden');
          otherItem.setAttribute('aria-expanded', 'false');
        }
      }
    });

    // Toggle this item
    details.classList.toggle('hidden');
    item.setAttribute('aria-expanded', isHidden ? 'true' : 'false');
  }

  setupEventListeners() {
    // Search functionality
    if (this.searchInput) {
      this.searchInput.addEventListener('input', (e) => this.handleSearch(e.target.value));
      this.searchInput.addEventListener('keydown', (e) => {
        if (e.key === 'Escape') {
          this.searchInput.value = '';
          this.handleSearch('');
        } else if (e.key === 'ArrowDown') {
          e.preventDefault();
          this.focusFirstHelpItem();
        } else if (e.key === 'ArrowUp') {
          e.preventDefault();
          this.focusLastHelpItem();
        }
      });
    }

    // Arrow key navigation for help items
    if (this.helpCommandList) {
      this.helpCommandList.addEventListener('keydown', (e) => {
        if (e.target.classList.contains('help-command-item')) {
          if (e.key === 'ArrowDown') {
            e.preventDefault();
            this.focusNextHelpItem(e.target);
          } else if (e.key === 'ArrowUp') {
            e.preventDefault();
            this.focusPreviousHelpItem(e.target);
          } else if (e.key === 'Escape') {
            e.preventDefault();
            if (this.searchInput) {
              this.searchInput.focus();
            }
          }
        }
      });
    }

    // F1 keyboard shortcut to toggle help panel, Escape to close
    document.addEventListener('keydown', (e) => {
      if (e.key === 'F1') {
        e.preventDefault();
        this.toggleHelpPanel();
        // Focus search input when opening
        if (this.helpPanel && !this.helpPanel.classList.contains('collapsed') && this.searchInput) {
          setTimeout(() => this.searchInput.focus(), 100);
        }
      } else if (e.key === 'Escape') {
        // Close help panel if it's open
        if (this.helpPanel && !this.helpPanel.classList.contains('collapsed')) {
          e.preventDefault();
          this.helpPanel.classList.add('collapsed');
          const collapseBtn = this.helpPanel.querySelector('.btn-collapse');
          if (collapseBtn) {
            collapseBtn.setAttribute('aria-expanded', 'false');
          }
        }
      }
    });

    // Help button click
    if (this.btnHelp) {
      this.btnHelp.addEventListener('click', () => {
        this.toggleHelpPanel();
        if (this.searchInput) {
          setTimeout(() => this.searchInput.focus(), 100);
        }
      });
    }
  }

  toggleHelpPanel() {
    if (!this.helpPanel) return;
    this.helpPanel.classList.toggle('collapsed');
    const isCollapsed = this.helpPanel.classList.contains('collapsed');
    const collapseBtn = this.helpPanel.querySelector('.btn-collapse');
    if (collapseBtn) {
      collapseBtn.setAttribute('aria-expanded', !isCollapsed);
    }
  }

  handleSearch(query) {
    const normalizedQuery = query.toLowerCase().trim();
    let visibleCount = 0;

    // First check if query exactly matches any command name
    let exactMatch = false;
    if (normalizedQuery) {
      this.helpCommandList.querySelectorAll('.help-command-item').forEach(item => {
        const cmdName = item.getAttribute('data-command-name').toLowerCase();
        if (cmdName === normalizedQuery) {
          exactMatch = true;
        }
      });
    }

    this.helpCommandList.querySelectorAll('.help-command-item').forEach(item => {
      const cmdName = item.getAttribute('data-command-name').toLowerCase();
      const cmdDesc = item.getAttribute('data-command-description').toLowerCase();
      const fullText = item.textContent.toLowerCase();

      let matches;
      if (normalizedQuery === '') {
        // Empty query - show all
        matches = true;
      } else if (exactMatch) {
        // Exact command name match exists - show only that command
        matches = cmdName === normalizedQuery;
      } else {
        // No exact match - do full-text search
        matches = cmdName.includes(normalizedQuery) ||
                 cmdDesc.includes(normalizedQuery) ||
                 fullText.includes(normalizedQuery);
      }

      if (matches) {
        item.style.display = '';
        visibleCount++;
      } else {
        item.style.display = 'none';
      }
    });

    this.updateSearchResults(visibleCount, query);
  }

  updateSearchResults(count, query = '') {
    if (!this.searchResultCount) return;

    if (query) {
      this.searchResultCount.textContent = `${count} result${count !== 1 ? 's' : ''}`;
    } else {
      this.searchResultCount.textContent = '';
    }
  }

  focusFirstHelpItem() {
    const items = this.getVisibleHelpItems();
    if (items.length > 0) {
      items[0].focus();
    }
  }

  focusLastHelpItem() {
    const items = this.getVisibleHelpItems();
    if (items.length > 0) {
      items[items.length - 1].focus();
    }
  }

  focusNextHelpItem(currentItem) {
    const items = this.getVisibleHelpItems();
    const currentIndex = Array.from(items).indexOf(currentItem);
    if (currentIndex >= 0 && currentIndex < items.length - 1) {
      items[currentIndex + 1].focus();
    }
  }

  focusPreviousHelpItem(currentItem) {
    const items = this.getVisibleHelpItems();
    const currentIndex = Array.from(items).indexOf(currentItem);
    if (currentIndex > 0) {
      items[currentIndex - 1].focus();
    } else if (currentIndex === 0 && this.searchInput) {
      // At first item, go back to search input
      this.searchInput.focus();
    }
  }

  getVisibleHelpItems() {
    return Array.from(this.helpCommandList.querySelectorAll('.help-command-item'))
      .filter(item => item.style.display !== 'none');
  }
}

// Initialize application
async function initApp() {
  tracer.info('INIT', 'Application initialization started');
  const statusElement = document.getElementById('status');
  const versionElement = document.getElementById('version');

  try {
    tracer.debug('INIT', 'Setting status to Loading WASM...');
    statusElement.innerHTML = '<span class="loading"></span> Loading WASM...';

    // Initialize WASM module with cache-busting
    tracer.info('WASM', 'Calling init() with cache-busting');
    const initStart = performance.now();
    const cacheBuster = Date.now();
    await init(`wos_bg.wasm?v=${cacheBuster}`);
    const initDuration = performance.now() - initStart;
    tracer.info('WASM', `init() completed in ${initDuration.toFixed(2)}ms (cache-busted: ${cacheBuster})`);

    // Create ConfigManager AFTER WASM is initialized
    tracer.debug('INIT', 'Creating ConfigManager');
    const configManager = new ConfigManager();

    tracer.debug('INIT', 'Creating PanelManager');
    const panelManager = new PanelManager(configManager);

    tracer.debug('INIT', 'Creating Terminal');
    const terminal = new Terminal(configManager);

    // WOS-FILE-EDIT-01: Set terminal reference in panel manager for file list updates
    panelManager.terminal = terminal;

    // Expose terminal instance to window for testing and monitoring
    tracer.debug('INIT', 'Exposing terminal instance to window');
    window.terminalInstance = terminal;

    // Create WOS instance
    tracer.debug('WASM', 'Creating WosWasm instance');
    const wos = new WosWasm();
    tracer.info('WASM', 'WosWasm instance created');

    // WOS-302: Extend WOS with Time-Travel Debugging methods (MVP mock implementation)
    tracer.debug('WASM', 'Adding Time-Travel Debugging methods to WOS');
    wos._startTime = Date.now();
    wos._nextTraceId = 0;

    // Initialize with mock trace history (MVP: Populate with sample syscalls for testing)
    wos._kernelHistory = [
      {
        trace_id: 0,
        calling_pid: 1,
        syscall: { Write: { fd: 1, data: 'init started' } },
        result: { Ok: 'Written 12 bytes' },
        timestamp_us: 100
      },
      {
        trace_id: 1,
        calling_pid: 1,
        syscall: { Read: { fd: 0, count: 1024 } },
        result: { Ok: 'Read 5 bytes' },
        timestamp_us: 250
      },
      {
        trace_id: 2,
        calling_pid: 2,
        syscall: { Fork: null },
        result: { Ok: 'PID 2' },
        timestamp_us: 500
      }
    ];
    wos._nextTraceId = 3; // Continue from last mock trace ID
    wos._currentPosition = 0; // Track current timeline position

    wos._currentState = {
      processes: {
        '1': { pid: 1, state: 'Running', parent_pid: null, memory: 1024 * 64 },
        '2': { pid: 2, state: 'Ready', parent_pid: 1, memory: 1024 * 32 }
      },
      memory: { total: 4096 * 1024, used: 1024 * 96, free: 4096 * 1024 - 1024 * 96 },
      filesystem: { files: ['/bin/echo', '/bin/ls', '/bin/ps', '/tmp/test.txt'] }
    };

    // Method: getKernelHistory() - Returns JSON array of all syscall traces
    wos.getKernelHistory = function() {
      tracer.debug('DEBUGGER', `getKernelHistory() called, returning ${this._kernelHistory.length} traces`);
      return JSON.stringify(this._kernelHistory);
    };

    // Method: getCurrentState() - Returns JSON of current kernel state
    wos.getCurrentState = function() {
      tracer.debug('DEBUGGER', `getCurrentState() called at position ${this._currentPosition}`);

      // Generate state that varies based on timeline position
      // This simulates how state evolves as syscalls are executed
      const pos = this._currentPosition || 0;

      // Memory usage increases with position
      const baseMemory = 1024 * 96;
      const memoryUsed = baseMemory + (pos * 1024 * 4);
      const totalMemory = 4096 * 1024;

      // Process states evolve
      const proc1State = pos < 1 ? 'Ready' : 'Running';
      const proc2State = pos < 1 ? 'Blocked' : 'Ready';

      // Additional processes appear at later positions
      const processes = {
        '1': { pid: 1, state: proc1State, parent_pid: null, memory: 1024 * (64 + pos * 2) },
        '2': { pid: 2, state: proc2State, parent_pid: 1, memory: 1024 * (32 + pos) }
      };

      if (pos > 0) {
        processes['3'] = { pid: 3, state: 'Ready', parent_pid: 1, memory: 1024 * 16 };
      }

      // Filesystem grows with position
      const files = ['/bin/echo', '/bin/ls', '/bin/ps'];
      if (pos > 0) files.push('/tmp/test.txt');
      if (pos > 1) files.push('/tmp/output.log');

      return JSON.stringify({
        processes: processes,
        memory: { total: totalMemory, used: memoryUsed, free: totalMemory - memoryUsed },
        filesystem: { files: files }
      });
    };

    // Method: jumpToPosition(pos) - Navigate to specific position in history
    wos.jumpToPosition = function(position) {
      tracer.debug('DEBUGGER', `jumpToPosition(${position}) called`);
      // Store the current position so getCurrentState() can return position-specific state
      this._currentPosition = position;
      return true;
    };

    // Helper: addTrace(syscall, result, pid) - Add a new trace entry
    wos._addTrace = function(syscall, result, pid = 1) {
      const timestamp_us = (Date.now() - this._startTime) * 1000; // Convert to microseconds
      const trace = {
        trace_id: this._nextTraceId++,
        calling_pid: pid,
        syscall: syscall,
        result: result,
        timestamp_us: timestamp_us
      };
      this._kernelHistory.push(trace);
      tracer.debug('DEBUGGER', `Added trace #${trace.trace_id}: ${JSON.stringify(syscall)}`);
      return trace;
    };

    tracer.info('WASM', 'Time-Travel Debugging methods added');

    tracer.debug('INIT', 'Connecting terminal to WOS');
    terminal.setWOS(wos);

    tracer.debug('INIT', 'Setting status to Ready');
    statusElement.textContent = 'Ready';
    statusElement.className = '';
    tracer.info('INIT', 'Status set to Ready');

    // Initial update of process table and memory view (WOS-301)
    tracer.debug('INIT', 'Performing initial system monitor update');
    terminal.updateSystemMonitor();
    tracer.debug('INIT', 'Initial system monitor update complete');

    // WOS-302: Initialize Time-Travel Debugger
    tracer.debug('INIT', 'Creating TimeTravelDebugger');
    const timeTravelDebugger = new TimeTravelDebugger(wos);
    tracer.info('INIT', 'TimeTravelDebugger initialized');

    // WOS-305: Initialize Contextual Help Panel
    tracer.debug('INIT', 'Creating HelpPanel');
    const helpPanel = new HelpPanel();
    tracer.info('INIT', 'HelpPanel initialized');

    // Expose to window for tests
    window.wos = wos;
    window.timeTravelDebugger = timeTravelDebugger;
    window.helpPanel = helpPanel;

    // Get and display version
    tracer.debug('WASM', 'Getting WOS version');
    const version = wos_version();
    versionElement.textContent = version;
    tracer.info('INIT', `WOS version: ${version}`);

    // Initialize Monaco editor asynchronously in the background
    tracer.debug('MONACO', 'Starting Monaco editor initialization');
    initMonacoEditor();

    // No startup banner - user requested removal
    tracer.info('INIT', 'Application initialization completed successfully');
  } catch (error) {
    tracer.error('INIT', 'Initialization failed', error);
    console.error('Initialization error:', error);
    statusElement.textContent = 'Error';
    statusElement.className = 'error';

    // Check if terminal exists before trying to use it
    if (typeof terminal !== 'undefined' && terminal) {
      terminal.printLine(`Failed to initialize WASM: ${error}`, 'error');
      terminal.printLine('', 'output');
      terminal.printLine('This may happen if:', 'output');
      terminal.printLine('- WASM files are not present', 'output');
      terminal.printLine('- WASM is not supported in your browser', 'output');
      terminal.printLine('- Files are being served from file:// instead of http://', 'output');
    } else {
      // Terminal doesn't exist, show error in page
      tracer.error('INIT', 'Terminal not created, cannot display error message');
      const terminalOutput = document.getElementById('terminal-output');
      if (terminalOutput) {
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
        terminalOutput.appendChild(errorMsg);
      }
    }
  }
}

// Start application when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}
