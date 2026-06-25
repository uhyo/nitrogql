// Global stylesheet side-effect imports (e.g. `import "./Toc.css"`).
// TypeScript 6 (with Next.js's bundler-style module resolution) requires such
// imports to resolve to a module; Next.js only ships ambient types for CSS
// *modules* (`*.module.css`), so declare the global case here.
declare module "*.css";
