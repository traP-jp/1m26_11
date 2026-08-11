import { setupWorker } from 'msw/browser'

import { createMockApi } from './handlers'

const mock = createMockApi({ scenarioId: import.meta.env.VITE_MSW_SCENARIO })
const worker = setupWorker(...mock.handlers)

export async function startMockWorker(): Promise<void> {
  await worker.start({
    serviceWorker: { url: '/mockServiceWorker.js' },
    onUnhandledRequest(request, print) {
      const pathname = new URL(request.url).pathname
      if (pathname === '/openapi.yaml' || pathname.startsWith('/api/')) {
        print.error()
      }
    },
  })
}
