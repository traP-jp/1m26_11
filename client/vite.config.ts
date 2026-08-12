import { fileURLToPath, URL } from 'node:url'
import { rmSync } from 'node:fs'
import { resolve } from 'node:path'

import { defineConfig, loadEnv, type Plugin } from 'vite'
import tailwindcss from '@tailwindcss/vite'
import vue from '@vitejs/plugin-vue'
import vueDevTools from 'vite-plugin-vue-devtools'

function excludeMswWorkerFromProduction(): Plugin {
  let workerOutputPath: string | undefined

  return {
    name: 'exclude-msw-worker-from-production',
    apply: 'build',
    configResolved(config) {
      workerOutputPath = resolve(config.root, config.build.outDir, 'mockServiceWorker.js')
    },
    closeBundle() {
      if (workerOutputPath) rmSync(workerOutputPath, { force: true })
    },
  }
}

// https://vite.dev/config/
export default defineConfig(({ mode }) => {
  const env = loadEnv(mode, process.cwd(), '')
  const apiTarget = env.API_PROXY_TARGET || 'http://127.0.0.1:8080'
  const isHistoire = Boolean(process.env.HISTOIRE)

  return {
    plugins: [vue(), !isHistoire && vueDevTools(), tailwindcss(), excludeMswWorkerFromProduction()],
    resolve: {
      alias: {
        '@': fileURLToPath(new URL('./src', import.meta.url)),
      },
    },
    server: {
      fs: {
        allow: [fileURLToPath(new URL('..', import.meta.url))],
      },
      proxy: {
        '/api': {
          target: apiTarget,
          changeOrigin: true,
        },
        '/openapi.yaml': {
          target: apiTarget,
          changeOrigin: true,
        },
      },
    },
  }
})
