import { describe, expect, it } from 'vitest'
import { createMemoryHistory } from 'vue-router'

import { createAppRouter } from '../router'

describe('router', () => {
  it('resolves the three application routes', () => {
    const router = createAppRouter(createMemoryHistory())

    expect(router.resolve('/').name).toBe('portal')
    expect(router.resolve('/rooms/room-1')).toMatchObject({
      name: 'room',
      params: { roomId: 'room-1' },
    })
    expect(router.resolve('/rooms/room-1/clear')).toMatchObject({
      name: 'clear',
      params: { roomId: 'room-1' },
    })
  })

  it('redirects an unknown route to Portal', async () => {
    const router = createAppRouter(createMemoryHistory())

    await router.push('/unknown')

    expect(router.currentRoute.value.name).toBe('portal')
  })
})
