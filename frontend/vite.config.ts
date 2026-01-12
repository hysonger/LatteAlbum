import { defineConfig, loadEnv } from 'vite'
import vue from '@vitejs/plugin-vue'
import { resolve } from 'path'

// https://vitejs.dev/config/
export default defineConfig(({ mode }) => {
  // Load env file based on `mode` in the current working directory.
  const env = loadEnv(mode, process.cwd(), '')

  const backendPort = env.VITE_BACKEND_PORT || '8080'

  return {
    plugins: [vue()],
    base: '/',
    server: {
      port: parseInt(env.VITE_DEV_SERVER_PORT || '3000', 10),
      proxy: {
        '/api': {
          target: `http://localhost:${backendPort}`,
          changeOrigin: true
        },
        '/ws': {
          target: `http://localhost:${backendPort}`,
          changeOrigin: true,
          ws: true
        }
      }
    },
    resolve: {
      alias: {
        '@': resolve(__dirname, 'src')
      }
    },
    define: {
      global: 'window'
    }
  }
})
