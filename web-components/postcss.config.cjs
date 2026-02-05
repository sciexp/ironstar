// web-components/postcss.config.cjs
// PostCSS configuration for modern CSS features
// Pipeline: postcss main.css → processed.css → rolldown bundles with hash
//
// OKLch color system configuration:
// - oklab-function: oklch() and oklab() color functions
// - color-mix: color-mix(in oklch, ...) for perceptually uniform blending
// - relative-color-syntax: oklch(from var(--color) l c h) for color derivation
// - light-dark-function: automatic dark mode with light-dark()
//
// See docs/notes/architecture/frontend/css-architecture.md for details.

module.exports = {
  plugins: {
    "postcss-import": {},
    "postcss-preset-env": {
      stage: 2,
      features: {
        // OKLch perceptually uniform color system
        "oklab-function": true,
        "color-mix": true,
        "relative-color-syntax": true,
        // Theme and layout features
        "light-dark-function": true,
        "custom-media-queries": true,
        // CSS nesting for component styling
        "nesting-rules": true,
      },
    },
    autoprefixer: {},
    cssnano: {
      preset: "default",
    },
  },
};
