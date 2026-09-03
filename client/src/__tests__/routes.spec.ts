import { describe, expect, it } from 'vitest'
import { createMemoryHistory } from 'vue-router'

import { createAppRouter } from '../router'

describe('router', () => {
  it('resolves the application routes and the development device PoC route', () => {
    const router = createAppRouter(createMemoryHistory())

    expect(router.resolve('/').name).toBe('portal')
    expect(router.resolve('/rooms/room-1')).toMatchObject({
      name: 'room',
      params: { roomId: 'room-1' },
    })
    expect(router.resolve('/device-poc').name).toBe('device-poc')
  })

  it('redirects the legacy Clear URL to the canonical Room route', async () => {
    const router = createAppRouter(createMemoryHistory())

    await router.push('/rooms/room-1/clear')

    expect(router.currentRoute.value).toMatchObject({
      name: 'room',
      params: { roomId: 'room-1' },
    })
  })

  it('redirects an unknown route to Portal', async () => {
    const router = createAppRouter(createMemoryHistory())

    await router.push('/unknown')

    expect(router.currentRoute.value.name).toBe('portal')
  })
})
