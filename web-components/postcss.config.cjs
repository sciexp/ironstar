// web-components/postcss.config.cjs
// PostCSS configuration for modern CSS features
// Pipeline: postcss main.css → processed.css → rolldown bundles with hash

module.exports = {
  plugins: {
    "postcss-import": {},
    "postcss-preset-env": {
      stage: 0,
      features: {
        "oklab-function": true,
        "light-dark-function": true,
        "custom-media-queries": true,
      },
    },
    autoprefixer: {},
    cssnano: {
      preset: "default",
    },
  },
};
