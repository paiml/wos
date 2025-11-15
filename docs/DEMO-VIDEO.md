# WebOS Showcase Demo Video

## Overview

**Location**: `docs/webos-showcase-demo.webm`
**Duration**: ~18 seconds
**Size**: 362KB
**Resolution**: 1920x1080

## Demo Content

This comprehensive demo showcases WebOS's key features:

### 1. Environment Variables (3s)
```bash
export NAME="WebOS Demo"
export VERSION="1.0"
```

### 2. Vim Mode Editing (8s)
- Create new file: `vim demo.sh`
- Enter insert mode: `i`
- Write bash script with environment variables
- Save and exit: `:wq`

Script content:
```bash
#!/bin/bash
echo "=== $NAME ==="
echo "Version: $VERSION"
echo "User: $USER"
echo "Working Dir: $PWD"
ls -la
```

### 3. File Execution (3s)
```bash
chmod +x demo.sh
./demo.sh
```
Shows script output with interpolated environment variables.

### 4. File Operations (2s)
```bash
cat demo.sh
```
Display script contents.

### 5. Vim Editing Again (3s)
- Reopen file: `vim demo.sh`
- Add comment: `# WebOS Demo Script`
- Save and exit

### 6. Process Management (2s)
```bash
ps
```
Show running processes.

### 7. Environment Check (2s)
```bash
env | grep -E "NAME|VERSION"
echo "🎉 Demo Complete!"
```

## Technical Details

**Test File**: `tests/e2e/wos-showcase-demo.spec.js`
**Framework**: Playwright Test
**Browser**: Chromium
**Recording**: Automatic via Playwright config

## Running the Demo

### Prerequisites
```bash
# Install dependencies
npm install

# Build WebOS
make build

# Start dev server
ruchy serve dist --port 8000
```

### Execute Demo
```bash
# Run test and generate video
npx playwright test tests/e2e/wos-showcase-demo.spec.js --project=demo

# Video output
test-results/wos-showcase-demo-*/video.webm
```

### Customize Demo

Edit `tests/e2e/wos-showcase-demo.spec.js` to:
- Add more vim commands
- Include different bash scripts
- Show additional shell features
- Adjust timing with `page.waitForTimeout()`

## Configuration

**Playwright Config**: `playwright.config.js`
```javascript
projects: [
    {
        name: 'demo',
        testMatch: '**/wos-showcase-demo.spec.js',
        use: {
            video: 'on', // Always record video
        },
    },
]
```

## Use Cases

- **Marketing**: Show WebOS capabilities
- **Documentation**: Visual feature guide
- **Tutorials**: Interactive learning
- **Presentations**: Conference demos
- **Social Media**: Share on Twitter, LinkedIn
- **README**: Embed in GitHub README

## Embedding in README

```markdown
## Demo Video

See WebOS in action:

[![WebOS Demo](https://img.shields.io/badge/Demo-Watch%20Video-blue)](./webos-showcase-demo.webm)

![WebOS Demo](webos-showcase-demo.webm)
```

## Notes

- Demo runs headless by default (fast)
- Use `--headed` to watch live
- Video records even on test pass (demo project)
- Viewport: 1920x1080 (HD)
- Format: WebM (web-optimized)
- Codec: VP9 (efficient compression)
