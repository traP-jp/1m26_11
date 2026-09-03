import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import UserMenu from '../UserMenu.vue'

describe('UserMenu', () => {
  it('uses a native menu trigger and delegates a NeoShowcase logout link', async () => {
    const wrapper = mount(UserMenu, {
      props: {
        displayName: 'kaomojikun',
        logoutHref: '/_oauth/logout?redirect=/',
        logoutPending: false,
      },
    })

    const trigger = wrapper.get('button')
    expect(trigger.attributes('aria-haspopup')).toBe('menu')
    expect(trigger.attributes('aria-expanded')).toBe('false')

    await trigger.trigger('click')
    expect(trigger.attributes('aria-expanded')).toBe('true')

    const logoutLink = wrapper.get('a[href="/_oauth/logout?redirect=/"]')
    await logoutLink.trigger('click')
    expect(wrapper.emitted('logout')).toHaveLength(1)
  })

  it('disables the menu trigger while an authentication operation is busy', async () => {
    const wrapper = mount(UserMenu, {
      props: {
        displayName: 'kaomojikun',
        logoutHref: '/_oauth/logout?redirect=/',
        logoutPending: true,
      },
    })

    const trigger = wrapper.get('button')
    expect(trigger.attributes('disabled')).toBeDefined()
    await trigger.trigger('click')

    expect(trigger.attributes('aria-expanded')).toBe('false')
    expect(wrapper.find('[role="menu"]').exists()).toBe(false)
    expect(wrapper.emitted('logout')).toBeUndefined()
  })
})
