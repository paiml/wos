# WebOS Rapid Iteration Workflow

Quick guide for daily WebOS development with instant deployment to paiml.com.

## 🔗 One-Time Setup

Run once to create symlink:

```bash
cd /home/noah/src/wos
make link-dev
```

This creates a symlink from `/home/noah/src/interactive.paiml.com/dist/wos` → `/home/noah/src/wos/dist/wos`

## 🚀 Daily Workflow

### 1. Edit Code
Work on WebOS code in `/home/noah/src/wos/`:
- Rust: `wos/`, `kernel/`, `userspace/`
- Frontend: `wos/frontend/`, `wos/index.html`

### 2. Build
```bash
cd /home/noah/src/wos
make build
```

Changes appear **immediately** in paiml.com dist (via symlink)!

### 3. Test Locally
```bash
cd /home/noah/src/interactive.paiml.com
ruchy serve dist --port 8080
```

Visit: http://localhost:8080/wos/

### 4. Deploy to Production
When ready:
```bash
cd /home/noah/src/interactive.paiml.com
make deploy
```

## 📊 Complete Quality Workflow

Before deploying to production:

```bash
# In wos/ directory
make build          # Build WebOS
make test           # Run all tests
make quality        # Quality checks

# In interactive.paiml.com/ directory
make lint           # Lint all code
make test           # Run paiml.com tests
make deploy         # Deploy to production
```

## 🔄 How It Works

**Symlink Benefits:**
- No copying files between repos
- Changes in `wos/dist/wos` appear instantly in `paiml.com/dist/wos`
- Single source of truth (wos repo)
- Fast iteration cycles

**Architecture:**
```
/home/noah/src/wos/dist/wos/          # Source (real files)
    ↓ (symlink)
/home/noah/src/interactive.paiml.com/dist/wos/  # Link (points to source)
    ↓ (deployment)
S3: interactive.paiml.com-production-mces4cme   # Production
    ↓ (CloudFront)
https://interactive.paiml.com/wos/              # Live site
```

## 🛠️ Troubleshooting

### Symlink Broken?
Re-run setup:
```bash
cd /home/noah/src/wos
make link-dev
```

### Changes Not Appearing?
1. Verify symlink: `ls -la /home/noah/src/interactive.paiml.com/dist/wos`
2. Rebuild: `cd /home/noah/src/wos && make build`
3. Clear browser cache

### Want to Revert to Copying?
Remove symlink and copy files manually:
```bash
rm /home/noah/src/interactive.paiml.com/dist/wos
cp -r /home/noah/src/wos/dist/wos /home/noah/src/interactive.paiml.com/dist/
```

## 📝 Notes

- Symlink survives across builds (no need to re-link)
- Works with `make deploy` (follows symlinks)
- Safe for production (symlinks resolved during S3 upload)
- Perfect for daily iteration cycles
