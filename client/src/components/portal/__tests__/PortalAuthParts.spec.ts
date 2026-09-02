import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import MinimalProgressSummary from '../MinimalProgressSummary.vue'
import PortalLoginPrompt from '../PortalLoginPrompt.vue'

describe('Portal auth parts', () => {
  it('renders the fixed login copy and action slot', () => {
    const wrapper = mount(PortalLoginPrompt, {
      slots: { action: '<button>ログインする</button>' },
    })

    expect(wrapper.text()).toContain('ログインしてゲームをはじめる')
    expect(wrapper.get('button').text()).toBe('ログインする')
  })

  it.each([
    ['not_started', '未開始'],
    ['active', '挑戦中'],
    ['cleared', 'クリア済み'],
  ] as const)('shows %s progress without emitting events', (status, label) => {
    const wrapper = mount(MinimalProgressSummary, { props: { status } })

    expect(wrapper.text()).toContain(label)
    expect(wrapper.emitted()).toEqual({})
  })
})
