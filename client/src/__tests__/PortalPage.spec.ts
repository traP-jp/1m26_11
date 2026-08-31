import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

import PortalHeader from '../components/portal/PortalHeader.vue'
import PortalPage from '../PortalPage.vue'
import RoomCard from '../RoomCard.vue'
import meAuthenticated from '../../../openapi/examples/auth/me-demo-authenticated.json'
import meUnauthenticated from '../../../openapi/examples/auth/me-demo-unauthenticated.json'
import type { ApiClient, GetMeResponse } from '../api/client'
import { authApiClientKey } from '../utils/auth'

const apiClient: ApiClient = {
  getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(meAuthenticated as GetMeResponse),
  loginGuest: vi.fn<ApiClient['loginGuest']>(),
  logoutDemo: vi.fn<ApiClient['logoutDemo']>(),
  startOrResumeRun: vi.fn<ApiClient['startOrResumeRun']>(),
  getCurrentRun: vi.fn<ApiClient['getCurrentRun']>(),
  getProblem: vi.fn<ApiClient['getProblem']>(),
  submitQuery: vi.fn<ApiClient['submitQuery']>(),
  submitAnswer: vi.fn<ApiClient['submitAnswer']>(),
}

async function mountPortal() {
  const wrapper = mount(PortalPage, {
    global: { provide: { [authApiClientKey as symbol]: apiClient } },
  })
  await new Promise((resolve) => setTimeout(resolve))
  return wrapper
}

describe('PortalPage', () => {
  it('renders PortalHeader above the portal content', async () => {
    const wrapper = await mountPortal()

    expect(wrapper.findComponent(PortalHeader).exists()).toBe(true)
    expect(wrapper.element.firstElementChild?.tagName).toBe('HEADER')
  })

  it('renders RoomCard instances from the mock room list', async () => {
    const wrapper = await mountPortal()
    const roomCards = wrapper.findAllComponents(RoomCard)

    expect(roomCards).toHaveLength(2)
    expect(roomCards[0]?.props('room')).toEqual({
      room_id: '1411824c-d357-4941-af76-c76cb827dda6',
      number: 1,
      name: '最初の部屋',
      genre: 'logic',
      description: '動作確認用の問題セットです',
    })
  })

  it('notifies its parent when a room is selected', async () => {
    const wrapper = await mountPortal()
    const firstRoom = wrapper.findAllComponents(RoomCard)[0]

    await firstRoom?.vm.$emit('start', '1411824c-d357-4941-af76-c76cb827dda6')

    expect(wrapper.emitted('roomSelected')).toEqual([['1411824c-d357-4941-af76-c76cb827dda6']])
  })

  it('focuses the guest name input from the Demo login button', async () => {
    const guestApiClient = {
      ...apiClient,
      getMe: vi.fn<ApiClient['getMe']>().mockResolvedValue(meUnauthenticated as GetMeResponse),
    }
    const wrapper = mount(PortalPage, {
      attachTo: document.body,
      global: { provide: { [authApiClientKey as symbol]: guestApiClient } },
    })
    await new Promise((resolve) => setTimeout(resolve))

    await wrapper.get('header button').trigger('click')

    expect(document.activeElement).toBe(wrapper.get('#displayNameInput').element)
    wrapper.unmount()
  })
})
