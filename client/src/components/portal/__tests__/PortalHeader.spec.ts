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
    const homeLink = wrapper.get('a[aria-label="ワンマンそん ホーム"]')
    expect(homeLink.text()).toBe('ワンマンそん')
    expect(homeLink.attributes('href')).toBe('/')
    expect(wrapper.get('a[href="#instructions"]').text()).toBe('操作説明')
    expect(wrapper.text()).not.toContain('kaomojikun')

    await wrapper.get('button').trigger('click')

    expect(wrapper.emitted('login')).toHaveLength(1)
    expect(wrapper.emitted('logout')).toBeUndefined()
  })

  it('delegates the NeoShowcase login link and disables it while busy', async () => {
    const wrapper = mount(PortalHeader, {
      props: {
        ...portalHeaderFixtures.unauthenticated,
        userStatus: {
          authenticated: false,
          authMode: 'neoshowcase',
          loginHref: '/_oauth/login?redirect=/',
          loginPending: false,
        },
      },
    })

    const loginLink = wrapper.get('a[href="/_oauth/login?redirect=/"]')
    await loginLink.trigger('click')
    expect(wrapper.emitted('login')).toHaveLength(1)

    await wrapper.setProps({
      userStatus: {
        authenticated: false,
        authMode: 'neoshowcase',
        loginHref: '/_oauth/login?redirect=/',
        loginPending: true,
      },
    })

    expect(wrapper.find('a[href="/_oauth/login?redirect=/"]').exists()).toBe(false)
    expect(wrapper.get('button:not([aria-label="操作説明"])').attributes('disabled')).toBeDefined()
    expect(wrapper.get('button:not([aria-label="操作説明"])').attributes('aria-busy')).toBe('true')
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

  it('passes the NeoShowcase user and logout URL to UserMenu', async () => {
    const wrapper = mount(PortalHeader, {
      props: portalHeaderFixtures.neoshowcaseAuthenticated,
    })

    const status = wrapper.get('[data-auth-mode="neoshowcase"]')
    const userMenu = wrapper.getComponent(UserMenu)
    expect(userMenu.props('displayName')).toBe('kaomojikun')
    expect(userMenu.props('logoutHref')).toBe('/_oauth/logout?redirect=/')
    expect(userMenu.props('logoutPending')).toBe(false)

    await userMenu.get('button').trigger('click')
    await nextTick()

    expect(status.attributes('data-auth-mode')).toBe('neoshowcase')
    expect(userMenu.get('a').attributes('href')).toBe('/_oauth/logout?redirect=/')
    await userMenu.get('a').trigger('click')
    await nextTick()
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
