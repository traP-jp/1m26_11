import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import App from '../App.vue'
import PortalPage from '../PortalPage.vue'
import RoomPage from '../RoomPage.vue'

describe('App', () => {
  it('renders the portal page at the root route', () => {
    window.history.replaceState({}, '', '/')
    const wrapper = mount(App)

    expect(wrapper.get('h1').text()).toBe('Portal')
  })

  it('falls back to the portal page for an unknown route', () => {
    window.history.replaceState({}, '', '/unknown')
    const wrapper = mount(App)

    expect(wrapper.get('h1').text()).toBe('Portal')
  })

  it('navigates from Portal to Room and back through semantic UI events', async () => {
    window.history.replaceState({}, '', '/')
    const wrapper = mount(App)

    wrapper
      .getComponent(PortalPage)
      .vm.$emit('roomSelected', '1411824c-d357-4941-af76-c76cb827dda6')
    await wrapper.vm.$nextTick()

    expect(window.location.pathname).toBe('/rooms/1411824c-d357-4941-af76-c76cb827dda6')
    expect(wrapper.findComponent(RoomPage).exists()).toBe(true)

    wrapper.getComponent(RoomPage).vm.$emit('uiEvent', { type: 'room-exited' })
    await wrapper.vm.$nextTick()

    expect(window.location.pathname).toBe('/')
    expect(wrapper.findComponent(PortalPage).exists()).toBe(true)
  })

  it('renders the Clear page route', () => {
    window.history.replaceState({}, '', '/rooms/room-1/clear')

    const wrapper = mount(App)

    expect(wrapper.get('h1').text()).toBe('Clear')
  })
})
