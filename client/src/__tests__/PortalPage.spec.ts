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

  it('shows every published room with its progress after authentication', () => {
    const fixture = portalPageFixtures.demoAuthenticated
    const secondRoom = {
      room: { ...fixture.rooms[0]!.room, room_id: 'room-2', number: 2, name: '次の部屋' },
      progressStatus: 'cleared' as const,
    }
    const wrapper = mount(PortalPage, {
      props: { ...fixture, rooms: [...fixture.rooms, secondRoom] },
    })

    expect(wrapper.findComponent(PortalLoginPrompt).exists()).toBe(false)
    expect(wrapper.findAllComponents(RoomCard).map((card) => card.props('room'))).toEqual([
      fixture.rooms[0]!.room,
      secondRoom.room,
    ])
    expect(
      wrapper.findAllComponents(MinimalProgressSummary).map((summary) => summary.props('status')),
    ).toEqual(['active', 'cleared'])
  })

  it('shows an explicit empty state when no rooms are published', () => {
    const wrapper = mount(PortalPage, {
      props: { ...portalPageFixtures.demoAuthenticated, rooms: [] },
    })

    expect(wrapper.text()).toContain('公開中の部屋はありません。')
    expect(wrapper.findComponent(RoomCard).exists()).toBe(false)
  })

  it('shows a Room loading error and emits a retry request', async () => {
    const wrapper = mount(PortalPage, {
      props: {
        ...portalPageFixtures.demoAuthenticated,
        rooms: [],
        roomsError: 'Room一覧を読み込めませんでした。',
      },
    })

    expect(wrapper.get('[role="alert"]').text()).toContain('Room一覧を読み込めませんでした。')
    await wrapper.get('[role="alert"] button').trigger('click')
    expect(wrapper.emitted('retryRooms')).toHaveLength(1)
  })

  it('forwards header and required-room events', () => {
    const wrapper = mount(PortalPage, { props: portalPageFixtures.demoAuthenticated })

    wrapper.getComponent(PortalHeader).vm.$emit('logout')
    wrapper.getComponent(PortalHeader).vm.$emit('showInstructions')
    wrapper.getComponent(RoomCard).vm.$emit('start', 'room-1')

    expect(wrapper.emitted('logout')).toHaveLength(1)
    expect(wrapper.emitted('showInstructions')).toHaveLength(1)
    expect(wrapper.emitted('startRoom')).toEqual([['room-1']])
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
