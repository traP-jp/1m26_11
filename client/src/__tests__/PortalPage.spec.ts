import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import PortalHeader from '../components/portal/PortalHeader.vue'
import PortalPage from '../PortalPage.vue'
import RoomCard from '../RoomCard.vue'

describe('PortalPage', () => {
  it('renders PortalHeader above the portal content', () => {
    const wrapper = mount(PortalPage)

    expect(wrapper.findComponent(PortalHeader).exists()).toBe(true)
    expect(wrapper.element.firstElementChild?.tagName).toBe('HEADER')
  })

  it('renders RoomCard instances from the mock room list', () => {
    const wrapper = mount(PortalPage)
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
    const wrapper = mount(PortalPage)
    const firstRoom = wrapper.findAllComponents(RoomCard)[0]

    await firstRoom?.vm.$emit('start', '1411824c-d357-4941-af76-c76cb827dda6')

    expect(wrapper.emitted('roomSelected')).toEqual([['1411824c-d357-4941-af76-c76cb827dda6']])
  })
})
