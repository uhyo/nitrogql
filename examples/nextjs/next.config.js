/** @type {import('next').NextConfig} */
const nextConfig = {
  typescript: {
    // Types of GraphQL files are provided by nitrogql's content mapper, which
    // requires TypeScript 7.1. Next.js type-checks with the `typescript` package
    // (TypeScript 6), so type checking is done by `npm run typecheck` instead.
    ignoreBuildErrors: true,
  },
  webpack(config) {
    config.module.rules.push({
      test: /\.graphql$/,
      loader: "@nitrogql/graphql-loader",
      options: {
        configFile: "./graphql.config.yaml",
      },
    });

    return config;
  },
};

module.exports = nextConfig;
