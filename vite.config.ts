import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import dynamicImport from 'vite-plugin-dynamic-import';
import viteCompression from 'vite-plugin-compression';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [
    vue(),
    dynamicImport(),
    viteCompression()
  ],
  root: "./src",
  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. tauri expects a fixed port, fail if that port is not available
  build: {
    outDir: "../dist",
    target: "ESNext"
  },
  server: {
    port: 1420,
    strictPort: true,
    watch: {
      // 3. tell vite to ignore watching `src-tauri`
      ignored: ["**/src-tauri/**"],
    },
  },
}));
