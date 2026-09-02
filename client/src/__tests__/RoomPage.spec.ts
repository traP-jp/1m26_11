import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

import RoomPage from '../RoomPage.vue'
import { roomPageFixture } from '../RoomPage.fixture'
import type { SerialPortLike, WebSerialLike } from '../input/useWebSerialConnection'

describe('RoomPage', () => {
  it('passes the ViewModel to its child and forwards semantic UI events', async () => {
    const wrapper = mount(RoomPage, { props: { viewModel: roomPageFixture } })

    expect(wrapper.get('h1').text()).toBe(roomPageFixture.room.name)
    const exitButton = wrapper.findAll('button').find((button) => button.text() === '退出する')
    expect(exitButton).toBeDefined()
    await exitButton!.trigger('click')

    expect(wrapper.emitted('uiEvent')).toEqual([[{ type: 'room-exited' }]])
  })

  it('keeps the Serial connection UI in a separate input region', () => {
    const wrapper = mount(RoomPage, { props: { viewModel: roomPageFixture } })

    expect(wrapper.get('aside[aria-label="入力方法"]')).toBeTruthy()
    expect(wrapper.get('[data-testid="serial-connect-control"]')).toBeTruthy()
  })

  it('forwards a connected Serial frame as the shared input event and releases it for alternatives', async () => {
    let controller!: ReadableStreamDefaultController<Uint8Array>
    const cancel = vi.fn<(reason?: unknown) => void>()
    const readable = new ReadableStream<Uint8Array>({
      start(streamController) {
        controller = streamController
      },
      cancel,
    })
    const port: SerialPortLike = {
      readable,
      open: vi.fn<SerialPortLike['open']>(async () => undefined),
      close: vi.fn<SerialPortLike['close']>(async () => undefined),
    }
    const serial: WebSerialLike = {
      requestPort: vi.fn<WebSerialLike['requestPort']>(async () => port),
      addEventListener: vi.fn<(type: 'disconnect', listener: (event: Event) => void) => void>(),
      removeEventListener: vi.fn<(type: 'disconnect', listener: (event: Event) => void) => void>(),
    }
    const serialDescriptor = Object.getOwnPropertyDescriptor(navigator, 'serial')
    const secureContextDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'isSecureContext')
    Object.defineProperty(navigator, 'serial', { configurable: true, value: serial })
    Object.defineProperty(globalThis, 'isSecureContext', { configurable: true, value: true })

    const wrapper = mount(RoomPage, { props: { viewModel: roomPageFixture } })
    try {
      const connect = wrapper.findAll('button').find((button) => button.text() === 'Serialへ接続')
      expect(connect).toBeDefined()
      await connect!.trigger('click')
      await vi.waitFor(() => expect(wrapper.text()).toContain('Serial接続済み'))

      controller.enqueue(
        new TextEncoder().encode('{"v":1,"control":"up","gesture":"short_press"}\n'),
      )
      await vi.waitFor(() =>
        expect(wrapper.emitted('uiEvent')).toContainEqual([
          {
            type: 'condition-changed',
            source: 'serial',
            control: 'up',
            count: 1,
            gesture: 'short_press',
          },
        ]),
      )

      const alternative = wrapper
        .findAll('button')
        .find((button) => button.text() === 'キーボード／画面ボタンで続ける')
      expect(alternative).toBeDefined()
      await alternative!.trigger('click')
      await vi.waitFor(() => expect(port.close).toHaveBeenCalledTimes(1))
      expect(cancel).toHaveBeenCalledTimes(1)
    } finally {
      wrapper.unmount()
      if (serialDescriptor) {
        Object.defineProperty(navigator, 'serial', serialDescriptor)
      } else {
        Reflect.deleteProperty(navigator, 'serial')
      }
      if (secureContextDescriptor) {
        Object.defineProperty(globalThis, 'isSecureContext', secureContextDescriptor)
      } else {
        Reflect.deleteProperty(globalThis, 'isSecureContext')
      }
    }
  })
})
