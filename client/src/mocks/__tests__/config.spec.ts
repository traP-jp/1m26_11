import { describe, expect, it } from 'vitest'

import { isMswEnabled } from '../config'

describe('MSW configuration', () => {
  it('enables MSW by default only during development', () => {
    expect(isMswEnabled({ dev: true })).toBe(true)
    expect(isMswEnabled({ dev: false })).toBe(false)
  })

  it('allows development to opt out explicitly', () => {
    expect(isMswEnabled({ dev: true, enabled: 'false' })).toBe(false)
    expect(isMswEnabled({ dev: true, enabled: 'true' })).toBe(true)
  })
})
