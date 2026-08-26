import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import RoomCard from '../RoomCard.vue'

const room = {
  room_id: '1411824c-d357-4941-af76-c76cb827dda6',
  number: 1,
  name: '最初の部屋',
  genre: 'logic',
  description: '動作確認用の問題セットです',
}

describe('RoomCard', () => {
  it('renders the collapsed card shell', () => {
    const wrapper = mount(RoomCard, { props: { room } })

    expect(wrapper.get('article').classes()).toContain('room-card')
    expect(wrapper.get('.room-card__header').text()).toContain(room.name)
    expect(wrapper.find('.room-card__body').exists()).toBe(false)
  })

  it('opens and closes its content from the header button', async () => {
    const wrapper = mount(RoomCard, { props: { room } })

    await wrapper.get('.room-card__header').trigger('click')
    expect(wrapper.get('.room-card__description').text()).toBe(room.description)

    await wrapper.get('.room-card__header').trigger('click')
    expect(wrapper.find('.room-card__body').exists()).toBe(false)
  })

  it('emits the room ID from the start button', async () => {
    const wrapper = mount(RoomCard, { props: { room, defaultOpen: true } })

    await wrapper.get('.room-card__start').trigger('click')

    expect(wrapper.emitted('start')).toEqual([[room.room_id]])
  })
})
