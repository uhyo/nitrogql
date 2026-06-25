// Global stylesheet side-effect imports (e.g. `import "./globals.css"`).
// Under `moduleResolution: "bundler"` TypeScript 6 requires such imports to
// resolve to a module; Next.js only ships ambient types for CSS *modules*
// (`*.module.css`), so declare the global case here.
declare module "*.css";
