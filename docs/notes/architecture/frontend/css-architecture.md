# CSS architecture

This document explains ironstar's CSS architecture, covering Open Props design tokens, Open Props UI component library, CSS cascade layers, and browser compatibility requirements.
Open Props provides design tokens as CSS custom properties, while Open Props UI provides pure CSS component styles following a copy-paste ownership model.
CSS cascade layers provide explicit control over style precedence independent of selector specificity.

## Open Props + Open Props UI architecture

Open Props provides design tokens as CSS custom properties, eliminating the need for JIT compilation or class scanning.
Open Props UI provides pure CSS component styles that you copy into your project for full ownership and customization.

### Design tokens

Open Props provides comprehensive design tokens across multiple categories:

- **Colors**: Complete color scales in OKLch color space (better perceptual uniformity)
- **Sizes**: Responsive spacing scale from `--size-000` to `--size-15`
- **Typography**: Font sizes, line heights, weights, and tracking
- **Shadows**: Elevation scales from `--shadow-1` to `--shadow-6`
- **Borders**: Border radius values from `--radius-1` to `--radius-round`
- **Easings**: Animation curves from `--ease-1` to `--ease-spring-5`
- **Gradients**: Pre-defined gradient patterns
- **Media queries**: Responsive breakpoints via custom media

### Component ownership model

Unlike utility-first frameworks, Open Props UI follows a copy-paste ownership model:

1. **Browse components** at Open Props UI repository (`~/projects/lakescope-workspace/open-props-ui`)
2. **Copy CSS** for needed components into `web-components/styles/components/`
3. **Customize freely** - you own the CSS, modify as needed
4. **No npm dependency** - component CSS is not installed via npm

Example component structure:

```css
/* web-components/styles/components/button.css */
/* Copied from Open Props UI and customized for ironstar */

button,
.button {
  /* Base styles using Open Props tokens */
  padding: var(--size-2) var(--size-4);
  border-radius: var(--radius-default);
  font-size: var(--font-size-1);
  font-weight: var(--font-weight-6);

  /* Colors using theme tokens */
  background-color: var(--primary);
  color: light-dark(var(--gray-0), var(--gray-9));

  /* Interaction states */
  transition: background-color 0.2s var(--ease-in-out);

  &:hover {
    background-color: var(--primary-dark);
  }

  &:active {
    transform: scale(0.98);
  }

  /* Variant: secondary */
  &[data-variant="secondary"] {
    background-color: var(--surface-elevated);
    color: var(--text-primary);
    border: 1px solid var(--border-default);
  }
}
```

### Theme layer

```css
/* web-components/styles/theme.css */

:root {
  /* Primary color derived from Open Props */
  --primary: var(--blue-7);
  --primary-light: var(--blue-5);
  --primary-dark: var(--blue-9);

  /* Accent colors */
  --accent: var(--orange-6);
  --accent-light: var(--orange-4);
  --accent-dark: var(--orange-8);

  /* Surface colors using light-dark() function */
  --surface-default: light-dark(var(--gray-0), var(--gray-9));
  --surface-elevated: light-dark(var(--gray-1), var(--gray-8));
  --surface-overlay: light-dark(var(--gray-2), var(--gray-7));

  /* Text colors */
  --text-primary: light-dark(var(--gray-9), var(--gray-1));
  --text-secondary: light-dark(var(--gray-7), var(--gray-3));
  --text-tertiary: light-dark(var(--gray-6), var(--gray-4));

  /* Border colors */
  --border-default: light-dark(var(--gray-3), var(--gray-7));
  --border-emphasis: light-dark(var(--gray-4), var(--gray-6));

  /* Shadows using Open Props */
  --shadow-card: var(--shadow-2);
  --shadow-elevated: var(--shadow-4);
  --shadow-dialog: var(--shadow-6);

  /* Border radius */
  --radius-default: var(--radius-2);
  --radius-large: var(--radius-3);
  --radius-full: var(--radius-round);

  /* Animation easings */
  --ease-in-out: var(--ease-3);
  --ease-spring: var(--ease-spring-3);
}

/* Dark mode handled automatically by light-dark() function - no JavaScript needed */
/* Browser respects prefers-color-scheme and/or color-scheme meta tag */
```

### PostCSS configuration

PostCSS configuration is simpler than Tailwind since there is no JIT compilation or class scanning needed.
However, Open Props and Open Props UI use modern CSS features that require `postcss-preset-env` for proper processing.

```javascript
// web-components/postcss.config.js
export default {
  plugins: {
    'postcss-import': {},           // Handle @import statements
    'postcss-preset-env': {         // Modern CSS features (required for Open Props)
      stage: 0,                     // Stage 0 required for Open Props (combineSelectors plugin)
      features: {
        'oklab-function': true,     // OKLab/OKLch color spaces
        'light-dark-function': true, // light-dark() for theme switching
        'custom-media-queries': true // Open Props media queries
      }
    },
    'autoprefixer': {},             // Vendor prefixes (optional - Open Props doesn't require it)
    'cssnano': {                    // Minification (production only)
      preset: 'default'
    }
  }
};
```

The `postcss-preset-env` plugin is critical for Open Props integration because it:
- Processes `oklch()` and `oklab()` color functions used throughout Open Props
- Handles `light-dark()` function for automatic dark mode support in Open Props UI
- Supports custom media queries defined by Open Props for responsive design

---

## CSS cascade layers organization

CSS cascade layers (`@layer`) provide explicit control over style precedence independent of selector specificity or source order.
This is particularly important when integrating Open Props UI components with custom application styles.

### What are CSS cascade layers

The `@layer` at-rule establishes a layered cascade where later layers override earlier layers, regardless of specificity.
This eliminates specificity wars and makes style precedence predictable and declarative.

```css
/* Layer declaration - establishes order */
@layer base, components, utilities;

/* Styles in 'utilities' will override 'components',
   which will override 'base', regardless of selector specificity */

@layer base {
  h1 { font-size: 2rem; }
}

@layer utilities {
  .text-small { font-size: 1rem !important; }  /* Overrides h1.text-small */
}
```

Without layers, the `.text-small` class would need higher specificity or `!important` to override the `h1` selector.
With layers, the layer order determines precedence.

### Open Props UI layer structure

Open Props UI uses a specific layer structure for organizing styles with sublayers for component organization.
Reference implementation at `~/projects/lakescope-workspace/open-props-ui` uses:

```css
@layer openprops, normalize, theme, components.root, components.extended, utils;
```

**Layer precedence** (later layers override earlier ones):
- `openprops`: Base design tokens and custom properties
- `normalize`: CSS reset and normalization
- `theme`: Theme-specific token overrides
- `components.root`: Core component styles (base variants)
- `components.extended`: Extended component variants (enhanced versions with additional features)
- `utils`: Utility classes (highest precedence)

The `components.root` and `components.extended` are **sublayers** within a logical components layer.
This separation allows Open Props UI to provide base component styles in `components.root` while keeping enhanced variants (like buttons with icons, cards with actions) in `components.extended` for clearer organization and easier overrides.

This means `utils` overrides `components.extended`, which overrides `components.root`, and so on.

### Why this matters for Ironstar

Understanding the layer structure prevents common pitfalls when customizing components:

**Without layer awareness**:
```css
/* This might not work as expected */
.button {
  background: var(--custom-color);  /* May be overridden by component layer */
}
```

**With layer awareness**:
```css
/* Place in higher-precedence layer to ensure override */
@layer app {
  .button {
    background: var(--custom-color);  /* Guaranteed to override */
  }
}
```

Layers eliminate the need for specificity hacks like:
- Nested selectors (`.page .button`)
- ID selectors (`#app .button`)
- `!important` flags
- Inline styles

### Recommended Ironstar layer structure

```css
/* web-components/styles/main.css */

/* Declare layers upfront - establishes precedence */
@layer openprops, normalize, theme, compositions, components, utilities, app;

/* Import Open Props design tokens into dedicated layer */
@import "open-props/style" layer(openprops);

/* Normalization layer (if needed) */
@layer normalize {
  *, *::before, *::after {
    box-sizing: border-box;
  }
  body {
    margin: 0;
    line-height: 1.5;
  }
}

/* Theme layer - application-specific token overrides */
@layer theme {
  :root {
    --primary: var(--blue-7);
    --surface-default: light-dark(var(--gray-0), var(--gray-9));
    /* ... theme tokens */
  }
}

/* Compositions layer - CUBE CSS layout primitives */
@layer compositions {
  @import "./compositions/stack.css";
  @import "./compositions/cluster.css";
  @import "./compositions/center.css";
  @import "./compositions/sidebar.css";
  /* ... other composition primitives */
}

/* Components layer - Open Props UI component styles */
@layer components {
  @import "./components/button.css";
  @import "./components/card.css";
  @import "./components/dialog.css";
  /* Each component file can define sublayers if needed */
}

/* Utilities layer - single-purpose utility classes */
@layer utilities {
  .visually-hidden {
    position: absolute;
    width: 1px;
    height: 1px;
    overflow: hidden;
  }

  .truncate {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
}

/* App layer - application-specific overrides (highest precedence) */
@layer app {
  /* Page-specific customizations */
  .hero-section .button {
    padding: var(--size-3) var(--size-6);
  }
}
```

### Component-specific layer usage

Individual component files can use layers internally:

```css
/* web-components/styles/components/button.css */
@layer components {
  button,
  .button {
    /* Base button styles using Open Props tokens */
    padding: var(--size-2) var(--size-4);
    border-radius: var(--radius-default);
    background-color: var(--primary);
    color: light-dark(var(--gray-0), var(--gray-9));

    /* Variants can be in same layer - specificity still applies within layer */
    &[data-variant="secondary"] {
      background-color: var(--surface-elevated);
      color: var(--text-primary);
      border: 1px solid var(--border-default);
    }

    &[data-size="large"] {
      padding: var(--size-3) var(--size-6);
      font-size: var(--font-size-2);
    }
  }
}
```

### Composition layer details

The composition layer implements CUBE CSS layout primitives that provide semantic, composable layouts without viewport breakpoints.
These primitives use intrinsic design principles where layouts respond to available space rather than specific viewport widths.

**Purpose**: Compositions handle spatial relationships between elements (vertical spacing, horizontal clustering, centering, sidebars) using flexbox and grid with custom property configuration.

**Layer position**: The `compositions` layer sits between `theme` (which defines design tokens) and `components` (which style specific UI patterns).
Compositions consume theme tokens like `--size-3` but don't define visual styling like colors or shadows.
Components may contain composition classes, but compositions remain layout-agnostic.

**The eight primitives**:

1. **Stack**: Vertical spacing between siblings via owl selector (`> * + *`)
2. **Box**: Padded container with optional border
3. **Center**: Horizontal centering with maximum width
4. **Cluster**: Wrapping horizontal group with gap spacing
5. **Sidebar**: Fixed-width sidebar with flexible main content
6. **Switcher**: Conditional horizontal/vertical layout based on container width
7. **Cover**: Full-height container with centered principal element
8. **Grid**: Auto-filling responsive grid

For complete CSS patterns and custom property reference, see `~/.claude/commands/preferences/hypermedia-development/04-css-architecture.md`.
The "Composition primitives" section below provides detailed documentation of each primitive with examples.

**Integration with hypertext templates**:

Composition classes appear directly in server-rendered HTML, following CUBE CSS class grouping conventions:

```rust
use hypertext::{html_elements, GlobalAttributes, Renderable};

html! {
  <div class="[ stack ] [ card ]" style="--stack-space: var(--size-5)">
    <h2>"Card Title"</h2>
    <p>"Content with vertical rhythm"</p>
    <div class="[ cluster ]" style="--cluster-justify: flex-end">
      <button class="button">"Cancel"</button>
      <button class="button">"Confirm"</button>
    </div>
  </div>
}
```

Square brackets group classes for readability: `[ compositions ] [ components ] [ utilities ]`.

**Light DOM compatibility**:

Lit web components using Open Props tokens render to Light DOM (not Shadow DOM) to inherit CSS custom properties.
Composition primitives work identically in server-rendered HTML and Lit component templates because there's no Shadow DOM boundary blocking token inheritance.

**Algebraic properties**:

Composition primitives exhibit properties analogous to algebraic structures:

- **Closure**: Composing primitives yields valid layouts (Stack > Grid > Cluster nests correctly)
- **Local reasoning**: A Stack behaves identically regardless of parent context (like pure functions)
- **Identity**: An empty composition (div with no layout class) acts as identity—no layout transformation
- **Associativity**: Nesting order doesn't affect semantic meaning (though visual results may differ)

These properties emerge from CSS flexbox/grid intrinsic sizing rather than being explicitly enforced, but they're reliable enough to treat compositions as a layout algebra.
See `docs/notes/architecture/core/design-principles.md` for theoretical foundations.

**File structure**:

```
web-components/styles/compositions/
├── index.css         # Imports all primitives
├── stack.css         # Vertical spacing
├── box.css           # Padded container
├── center.css        # Horizontal centering
├── cluster.css       # Wrapping horizontal groups
├── sidebar.css       # Fixed sidebar + flexible content
├── switcher.css      # Conditional layout switching
├── cover.css         # Full-height centered content
└── grid.css          # Auto-filling grid
```

Each file contains a single composition class with configurable custom properties and no visual styling (colors, shadows, borders beyond structural needs).

### Unlayered styles

Styles not placed in any `@layer` have the highest precedence of all, even higher than the last declared layer.
This is intentional - it allows emergency overrides and inline customizations.

```css
/* These styles override ALL layers, including @layer app */
.emergency-override {
  display: none !important;
}
```

For maintainability, avoid unlayered styles in production code.
All styles should be in explicit layers.

### Layers are optional but recommended

CSS layers are **optional** for ironstar - the stack will work without them.
However, they provide significant benefits:

**Benefits of using layers**:
- Predictable cascade behavior independent of specificity
- Easier component customization without specificity hacks
- Clear separation between base styles, components, and overrides
- Scalable architecture as project grows

**When to skip layers**:
- Very simple projects with minimal CSS
- Need to support older browsers (pre-2022)
- Team unfamiliar with cascade layers

For ironstar, layers are recommended as part of the modern CSS architecture, aligning with the project's preference for modern features over wide compatibility.

---

## Browser compatibility

Open Props and Open Props UI rely on modern CSS features.
Browser support requirements are more stringent than traditional CSS frameworks.

### Required browser versions

Open Props and Open Props UI use multiple modern CSS features with different browser support requirements.
The minimum browser versions are determined by the most recent feature (light-dark() function).

**OKLch color space** (perceptual uniformity):
- **Chrome/Edge**: 111+ (March 2023)
- **Firefox**: 113+ (May 2023)
- **Safari**: 15.4+ (March 2022)

**light-dark() function** (automatic dark mode):
- **Chrome/Edge**: 123+ (March 2024)
- **Firefox**: 120+ (November 2023)
- **Safari**: 17.5+ (May 2024)

**Minimum browser requirements for ironstar** (light-dark() support):
- **Chrome/Edge**: 123+
- **Firefox**: 120+
- **Safari**: 17.5+

### Modern CSS features used

| Feature | Purpose | Fallback strategy |
|---------|---------|-------------------|
| CSS custom properties | Design tokens | Required - no fallback |
| `light-dark()` function | Automatic dark mode | Use `@media (prefers-color-scheme)` |
| OKLch colors | Perceptual uniformity | Use fallback RGB/HSL values |
| `color-mix()` function | Dynamic color blending | Pre-calculate mixed colors |
| Container queries | Responsive components | Use media queries |
| Cascade layers (`@layer`) | Style organization | Not critical - remove if needed |
| CSS nesting | Component styling | PostCSS plugin transforms to flat CSS |

### Graceful degradation patterns

For projects requiring wider browser support, implement fallbacks:

```css
/* Fallback for light-dark() function */
:root {
  --surface-default: var(--gray-0);
}

@media (prefers-color-scheme: dark) {
  :root {
    --surface-default: var(--gray-9);
  }
}

/* Fallback for OKLch colors */
.button {
  background: rgb(59, 130, 246);  /* RGB fallback */
  background: oklch(0.6 0.15 250deg);  /* Modern syntax */
}

/* Fallback for color-mix() */
.hover-state {
  /* Pre-calculated mixed color */
  background: rgb(49, 108, 207);
  /* Modern dynamic mixing */
  background: color-mix(in oklch, var(--primary) 80%, black);
}
```

For ironstar, we target modern browsers exclusively and do not implement fallbacks.

---

## Light DOM requirement for Lit components

All Lit components using Open Props tokens must render to Light DOM instead of Shadow DOM.
This is a fundamental architectural constraint when combining Lit with Open Props.

### Why Light DOM is required

Shadow DOM creates a CSS encapsulation boundary that blocks CSS custom property inheritance.
Since Open Props design tokens are defined in the global `:root` scope, Shadow DOM prevents these tokens from reaching the component's internal styles.

```typescript
// In your Lit component
protected createRenderRoot() {
  return this  // Light DOM, not Shadow DOM
}
```

### When Shadow DOM is incompatible

Components that require Shadow DOM encapsulation cannot use Open Props tokens and must define their own isolated styles.
This creates a design constraint: you must choose between Open Props token inheritance (Light DOM) or CSS encapsulation (Shadow DOM).

**Light DOM (recommended for ironstar)**:
- ✅ Open Props tokens available
- ✅ Global theme automatically applied
- ✅ Consistent styling with rest of application
- ❌ No style encapsulation
- ❌ Global styles can leak into component

**Shadow DOM**:
- ✅ Complete style encapsulation
- ✅ No style leakage concerns
- ❌ Open Props tokens unavailable
- ❌ Must define all styles internally
- ❌ Theme switching requires JavaScript

For ironstar's architecture, Light DOM is the correct choice because:
1. Open Props tokens provide the design system foundation
2. Server-rendered components benefit from global theme consistency
3. Datastar's hypermedia approach reduces need for style encapsulation
4. Web components are thin wrappers, not complex isolated widgets

---

## CSS entry point structure

The main CSS entry point imports Open Props tokens, theme customizations, and component styles in proper cascade order:

```css
/* web-components/styles/main.css */

/* Import Open Props design tokens */
@import "open-props/style";

/* Or selective imports for smaller bundle size */
/* @import "open-props/colors"; */
/* @import "open-props/sizes"; */
/* @import "open-props/animations"; */
/* @import "open-props/easings"; */
/* @import "open-props/shadows"; */
/* @import "open-props/borders"; */

/* Theme layer - application-specific tokens */
@import "./theme.css";

/* Composition layer - CUBE CSS layout primitives */
@import "./compositions/stack.css";
@import "./compositions/cluster.css";
@import "./compositions/center.css";
@import "./compositions/sidebar.css";
@import "./compositions/switcher.css";
@import "./compositions/box.css";
@import "./compositions/cover.css";
@import "./compositions/grid.css";

/* Component styles (copied from Open Props UI, owned by project) */
@import "./components/button.css";
@import "./components/card.css";
@import "./components/dialog.css";
@import "./components/input.css";
/* Add other components as needed */

/* View transitions (optional) */
@view-transition {
  navigation: auto;
}
```

---

## Related documentation

- Frontend build pipeline: `frontend-build-pipeline.md` (Rolldown, PostCSS, asset serving)
- Integration patterns: `integration-patterns.md` (Lit components, web components, third-party libraries)
- Architecture decisions: `../core/architecture-decisions.md` (Open Props rationale, design philosophy)
- Local repositories:
  - Open Props: `~/projects/lakescope-workspace/open-props`
  - Open Props UI: `~/projects/lakescope-workspace/open-props-ui`
