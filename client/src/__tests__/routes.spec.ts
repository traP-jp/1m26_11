import { describe, expect, it } from 'vitest'

import { resolveRoute } from '../routes'

describe('resolveRoute', () => {
  it('resolves the three application routes', () => {
    expect(resolveRoute('/')).toEqual({ name: 'portal' })
    expect(resolveRoute('/rooms/room-1')).toEqual({ name: 'room', roomId: 'room-1' })
    expect(resolveRoute('/rooms/room-1/clear')).toEqual({ name: 'clear', roomId: 'room-1' })
  })

  it('marks an unknown route for fallback', () => {
    expect(resolveRoute('/unknown')).toEqual({ name: 'not-found' })
  })
})
