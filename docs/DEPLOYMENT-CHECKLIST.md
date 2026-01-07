# WebOS Deployment Checklist

## Pre-Deployment: Quality Gates

### WOS Project Tests
- [ ] **Rust Unit Tests**: `cd /home/noah/src/wos && make test`
  - Expected: All tests passing
  - Current: 121 unit tests + 3 doc tests passing ✅

- [ ] **E2E Tests**: `cd /home/noah/src/wos && make e2e`
  - Expected: 12 tests passing (toolbar, resize, help, badge)
  - Current: All passing ✅

- [ ] **Quality Gates**: `cd /home/noah/src/wos && make quality`
  - Linting ✅
  - Complexity checks ✅
  - Coverage 85%+ ✅

- [ ] **WASM Build**: `cd /home/noah/src/wos && make build`
  - Expected: `dist/wos/wos_bg.wasm` created
  - Size: <500KB target

### Interactive.paiml.com Tests
- [ ] **Build**: `cd /home/noah/src/interactive.paiml.com && make build`
  - Expected: All books generated successfully
  - Verify: `dist/index.html` contains WebOS entry

- [ ] **Linting**: `make lint`
  - Expected: 0 errors (warnings acceptable)

- [ ] **E2E Tests**: `make test-e2e`
  - Expected: 9/9 tests passing
  - Note: 10 pre-existing unit test failures (unrelated to WebOS)

## Deployment Steps

### 1. One-Time Setup (if not done)
```bash
cd /home/noah/src/wos
make link-dev
```
Verify symlink: `ls -la /home/noah/src/interactive.paiml.com/dist/wos`

### 2. Build WebOS
```bash
cd /home/noah/src/wos
make build
```

### 3. Verify Local
```bash
cd /home/noah/src/interactive.paiml.com
ruchy serve dist --port 8080

# Open: http://localhost:8080/
# Check: WebOS card appears in catalog
# Click: "Launch WebOS →" opens /wos/index.html
```

### 4. Deploy to Production
```bash
cd /home/noah/src/interactive.paiml.com
make deploy
```

Expected output:
- ✅ Linting passed
- ✅ Build completed
- ✅ E2E tests passed (9/9)
- ✅ Content validation passed
- ✅ S3 sync completed
- ✅ CloudFront invalidation created

### 5. Post-Deployment Verification

#### Homepage Check
```bash
curl -I https://interactive.paiml.com/index.html
```
Expected: HTTP 200

#### WebOS Check
```bash
curl -I https://interactive.paiml.com/wos/index.html
```
Expected: HTTP 200

#### CloudFront Cache Status
```bash
aws cloudfront list-invalidations \
  --distribution-id ELY820FVFXAFF \
  --max-items 1
```
Expected: Status "Completed"

#### Visual Verification
- [ ] Visit https://interactive.paiml.com/
- [ ] Verify WebOS card visible with 🖥️ icon
- [ ] Click "Launch WebOS →"
- [ ] Verify WebOS loads
- [ ] Test vim mode: `vim test.sh`
- [ ] Test bash script execution
- [ ] Test environment variables

## Rollback Procedure

If issues occur in production:

```bash
# Option 1: Revert git commit
cd /home/noah/src/interactive.paiml.com
git revert HEAD
make deploy

# Option 2: Checkout specific files
git checkout HEAD~1 -- dist/index.html
make deploy

# Option 3: Emergency S3 rollback
# (Copy from backup or previous deployment)
aws s3 sync s3://backup-bucket/ s3://interactive.paiml.com-production-mces4cme/
aws cloudfront create-invalidation --distribution-id ELY820FVFXAFF --paths "/*"
```

## Known Issues

### Pre-Existing Test Failures
- **10 unit test failures** in `interactive.paiml.com`
- **Cause**: Missing `books/ruchy/metadata.json`
- **Impact**: None (unrelated to WebOS deployment)
- **Status**: Pre-existing, not introduced by WebOS changes

### Browser Compatibility
- **Supported**: Chrome, Firefox, Safari, Edge (modern versions)
- **Required**: WebAssembly support
- **Required**: ES6 module support

## Deployment Frequency

**Rapid Iteration Workflow** allows daily deployments:

1. **Morning**: Develop features in `/home/noah/src/wos`
2. **Afternoon**: Build and test locally
3. **Evening**: Deploy to production after quality gates

Average deployment time: **5-8 minutes** (including E2E tests)

## Monitoring

### Post-Deployment (First 24 hours)
- [ ] Check CloudFront access logs for 404 errors
- [ ] Monitor browser console for JavaScript errors
- [ ] Review user feedback
- [ ] Check WASM load times (should be <2s)

### Tools
- CloudFront Logs: AWS Console → CloudFront → Reports
- Browser Console: F12 → Console tab
- Network Tab: F12 → Network tab (filter: WS, WASM)

## Emergency Contacts

- **S3 Bucket**: `interactive.paiml.com-production-mces4cme`
- **CloudFront**: Distribution ID `ELY820FVFXAFF`
- **Region**: us-east-1
- **GitHub**: https://github.com/paiml/wos/issues

## Deployment History

| Date | Version | Deployer | Notes |
|------|---------|----------|-------|
| 2025-10-23 | v0.1.0-alpha | Claude Code | Initial catalog deployment |
| | | | Added WebOS to interactive.paiml.com |
| | | | Symlink workflow established |
| | | | Demo video created |

---

**Last Updated**: 2025-10-23
**Next Review**: After next major feature release
