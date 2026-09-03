import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import AuthActionButton from '../AuthActionButton.vue'

describe('AuthActionButton', () => {
  it('emits activate once when enabled', async () => {
    const wrapper = mount(AuthActionButton, { props: { action: 'login' } })

    await wrapper.trigger('click')

    expect(wrapper.emitted('activate')).toHaveLength(1)
  })

  it('does not emit while disabled', async () => {
    const wrapper = mount(AuthActionButton, {
      props: { action: 'logout', disabled: true },
    })

    await wrapper.trigger('click')

    expect(wrapper.emitted('activate')).toBeUndefined()
  })

  it('renders an enabled href action as a link and delegates its activation', async () => {
    const wrapper = mount(AuthActionButton, {
      props: { action: 'login', href: '/_oauth/login?redirect=/' },
    })

    expect(wrapper.element.tagName).toBe('A')
    expect(wrapper.attributes('href')).toBe('/_oauth/login?redirect=/')
    await wrapper.trigger('click')
    expect(wrapper.emitted('activate')).toHaveLength(1)
  })
})
