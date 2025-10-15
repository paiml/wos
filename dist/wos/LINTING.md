# Frontend Linting Guide

Comprehensive linting for HTML, CSS, and JavaScript files in the WOS browser interface.

## Overview

WOS uses **Deno** for frontend code quality:
- ✅ **JavaScript linting** - Code quality and best practices
- ✅ **CSS linting** - Style consistency and patterns
- ✅ **HTML linting** - Semantic markup and accessibility
- ✅ **Formatting** - Consistent code style

## Prerequisites

Install Deno:

```bash
# macOS/Linux
curl -fsSL https://deno.land/x/install/install.sh | sh

# Or use package manager
brew install deno  # macOS
```

## Quick Start

```bash
# Run all linting checks
deno task validate

# Or use the Makefile
cd ../.. && make lint-frontend
```

## Available Commands

### Linting

```bash
# Lint JavaScript
deno task lint

# Lint and auto-fix
deno task lint:fix

# Custom linter (HTML, CSS, JS)
deno run --allow-read lint.ts
```

### Formatting

```bash
# Check formatting
deno task fmt:check

# Auto-format files
deno task fmt
```

### Type Checking

```bash
# Check types
deno task check
```

### Combined

```bash
# Run all checks
deno task validate
```

## What Gets Checked

### HTML Validation

- ✅ DOCTYPE declaration
- ✅ `lang` attribute on `<html>`
- ✅ Viewport meta tag
- ✅ Semantic HTML5 tags
- ✅ Alt attributes on images
- ✅ Script tags use `type="module"`
- ⚠️ Inline styles (anti-pattern)

### CSS Validation

- ✅ No excessive `!important`
- ✅ Color consistency
- ✅ CSS custom properties usage
- ✅ Font sizing (rem vs px)
- ⚠️ Manual vendor prefixes
- ⚠️ Color palette size

### JavaScript Validation

- ✅ No `var` (use `let`/`const`)
- ✅ Use `===` instead of `==`
- ✅ No `debugger` statements
- ✅ No TODO/FIXME/HACK comments
- ⚠️ console.log statements

## Configuration

### deno.json

Main configuration file:

```json
{
  "lint": {
    "rules": {
      "tags": ["recommended"],
      "include": [
        "camelcase",
        "eqeqeq",
        "no-debugger",
        "no-eval"
      ]
    }
  },
  "fmt": {
    "options": {
      "useTabs": false,
      "lineWidth": 100,
      "indentWidth": 2
    }
  }
}
```

### .stylelintrc.json

CSS linting rules:

```json
{
  "extends": "stylelint-config-standard",
  "rules": {
    "color-hex-length": "long",
    "selector-class-pattern": "^[a-z][a-z0-9]*(-[a-z0-9]+)*$"
  }
}
```

## Lint Rules

### JavaScript Rules

| Rule | Severity | Description |
|------|----------|-------------|
| no-debugger | Error | No debugger statements |
| eqeqeq | Warning | Use === instead of == |
| prefer-const | Warning | Use const for immutable vars |
| no-unused-vars | Error | No unused variables |
| camelcase | Warning | Use camelCase naming |
| ban-untagged-todo | Error | No TODO comments |

### CSS Rules

| Rule | Severity | Description |
|------|----------|-------------|
| color-hex-length | Warning | Use long hex (#ffffff) |
| color-named | Error | No named colors |
| length-zero-no-unit | Warning | 0 doesn't need units |
| selector-class-pattern | Error | kebab-case classes |

### HTML Rules

| Rule | Severity | Description |
|------|----------|-------------|
| doctype | Error | Must have DOCTYPE |
| lang-attribute | Error | html tag needs lang |
| alt-attribute | Error | Images need alt text |
| semantic-tags | Warning | Use semantic HTML5 |

## Integration with CI

### GitHub Actions

```yaml
name: Frontend Lint

on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Deno
        uses: denoland/setup-deno@v1
        with:
          deno-version: v1.x

      - name: Lint Frontend
        working-directory: dist/wos
        run: deno task validate

      - name: Custom Lint
        working-directory: dist/wos
        run: deno run --allow-read lint.ts
```

### Pre-commit Hook

Add to `.git/hooks/pre-commit`:

```bash
#!/bin/bash
cd dist/wos
deno task validate || exit 1
cd ../..
```

## Fixing Common Issues

### Issue: "Missing DOCTYPE"

**Problem:**
```html
<html>
```

**Fix:**
```html
<!DOCTYPE html>
<html lang="en">
```

### Issue: "Use === instead of =="

**Problem:**
```javascript
if (value == 5) {
```

**Fix:**
```javascript
if (value === 5) {
```

### Issue: "No TODO comments"

**Problem:**
```javascript
// TODO: Implement this feature
```

**Fix:**
Create GitHub issue instead, remove comment.

### Issue: "Image missing alt"

**Problem:**
```html
<img src="logo.png">
```

**Fix:**
```html
<img src="logo.png" alt="WOS Logo">
```

### Issue: "Use rem instead of px"

**Problem:**
```css
font-size: 16px;
```

**Fix:**
```css
font-size: 1rem;  /* 16px base */
```

## Custom Linter Output

The custom TypeScript linter provides detailed output:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  WOS Frontend Lint Results
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

✓ index.html - No issues

style.css:
  ⚠️  Found 3 font-size declarations in px - consider using rem

app.js:
  ⚠️  console.log found - remove before production:42

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  Errors: 0
  Warnings: 2
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

⚠️  Linting passed with warnings
```

## Best Practices

### HTML

1. **Use semantic tags**: `<header>`, `<main>`, `<nav>`, `<footer>`
2. **Add accessibility**: alt text, ARIA labels, lang attribute
3. **Avoid inline styles**: Use CSS classes
4. **Use type="module"**: For script tags

### CSS

1. **Use CSS variables**: For colors and spacing
2. **Use rem for font sizes**: Better accessibility
3. **Avoid !important**: Structure CSS better
4. **Use kebab-case**: For class names
5. **Group related rules**: Organize by component

### JavaScript

1. **Use const by default**: Only let when needed
2. **Use === always**: Strict equality
3. **No console.log**: Use proper logging
4. **No TODO comments**: Create issues
5. **Type your code**: Use JSDoc or TypeScript

## Formatting Rules

### Indentation

- **JavaScript**: 2 spaces
- **HTML**: 2 spaces
- **CSS**: 2 spaces
- **No tabs**: Spaces only

### Line Width

- **Maximum**: 100 characters
- **Soft limit**: 80 characters (recommended)

### Quotes

- **JavaScript**: Single quotes preferred
- **HTML**: Double quotes
- **CSS**: Single quotes

### Semicolons

- **JavaScript**: Required
- **CSS**: Required

## Metrics

Track frontend code quality:

```bash
# Count issues
deno run --allow-read lint.ts | grep "Errors:"

# Track over time
echo "$(date),$(deno run --allow-read lint.ts 2>&1 | grep -c 'Error')" >> lint-history.csv
```

## Troubleshooting

### Deno not found

```bash
# Check installation
which deno

# Add to PATH
export PATH="$HOME/.deno/bin:$PATH"
```

### Permission denied

```bash
# Add permissions
deno run --allow-read --allow-write lint.ts
```

### Module not found

```bash
# Check Deno version
deno --version

# Update Deno
deno upgrade
```

## Future Enhancements

Planned improvements:

- [ ] TypeScript migration
- [ ] Accessibility (a11y) testing
- [ ] Performance budgets
- [ ] Bundle size tracking
- [ ] Lighthouse CI integration
- [ ] Visual regression testing

## Resources

- [Deno Lint Rules](https://lint.deno.land/)
- [Deno Formatter](https://deno.land/manual/tools/formatter)
- [Stylelint Rules](https://stylelint.io/user-guide/rules)
- [HTML Validator](https://validator.w3.org/)
- [WCAG Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)

## License

Same as WOS project (MIT).
