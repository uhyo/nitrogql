import { defineConfig } from "vite";
import react from "@vitejs/plugin-react-swc";
import nitrogql from "@nitrogql/rollup-plugin";

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    react(),
    nitrogql({
      include: ["**/*.graphql"],
    }),
  ],
});
