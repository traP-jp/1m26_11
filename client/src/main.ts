import { createApp } from 'vue'
import App from './App.vue'
import './assets/main.css'
import { isMswEnabled } from './mocks/config'

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

  createApp(App).mount('#app')
}

void bootstrap()
