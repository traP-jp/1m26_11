import { createApp } from 'vue'
import App from './App.vue'
import './assets/main.css'
import { isMswEnabled } from './mocks/config'
import { router } from './router'

async function bootstrap(): Promise<void> {
  if (
    import.meta.env.DEV &&
    isMswEnabled({
      dev: import.meta.env.DEV,
      enabled: import.meta.env.VITE_ENABLE_MSW,
    })
  ) {
    const { startMockWorker } = await import('./mocks/browser')
    await startMockWorker()
  }

  createApp(App).use(router).mount('#app')
}

void bootstrap()
