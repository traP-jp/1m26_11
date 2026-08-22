import { mount } from '@vue/test-utils'
import { nextTick } from 'vue'
import { describe, expect, it } from 'vitest'

import UserMenu from '../../auth/UserMenu.vue'
import PortalHeader from '../PortalHeader.vue'
import { portalHeaderFixtures } from '../PortalHeader.fixture'

describe('PortalHeader', () => {
  it('renders the header and emits login while unauthenticated', async () => {
    const wrapper = mount(PortalHeader, {
      props: portalHeaderFixtures.unauthenticated,
    })

    expect(wrapper.get('header').element.tagName).toBe('HEADER')
    expect(wrapper.get('[aria-label="ワンマンそん ホーム"]').text()).toBe('ワンマンそん')
    expect(wrapper.get('a[href="#instructions"]').text()).toBe('操作説明')
    expect(wrapper.text()).not.toContain('kaomojikun')

    await wrapper.get('button').trigger('click')

    expect(wrapper.emitted('login')).toHaveLength(1)
    expect(wrapper.emitted('logout')).toBeUndefined()
  })

  it('shows the Demo display name and emits logout from its menu', async () => {
    const wrapper = mount(PortalHeader, {
      props: portalHeaderFixtures.demoAuthenticated,
    })

    const userMenu = wrapper.getComponent(UserMenu)
    expect(userMenu.props('displayName')).toBe('kaomojikun')
    expect(userMenu.props('logoutPending')).toBe(false)

    userMenu.vm.$emit('logout')
    await nextTick()

    expect(wrapper.emitted('logout')).toHaveLength(1)
  })

  it('disables the Demo logout operation while it is pending', async () => {
    const fixture = portalHeaderFixtures.demoAuthenticated
    const wrapper = mount(PortalHeader, {
      props: {
        ...fixture,
        userStatus: {
          ...fixture.userStatus,
          logoutPending: true,
        },
      },
    })

    expect(wrapper.getComponent(UserMenu).props('logoutPending')).toBe(true)
  })

  it('passes the NeoShowcase display name from the shared API fixture to UserMenu', async () => {
    const wrapper = mount(PortalHeader, {
      props: portalHeaderFixtures.neoshowcaseAuthenticated,
    })

    const status = wrapper.get('[data-auth-mode="neoshowcase"]')
    const userMenu = wrapper.getComponent(UserMenu)
    expect(userMenu.props('displayName')).toBe('kaomojikun')
    expect(userMenu.props('logoutPending')).toBe(false)

    userMenu.vm.$emit('logout')
    await nextTick()

    expect(status.attributes('data-auth-mode')).toBe('neoshowcase')
    expect(wrapper.emitted('logout')).toHaveLength(1)
  })

  it('emits an instruction event when no instruction path is supplied', async () => {
    const wrapper = mount(PortalHeader, {
      props: {
        ...portalHeaderFixtures.unauthenticated,
        instructionsHref: null,
      },
    })

    await wrapper.get('button[aria-label="操作説明"]').trigger('click')

    expect(wrapper.emitted('showInstructions')).toHaveLength(1)
  })
})
