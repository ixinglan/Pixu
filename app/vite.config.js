import { defineConfig } from 'vite';

// Pixu 前端构建：vanilla 原型平移，无框架；端口固定 1420 与 tauri.conf.json 对齐
export default defineConfig({
  clearScreen: false,
  server: {
    port: 1420,
    strictPort: true,
  },
  build: {
    target: 'es2022',
    outDir: 'dist',
  },
});
