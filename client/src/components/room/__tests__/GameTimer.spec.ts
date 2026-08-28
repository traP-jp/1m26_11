import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'

import GameTimer from '../GameTimer.vue'

describe('GameTimer', () => {
  beforeEach(() => {
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.useRealTimers()
  })

  it('displays server milliseconds as minutes and seconds', () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: 65_000, active: false } })

    expect(wrapper.text()).toBe('1m5s')
    expect(wrapper.attributes('aria-label')).toBe('経過時間 1分5秒')
  })

  it('rounds elapsed milliseconds down to a whole second', () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: 119_820, active: false } })

    expect(wrapper.text()).toBe('1m59s')
  })

  it('does not display a negative elapsed time', () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: -1, active: false } })

    expect(wrapper.text()).toBe('0m0s')
  })

  it('ticks from the server value while active', async () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: 65_000, active: true } })

    await vi.advanceTimersByTimeAsync(1_000)

    expect(wrapper.text()).toBe('1m6s')
  })

  it('resynchronizes when a new server value arrives', async () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: 65_000, active: true } })
    await vi.advanceTimersByTimeAsync(2_000)
    expect(wrapper.text()).toBe('1m7s')

    await wrapper.setProps({ serverElapsedMs: 10_000 })
    expect(wrapper.text()).toBe('0m10s')

    await vi.advanceTimersByTimeAsync(1_000)
    expect(wrapper.text()).toBe('0m11s')
  })

  it('stops ticking when inactive', async () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: 65_000, active: true } })
    await vi.advanceTimersByTimeAsync(1_000)

    await wrapper.setProps({ serverElapsedMs: 66_000, active: false })
    await nextTick()
    await vi.advanceTimersByTimeAsync(5_000)

    expect(wrapper.text()).toBe('1m6s')
  })
})
