import { describe, expect, it } from 'vitest'
import { mount } from '@vue/test-utils'

import RoomPage from '../RoomPage.vue'
import { roomPageFixture } from '../RoomPage.fixture'

describe('RoomPage', () => {
  it('passes the ViewModel to its child and forwards semantic UI events', async () => {
    const wrapper = mount(RoomPage, { props: { viewModel: roomPageFixture } })

    expect(wrapper.get('h1').text()).toBe(roomPageFixture.room.name)
    await wrapper.get('button').trigger('click')

    expect(wrapper.emitted('uiEvent')).toEqual([[{ type: 'room-exited' }]])
  })
})
