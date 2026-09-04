import { mount } from '@vue/test-utils'
import { describe, expect, it } from 'vitest'

import AuthActionButton from '../components/auth/AuthActionButton.vue'
import GuestNameForm from '../components/auth/GuestNameForm.vue'
import MinimalProgressSummary from '../components/portal/MinimalProgressSummary.vue'
import PortalHeader from '../components/portal/PortalHeader.vue'
import PortalLoginPrompt from '../components/portal/PortalLoginPrompt.vue'
import { portalPageFixtures } from '../PortalPage.fixture'
import PortalPage from '../PortalPage.vue'
import RoomCard from '../RoomCard.vue'

describe('PortalPage', () => {
  it('shows the Demo guest form and hides room progress before authentication', () => {
    const wrapper = mount(PortalPage, { props: portalPageFixtures.demoUnauthenticated })

    expect(wrapper.findComponent(PortalHeader).exists()).toBe(true)
    expect(wrapper.element.firstElementChild?.tagName).toBe('HEADER')
    expect(wrapper.findComponent(PortalLoginPrompt).exists()).toBe(true)
    expect(wrapper.findComponent(GuestNameForm).exists()).toBe(true)
    expect(wrapper.findComponent(RoomCard).exists()).toBe(false)
    expect(wrapper.findComponent(MinimalProgressSummary).exists()).toBe(false)
  })

  it('shows the NeoShowcase login URL and delegates its activation', async () => {
    const fixture = portalPageFixtures.neoshowcaseUnauthenticated
    const wrapper = mount(PortalPage, { props: fixture })

    expect(wrapper.getComponent(PortalHeader).props('userStatus')).toMatchObject({
      authenticated: false,
      loginHref: fixture.loginHref,
    })
    expect(wrapper.getComponent(AuthActionButton).element.tagName).toBe('A')
    expect(wrapper.getComponent(AuthActionButton).attributes('href')).toBe(fixture.loginHref)
    await wrapper.getComponent(AuthActionButton).trigger('click')
    expect(wrapper.emitted('login')).toHaveLength(1)
  })

  it('forwards the NeoShowcase login action when no login URL is available', async () => {
    const wrapper = mount(PortalPage, {
      props: { ...portalPageFixtures.neoshowcaseUnauthenticated, loginHref: null },
    })

    await wrapper.getComponent(AuthActionButton).trigger('click')

    expect(wrapper.emitted('login')).toHaveLength(1)
  })

  it('forwards the guest name without transforming it', () => {
    const wrapper = mount(PortalPage, { props: portalPageFixtures.demoUnauthenticated })

    wrapper.getComponent(GuestNameForm).vm.$emit('submit', 'kaomojikun')

    expect(wrapper.emitted('guestLogin')).toEqual([['kaomojikun']])
  })

  it('shows one required room and progress after authentication', () => {
    const fixture = portalPageFixtures.demoAuthenticated
    const wrapper = mount(PortalPage, { props: fixture })

    expect(wrapper.findComponent(PortalLoginPrompt).exists()).toBe(false)
    expect(wrapper.getComponent(RoomCard).props('room')).toEqual(fixture.requiredRoom)
    expect(wrapper.getComponent(MinimalProgressSummary).props('status')).toBe(
      fixture.progressStatus,
    )
  })

  it('forwards header and required-room events', async () => {
    const fixture = portalPageFixtures.demoAuthenticated
    const wrapper = mount(PortalPage, { props: fixture })

    wrapper.getComponent(PortalHeader).vm.$emit('logout')
    wrapper.getComponent(PortalHeader).vm.$emit('showInstructions')
    wrapper.getComponent(RoomCard).vm.$emit('start', 'room-1')
    await wrapper.get('[data-testid="portal-author-problem"]').trigger('click')

    expect(wrapper.emitted('logout')).toHaveLength(1)
    expect(wrapper.emitted('showInstructions')).toHaveLength(1)
    expect(wrapper.emitted('startRoom')).toEqual([['room-1']])
    expect(wrapper.emitted('authorRoom')).toEqual([[fixture.requiredRoom.room_id]])
  })

  it('hides problem authoring in NeoShowcase mode', () => {
    const wrapper = mount(PortalPage, { props: portalPageFixtures.cleared })

    expect(wrapper.find('[data-testid="portal-author-problem"]').exists()).toBe(false)
  })

  it('focuses the guest name input when Demo login is requested from the header', () => {
    const wrapper = mount(PortalPage, {
      props: portalPageFixtures.demoUnauthenticated,
      attachTo: document.body,
    })

    wrapper.getComponent(PortalHeader).vm.$emit('login')

    expect(document.activeElement).toBe(wrapper.get('#displayNameInput').element)
    wrapper.unmount()
  })
})
