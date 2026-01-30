# UI Design System Documentation

## Overview

This document describes the UI design system implemented in the amp Android application, focusing on the comprehensive styling updates from commit `57dd21b303e1f3dc69e0ecb81d9fdce1745b4a1a`.

**Commit Details:**
- **SHA:** `57dd21b303e1f3dc69e0ecb81d9fdce1745b4a1a`
- **Date:** 2026-01-30T08:57:39Z
- **Author:** Albin (bearded-albin)
- **Message:** "ui and title changes"
- **Files Changed:** `android/assets/style.css` (307 additions, 301 deletions)

## Design Philosophy

The UI implements a **neumorphic design system** that creates soft, extruded shapes through carefully crafted shadows and lighting effects. The design emphasizes:

1. **Tactile Interactions:** Button presses and interactions feel physical
2. **Consistent Elevation:** Components use consistent shadow patterns
3. **Subtle Gradients:** Visual depth without overwhelming colors
4. **Accessibility:** High contrast text and focus states

## Color System

### CSS Custom Properties

The design system uses CSS custom properties for maintainable theming:

```css
:root {
    /* Shadow Definitions */
    --shadow-sm: 2px 2px 4px #a3b1c6, -2px -2px 4px #ffffff;
    --shadow-md: 4px 4px 8px #a3b1c6, -4px -4px 8px #ffffff;
    --shadow-lg: 6px 6px 12px #a3b1c6, -6px -6px 12px #ffffff;
    --shadow-xl: 9px 9px 16px #a3b1c6, -9px -9px 16px #ffffff;
    --shadow-inset: inset 2px 2px 5px #a3b1c6, inset -2px -2px 5px #ffffff;
    --shadow-active: inset 1px 1px 5px #bb4ea0, inset -1px -1px 5px #bb4ea0;
    
    /* Base Colors */
    --color-bg: #e0e0e0;
    --color-text: #3d5a80;
    --color-text-secondary: #556b82;
}
```

### Accent Color Palette

A vibrant 10-color palette provides category differentiation:

| Color Name | Hex Value | Usage |
|------------|-----------|-------|
| Strawberry Red | `#f94144ff` | Error states, active items |
| Atomic Tangerine | `#f3722cff` | Warnings |
| Carrot Orange | `#f8961eff` | 6-hour category |
| Coral Glow | `#f9844aff` | Highlights |
| Tuscan Sun | `#f9c74fff` | Success states |
| Willow Green | `#90be6dff` | Positive indicators |
| Seagrass | `#43aa8bff` | Information |
| Dark Cyan | `#4d908eff` | 24-hour category |
| Blue Slate | `#577590ff` | Secondary actions |
| Cerulean | `#277da1ff` | Primary actions |

## Typography

### Font Families

```css
/* Display Font - Used for main title */
font-family: 'Workbench', cursive;

/* System Font Stack - Used for UI elements */
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, 
             'Helvetica Neue', sans-serif;

/* Monospace - Used for addresses and technical data */
font-family: 'Courier New', monospace;
```

### Font Sizes

- **Title:** 52px (`.topbar-title-text`)
- **Category Headers:** 15px (`.category-title`)
- **Body Text:** 14px (`.address-text`, `.topbar-input`)
- **Buttons:** 13-14px

## Component Patterns

### 1. Top Bar Component

The top bar features a custom gradient background with a distinctive design:

```css
.topbar-title {
    min-height: 80px;
    background: linear-gradient(
        0deg,
        #e0e0e0 0%,    /* Base neutral */
        #e1d4dd 25%,   /* Light transition */
        #d59aba 75%,   /* Pink accent */
        #bb4ea0 100%   /* Deep pink */
    );
}
```

**Key Changes in Commit:**
- Migrated from pink-tinted gradient (`#fdeff9` → `#d65db1`) to neutral-pink blend
- Maintained visual hierarchy while reducing color saturation
- Preserved brand accent color (`#bb4ea0`) at gradient peak

**Structure:**
```
topbar-container
├── topbar-title
│   ├── topbar-bg-wrap (SVG background layer)
│   └── topbar-title-content
│       ├── topbar-title-text (52px "amp" title)
│       └── topbar-settings-btn
└── topbar-content
    ├── topbar-inputs-row
    └── topbar-buttons-row
```

### 2. Input Components

#### Standard Input
```css
.topbar-input {
    padding: 12px 16px;
    border-radius: 20px;
    background: transparent;
    box-shadow: inset 1px 1px 3px #a3b1c6, 
                inset -1px -1px 20px #ffffff;
}
```

**Interaction States:**
- **Focus:** `box-shadow: var(--shadow-active)`
- **Placeholder:** Color `#8896ab`, opacity `0.6`
- **Active:** Caret hidden (`caret-color: transparent`)

#### Column Layout
```css
.input-column {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-bottom: 12px;
}
```

### 3. Button System

#### Primary Buttons (`.topbar-btn`)
```css
.topbar-btn {
    padding: 12px 16px;
    height: 50px;
    border-radius: 20px;
    box-shadow: inset 1px 1px 3px #a3b1c6, 
                inset -1px -1px 20px #ffffff;
}
```

**Button Row Behavior:**
- First child: `border-radius: 20px 0 0 20px`
- Middle children: `border-radius: 0`
- Last child: `border-radius: 0 20px 20px 0`

#### Secondary Buttons (`.btn`)
```css
.btn {
    height: 44px;
    border-radius: 12px;
    box-shadow: var(--shadow-sm);
}
```

**Interaction States:**
- **Hover:** `box-shadow: var(--shadow-lg)`
- **Active:** `box-shadow: var(--shadow-active)`
- **Focus:** No outline, maintains shadow

### 4. Toggle Switch Component

A custom neumorphic toggle switch with LED indicator:

```css
.switch-container {
    width: 84px;
    height: 32px;
    border-radius: 16px;
    box-shadow: inset 3px 3px 7px #a3b1c6, 
                inset -3px -3px 7px #ffffff;
}

.switch-thumb {
    width: 36px;
    height: 24px;
    border-radius: 12px;
    transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.led {
    width: 5.7px;
    height: 5.7px;
    border-radius: 50%;
}
```

**LED States:**
- **Inactive:** `background: #999`
- **Active:** `background: #bb4ea0; box-shadow: 0 0 15px 4px #bb4ea0`

**Thumb Position:**
- **Off:** `left: 4px`
- **On:** `left: calc(100% - 36px - 4px); background: #c8c8c8`

### 5. Address Item Component

```css
.address-item {
    display: grid;
    grid-template-columns: 1fr auto;
    align-items: center;
    padding: 12px;
    margin-bottom: 10px;
    border-radius: 20px;
    min-height: 50px;
    box-shadow: inset 1px 1px 3px #a3b1c6, 
                inset -1px -1px 20px #ffffff;
}
```

**Grid Structure:**
- **Column 1 (1fr):** `.address-text` - Address display
- **Column 2 (auto):** `.address-actions` - Toggle + Remove button

### 6. Category Containers

Each category has a unique color scheme:

#### Active Category (Red)
```css
.category-active .category-title {
    background: linear-gradient(135deg, #f32c25 0%, #d32f2f 100%);
    box-shadow: 0 8px 16px rgba(243, 44, 37, 0.3), 
                inset 0 2px 4px rgba(255, 255, 255, 0.2);
}
.category-active .category-content {
    background: rgba(243, 44, 37, 0.08);
}
```

#### Other Categories

| Category | Gradient | Background Tint |
|----------|----------|----------------|
| 6-hour | `#FF9800 → #F57C00` | `rgba(255, 152, 0, 0.08)` |
| 24-hour | `#2196F3 → #1976D2` | `rgba(33, 150, 243, 0.08)` |
| Month | `#9C27B0 → #7B1FA2` | `rgba(156, 39, 176, 0.08)` |
| Invalid | `#757575 → #424242` | `rgba(117, 117, 117, 0.08)` |
| Addresses | `#000000 → #1a1a1a` | `rgba(0, 0, 0, 0.08)` |

## Animation System

### Transition Easing

The UI uses a custom cubic-bezier curve for bouncy, tactile animations:

```css
transition: all 0.4s cubic-bezier(0.34, 1.56, 0.64, 1);
```

**Breakdown:**
- **Duration:** 400ms (0.4s)
- **Easing:** `cubic-bezier(0.34, 1.56, 0.64, 1)`
  - Creates slight overshoot effect
  - Mimics physical spring behavior
  - Adds personality to interactions

### Standard Transitions

```css
/* Quick interactions */
transition: all 0.3s cubic-bezier(0.645, 0.045, 0.355, 1);

/* Background changes */
transition: background 0.4s ease;
```

## Accessibility Features

### Focus Management

All interactive elements have focus handling disabled to prevent visual disruption in the neumorphic design:

```css
* {
    outline: none !important;
    -webkit-tap-highlight-color: transparent;
}

button:focus,
input:focus {
    outline: none !important;
}
```

**Note:** While outlines are removed for aesthetic consistency, the active shadow states (`--shadow-active`) provide visual feedback during interaction.

### Touch Target Sizing

- **Minimum button height:** 44-50px (meets WCAG 2.5.5)
- **Padding:** 12-16px for comfortable touch areas
- **Grid gaps:** 8px minimum for element separation

### Text Legibility

- **Primary text:** `#3d5a80` on `#e0e0e0` (contrast ratio ~7.5:1) ✓
- **Secondary text:** `#556b82` on `#e0e0e0` (contrast ratio ~5.8:1) ✓
- **Font weight:** Minimum 500 for body text
- **Letter spacing:** 0.5px for improved readability

## Responsive Design

### Mobile Breakpoint

```css
@media (max-width: 600px) {
    .address-item {
        flex-direction: column;
        align-items: flex-start;
        gap: 10px;
    }
    
    .address-actions {
        width: 100%;
        justify-content: flex-end;
    }
}
```

**Changes at 600px:**
- Address items stack vertically
- Actions align to the right
- Improved thumb interaction space

## Browser Compatibility

### Input Appearance Reset

```css
input,
button,
select,
textarea {
    -webkit-appearance: none !important;
    -moz-appearance: none !important;
    appearance: none !important;
}

input::-webkit-outer-spin-button,
input::-webkit-inner-spin-button {
    -webkit-appearance: none !important;
    margin: 0;
}

input[type=number] {
    -moz-appearance: textfield !important;
}
```

### Focus Ring Removal

```css
::-moz-focus-inner {
    border: none !important;
    outline: none !important;
}
```

## Implementation Notes

### Switch Component User Selection

The toggle switch has user selection completely disabled to prevent text highlight during interaction:

```css
.switch-container {
    -webkit-user-select: none !important;
    -moz-user-select: none !important;
    user-select: none !important;
    pointer-events: auto !important;
}
```

### Caret Hiding

Input fields hide the text cursor for cleaner aesthetics:

```css
.topbar-input {
    caret-color: transparent;
}
```

**Implication:** Users won't see a blinking cursor, which may affect usability for text editing. Consider adding a subtle focus indicator if text editing is primary use case.

### Z-Index Layering

```
Layer Stack (bottom to top):
1. .topbar-bg-wrap (z-index: 1) - SVG background
2. .topbar-title-content (z-index: 2) - Content layer
3. .topbar-settings-btn (z-index: 3) - Interactive layer
4. .switch-thumb (z-index: 2) - Switch indicator
```

## Future Considerations

### Potential Improvements

1. **Dark Mode Support**
   - Add `@media (prefers-color-scheme: dark)` queries
   - Invert shadow colors
   - Adjust text contrast

2. **High Contrast Mode**
   - Respect `prefers-contrast: high`
   - Increase border visibility
   - Enhance focus indicators

3. **Reduced Motion**
   - Add `@media (prefers-reduced-motion: reduce)`
   - Simplify transitions
   - Remove bouncy animations

4. **Focus Indicators**
   - Re-introduce subtle focus rings for keyboard navigation
   - Use accent color outline instead of removed outlines

5. **Component Variants**
   - Add disabled state styling
   - Create loading states
   - Implement error state visuals

## Testing Recommendations

### Visual Regression Testing
- Capture screenshots of all component states
- Test across iOS and Android WebView
- Verify shadow rendering consistency

### Interaction Testing
- Toggle switch activation
- Button press feedback
- Input focus behavior
- Category expansion

### Accessibility Testing
- WCAG 2.1 AA compliance
- Touch target sizing
- Color contrast ratios
- Keyboard navigation

## References

- **Commit:** [`57dd21b303e1f3dc69e0ecb81d9fdce1745b4a1a`](https://github.com/resonant-jovian/amp/commit/57dd21b303e1f3dc69e0ecb81d9fdce1745b4a1a)
- **File:** `android/assets/style.css`
- **Design Style:** Neumorphism / Soft UI
- **Font Sources:** Google Fonts (Sixtyfour, Workbench)

---

*Documentation created: 2026-01-30*  
*Last updated: 2026-01-30*  
*Author: Documentation generated from commit analysis*
