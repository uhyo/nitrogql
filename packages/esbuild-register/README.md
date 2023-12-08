# `@nitrogql/esbuild-register`

This package contains an implementation similar to [egoist/esbuild-register](https://github.com/egoist/esbuild-register).

## Usage

Usage depends on your Node.js version.

### Node.js >= 20.6.0 or >= 18.19.0

```sh
node --import=@nitrogql/esbuild-register
```

### Node.js < 20.6.0 and < 18.19.0

```sh
node --import=@nitrogql/esbuild-register --experimental-loader=@nitrogql/esbuild-register/hook
```

## Notes

This package does not intend to be a drop-in replacement for `esbuild-register`. It intentionally does not support some features, for example `import.meta.url` support in CJS modules.