import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import GameTimer from '../GameTimer.vue'

describe('GameTimer', () => {
  it('displays server milliseconds as minutes and seconds', () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: 65_000 } })

    expect(wrapper.text()).toBe('1m5s')
    expect(wrapper.attributes('aria-label')).toBe('経過時間 1分5秒')
  })

  it('rounds elapsed milliseconds down to a whole second', () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: 119_820 } })

    expect(wrapper.text()).toBe('1m59s')
  })

  it('does not display a negative elapsed time', () => {
    const wrapper = mount(GameTimer, { props: { serverElapsedMs: -1 } })

    expect(wrapper.text()).toBe('0m0s')
  })
})
