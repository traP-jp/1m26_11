import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import RoomCard from '../RoomCard.vue'

const room = {
  id: 'room-01',
  number: 1,
  title: 'Room 01',
  genre: 'General',
  description: 'A cozy room for general discussions.',
}

describe('RoomCard', () => {
  it('renders the collapsed card shell', () => {
    const wrapper = mount(RoomCard, { props: { room } })

    expect(wrapper.get('article').classes()).toContain('room-card')
    expect(wrapper.get('button').text()).toContain('Room 01')
    expect(wrapper.find('.room-card__body').exists()).toBe(false)
  })

  it('opens and closes its content from the header button', async () => {
    const wrapper = mount(RoomCard, { props: { room } })

    await wrapper.get('button').trigger('click')
    expect(wrapper.get('.room-card__description').text()).toBe(room.description)

    await wrapper.get('button').trigger('click')
    expect(wrapper.find('.room-card__body').exists()).toBe(false)
  })

  it('emits the room from the start button', async () => {
    const wrapper = mount(RoomCard, { props: { room, defaultOpen: true } })

    await wrapper.get('.room-card__start').trigger('click')

    expect(wrapper.emitted('start')).toEqual([[room]])
  })
})
