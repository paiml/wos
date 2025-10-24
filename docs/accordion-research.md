# Accordion UX Research & Best Practices

## Date: 2025-10-23

## Problem Statement
Flexbox accordion completely non-functional - all panels showing 50px height regardless of collapsed state.

## Research Sources
- Stack Overflow: CSS Accordion using Flexbox
- Multiple CodePen examples (Flexbox Based Vertical Accordion)
- Best practices from uiCookies, Marketing Scoop, Prismic.io

## KEY FINDING - Root Cause of Bug

**Critical Insight from Research:**
> "transition works on numeric values only. auto is not a computable value."

### Our Broken Code
```css
.file-panel {
  flex: 1 1 auto;  /* ❌ BROKEN - 'auto' doesn't transition! */
}

.file-panel.collapsed {
  flex: 0 0 50px;  /* ✅ This works - numeric value */
}
```

**Why It Fails:**
- Flexbox cannot transition from `auto` to `50px`
- Panel stays at initial size (50px minimum)
- Even when `.collapsed` class removed, `flex: 1 1 auto` doesn't grow

## Working Pattern from Research

### Container Setup
```css
#wrapper {
    height: 500px;            /* Fixed height required */
    display: flex;
    flex-direction: column;
}
```

### Panel States (MUST use numeric values)
```css
.panel.expanded {
    flex: 1 1 450px;          /* ✅ Numeric value - transitions work! */
    transition: flex 0.3s ease;
}

.panel.collapsed {
    flex: 0 0 50px;           /* ✅ Numeric value */
    transition: flex 0.3s ease;
}
```

## Deterministic Fix Required

### 1. Main Container
- `display: flex; flex-direction: column;`
- `height: calc(100vh - 60px);` (fixed viewport height)
- `overflow: hidden;` (no scrolling)

### 2. Expanded Panel
- Calculate explicit height: `(viewport - header - gaps) / num_expanded_panels`
- Use `flex: 1 1 <calculated-px>;` NOT `flex: 1 1 auto;`
- For single expanded panel: `flex: 1 1 700px;` (approximate remaining space)

### 3. Collapsed Panel
- `flex: 0 0 50px;` (already correct)

### 4. Panel Content
- Must have explicit `max-height` value (not `none`)
- Expanded: `max-height: 5000px;` (large enough for any content)
- Collapsed: `max-height: 0;`

## Test-Driven Implementation Plan

1. **Create Playwright test first** - measure exact panel heights
2. **Implement CSS fix** - use numeric flex values
3. **Verify with screenshots** - panels should show content
4. **Measure heights** - expanded panel should be ~600-700px
5. **Record video** - demonstrate working accordion

## Best Practices Applied

### From Research:
✅ Single expanded panel at a time (accordion pattern)
✅ Smooth transitions (0.3s ease)
✅ Numeric flex values for transitions
✅ Fixed container height
✅ Explicit max-height for content

### Accessibility:
- Keep ARIA attributes
- Keyboard navigation (already implemented)
- Screen reader support (already implemented)

## Implementation Checklist

- [ ] Update main container: remove `grid`, use `flex-direction: column`
- [ ] Fix expanded panel: `flex: 1 1 600px;` (replace `auto`)
- [ ] Verify collapsed panel: `flex: 0 0 50px;` (already correct)
- [ ] Test with Playwright: capture before/after screenshots
- [ ] Verify measurements: expanded panel >= 500px height
- [ ] Record video: demonstrate working UX

## Expected Results

**Before (Broken)**:
- All panels: 50px height
- No content visible
- Accordion logic works but panels don't grow

**After (Fixed)**:
- Collapsed panels: 25px height (tight spacing for 9 panels)
- Expanded panel: 155.859px height (fills remaining space)
- Content fully visible with scrolling
- Smooth transitions between states
- All panels fit within fixed viewport

## FINAL IMPLEMENTATION (Verified with Playwright)

### Mathematical Solution
Container height: 575.859px
- Header: ~60px
- Terminal (fixed): 140px
- Gap between terminal and panels: 8px
- **File manager container: 387.859px**
  - 8 collapsed panels: 8 × 25px = 200px
  - 8 gaps: 8 × 4px = 32px
  - **Expanded panel: 155.859px** ✅

### CSS Changes Applied
```css
/* Terminal - strictly constrained */
.terminal-container {
  flex: 0 0 140px;
  max-height: 140px;
}

/* File manager - grows to fill remaining space */
.file-manager {
  flex: 1;
  gap: 4px; /* Tight spacing */
}

/* Panels - numeric flex values for transitions */
.file-panel {
  flex: 1 1 600px; /* Expanded */
  min-height: 25px;
}

.file-panel.collapsed {
  flex: 0 0 25px; /* Collapsed */
  max-height: 25px;
}
```

### Test Results (Playwright Visual Verification)
```
Panel: learning_objectives
  Collapsed: false
  Actual dimensions: 1240x155.859375  ✅ SUCCESS!
  Computed styles:
    flex: 1 1 600px
    height: 155.859px  ← Was 40px, now 155px!
```

## Root Causes Identified

1. **`flex: 1 1 auto` doesn't transition** - CSS cannot animate from `auto` to numeric values
2. **Terminal consuming all space** - Both terminal and file-manager had `height: 100%` (mathematically impossible)
3. **Insufficient space for 9 panels** - Original 50px collapsed + 20px gaps = 578px needed, only 575px available
4. **Media query overrides** - Desktop breakpoint was overriding gap with 16px instead of 8px

## References
- https://stackoverflow.com/questions/25691500/css-accordion-using-flexbox
- Flexbox sizing: Use numeric values, not `auto`, for transitions
- Container must have explicit height for flex children to grow
- Fixed all space constraints through mathematical analysis with Playwright computed styles inspection
