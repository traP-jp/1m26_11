import { describe, expect, it, vi } from 'vitest'
import { mount } from '@vue/test-utils'

import type { SubmitQueryRequest } from '@/api/client'

import AnswerPanel from '../components/room/AnswerPanel.vue'
import ClearScreen from '../components/room/ClearScreen.vue'
import QuestionArea from '../components/room/QuestionArea.vue'
import RoomPageShell from '../components/room/RoomPageShell.vue'
import RoomTopBar from '../components/room/RoomTopBar.vue'
import RoomPage from '../RoomPage.vue'
import { roomPageFixture } from '../RoomPage.fixture'
import type { RoomUiEvent } from '../RoomPage.types'
import hardwareSample from '../input/__fixtures__/serial-protocol-v1-hardware-sample.jsonl?raw'
import type { InputSource } from '../input/InputAdapter.types'
import { createOperationBuffer } from '../input/operationBuffer'
import type { SerialPortLike, WebSerialLike } from '../input/useWebSerialConnection'

interface BrowserSerialFixture {
  serial: WebSerialLike
  port: SerialPortLike
  enqueue(bytes: Uint8Array): void
  emitDisconnect(): void
  cancel: ReturnType<typeof vi.fn>
  close: ReturnType<typeof vi.fn>
}

function createBrowserSerialFixture(): BrowserSerialFixture {
  let controller!: ReadableStreamDefaultController<Uint8Array>
  const cancel = vi.fn<() => void>()
  const readable = new ReadableStream<Uint8Array>({
    start(streamController) {
      controller = streamController
    },
    cancel,
  })
  const close = vi.fn<() => Promise<void>>(async () => undefined)
  const port: SerialPortLike = {
    readable,
    open: vi.fn<SerialPortLike['open']>(async () => undefined),
    close,
  }
  const disconnectListeners = new Set<(event: Event) => void>()
  const serial: WebSerialLike = {
    requestPort: vi.fn<WebSerialLike['requestPort']>(async () => port),
    addEventListener(_type, listener) {
      disconnectListeners.add(listener)
    },
    removeEventListener(_type, listener) {
      disconnectListeners.delete(listener)
    },
  }

  return {
    serial,
    port,
    enqueue: (bytes) => controller.enqueue(bytes),
    emitDisconnect() {
      const event = { port } as unknown as Event
      for (const listener of disconnectListeners) listener(event)
    },
    cancel,
    close,
  }
}

function installBrowserSerial(serial: WebSerialLike): () => void {
  const serialDescriptor = Object.getOwnPropertyDescriptor(navigator, 'serial')
  const secureContextDescriptor = Object.getOwnPropertyDescriptor(globalThis, 'isSecureContext')
  Object.defineProperty(navigator, 'serial', { configurable: true, value: serial })
  Object.defineProperty(globalThis, 'isSecureContext', { configurable: true, value: true })

  return () => {
    if (serialDescriptor) Object.defineProperty(navigator, 'serial', serialDescriptor)
    else Reflect.deleteProperty(navigator, 'serial')

    if (secureContextDescriptor) {
      Object.defineProperty(globalThis, 'isSecureContext', secureContextDescriptor)
    } else {
      Reflect.deleteProperty(globalThis, 'isSecureContext')
    }
  }
}

describe('RoomPage', () => {
  it('renders the normal shell and forwards child events', async () => {
    const wrapper = mount(RoomPage, { props: { viewModel: roomPageFixture } })

    expect(wrapper.findComponent(RoomPageShell).exists()).toBe(true)
    expect(wrapper.text()).toContain(roomPageFixture.selectedProblem?.title)
    expect(wrapper.getComponent(QuestionArea).props('problemType')).toBe(
      roomPageFixture.selectedProblem?.type,
    )

    wrapper.getComponent(RoomTopBar).vm.$emit('exit')
    wrapper.getComponent(AnswerPanel).vm.$emit('submit', 'fixture answer', 'mouse')
    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('uiEvent')).toEqual([
      [{ type: 'room-exited' }],
      [{ type: 'answer-submitted', source: 'mouse', answer: 'fixture answer' }],
    ])
  })

  it('switches to ClearScreen with the server elapsed time and forwards Portal navigation', async () => {
    const clearedViewModel = {
      ...roomPageFixture,
      serverElapsedMs: 75_432,
      clear: { ...roomPageFixture.clear, cleared: true },
    }
    const wrapper = mount(RoomPage, { props: { viewModel: clearedViewModel } })

    expect(wrapper.findComponent(RoomPageShell).exists()).toBe(false)
    expect(wrapper.find('[data-testid="serial-status-notice"]').exists()).toBe(false)
    expect(wrapper.get('[data-testid="final-elapsed-time"]').text()).toBe('1:15')

    wrapper.getComponent(ClearScreen).vm.$emit('backToPortal')
    await wrapper.vm.$nextTick()

    expect(wrapper.emitted('uiEvent')).toEqual([[{ type: 'portal-returned' }]])
  })

  it('keeps one room operation buffer across Serial, keyboard, and screen input', async () => {
    const browserSerial = createBrowserSerialFixture()
    const restoreBrowserSerial = installBrowserSerial(browserSerial.serial)
    const buffer = createOperationBuffer()
    const wrapper = mount(RoomPage, {
      props: { viewModel: roomPageFixture },
      attrs: {
        onUiEvent(event: RoomUiEvent) {
          if (event.type === 'condition-changed') buffer.append(event)
        },
      },
      attachTo: document.body,
    })

    try {
      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'connected',
        )
      })

      browserSerial.enqueue(new TextEncoder().encode(hardwareSample))
      await vi.waitFor(() => expect(wrapper.emitted('uiEvent')).toHaveLength(1))

      await wrapper.get('[data-testid="use-keyboard"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-input-mode="keyboard"]')).toBeTruthy()
      })
      expect(browserSerial.cancel).toHaveBeenCalledTimes(1)
      expect(browserSerial.close).toHaveBeenCalledTimes(1)

      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, cancelable: true }),
      )
      expect(wrapper.emitted('uiEvent')).toHaveLength(2)

      await wrapper.get('[data-testid="use-screen"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-input-mode="screen"]')).toBeTruthy()
      })
      await wrapper.get('button[data-control="up"]').trigger('click')

      const events = (wrapper.emitted('uiEvent') ?? []).map(([event]) => event as RoomUiEvent)
      expect(events).toEqual([
        { type: 'condition-changed', source: 'serial', control: 'up', count: 1 },
        { type: 'condition-changed', source: 'keyboard', control: 'up', count: 1 },
        { type: 'condition-changed', source: 'mouse', control: 'up', count: 1 },
      ])

      expect(buffer.snapshot()).toEqual([{ control: 'up', count: 3 }])
    } finally {
      wrapper.unmount()
      restoreBrowserSerial()
    }
  })

  it('activates the latest alternative input while Serial cleanup is still pending', async () => {
    const browserSerial = createBrowserSerialFixture()
    const restoreBrowserSerial = installBrowserSerial(browserSerial.serial)
    let resolveClose!: () => void
    const pendingClose = new Promise<void>((resolve) => {
      resolveClose = resolve
    })
    browserSerial.close.mockImplementationOnce(() => pendingClose)
    const wrapper = mount(RoomPage, {
      props: { viewModel: roomPageFixture },
      attachTo: document.body,
    })

    try {
      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'connected',
        )
      })

      await wrapper.get('[data-testid="use-keyboard"]').trigger('click')
      await vi.waitFor(() => expect(browserSerial.close).toHaveBeenCalledTimes(1))

      const keyboardMode = wrapper.get('[data-input-mode="keyboard"]')
      expect(keyboardMode.text()).toContain('↓（下）、→（右）、↑（上）キー')
      expect(keyboardMode.text()).not.toContain('R（赤）')

      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'ArrowDown', bubbles: true, cancelable: true }),
      )
      expect(wrapper.emitted('uiEvent')).toEqual([
        [{ type: 'condition-changed', source: 'keyboard', control: 'down', count: 1 }],
      ])

      await wrapper.get('[data-testid="use-screen"]').trigger('click')
      const screenMode = wrapper.get('[data-input-mode="screen"]')
      const controlGroup = screenMode.get('[role="group"]')
      expect(controlGroup.attributes('aria-labelledby')).toBe(
        screenMode.attributes('aria-labelledby'),
      )

      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'ArrowRight', bubbles: true, cancelable: true }),
      )
      await wrapper.get('button[data-control="down"]').trigger('click')
      expect(wrapper.emitted('uiEvent')).toEqual([
        [{ type: 'condition-changed', source: 'keyboard', control: 'down', count: 1 }],
        [{ type: 'condition-changed', source: 'mouse', control: 'down', count: 1 }],
      ])

      resolveClose()
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'disconnected',
        )
      })
    } finally {
      resolveClose()
      wrapper.unmount()
      restoreBrowserSerial()
    }
  })

  it('does not reconnect after the user chooses an alternative during a close retry', async () => {
    const browserSerial = createBrowserSerialFixture()
    const restoreBrowserSerial = installBrowserSerial(browserSerial.serial)
    browserSerial.close.mockRejectedValueOnce(new DOMException('Port is busy', 'InvalidStateError'))
    const wrapper = mount(RoomPage, {
      props: { viewModel: roomPageFixture },
      attachTo: document.body,
    })

    let resolveRetriedClose!: () => void
    try {
      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'connected',
        )
      })

      await wrapper.get('[data-testid="use-screen"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'disconnected',
        )
      })
      expect(wrapper.get('[data-input-mode="screen"]')).toBeTruthy()
      expect(wrapper.get('[data-testid="serial-retry"]').text()).toBe('Serialを解放する')

      const pendingRetriedClose = new Promise<void>((resolve) => {
        resolveRetriedClose = resolve
      })
      browserSerial.close.mockImplementationOnce(() => pendingRetriedClose)

      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => expect(browserSerial.close).toHaveBeenCalledTimes(2))
      expect(wrapper.get('[data-input-mode="screen"]')).toBeTruthy()
      expect(wrapper.get<HTMLButtonElement>('[data-testid="serial-retry"]').element.disabled).toBe(
        true,
      )
      await wrapper.get('[data-testid="use-keyboard"]').trigger('click')
      expect(wrapper.get('[data-input-mode="keyboard"]')).toBeTruthy()

      resolveRetriedClose()
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-retry"]').text()).toBe('接続する')
      })
      expect(browserSerial.serial.requestPort).toHaveBeenCalledTimes(1)

      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, cancelable: true }),
      )
      expect(wrapper.emitted('uiEvent')).toEqual([
        [{ type: 'condition-changed', source: 'keyboard', control: 'up', count: 1 }],
      ])
    } finally {
      resolveRetriedClose?.()
      wrapper.unmount()
      restoreBrowserSerial()
    }
  })

  it('shows physical disconnection only for the active Serial port and cleans it up', async () => {
    const browserSerial = createBrowserSerialFixture()
    const restoreBrowserSerial = installBrowserSerial(browserSerial.serial)
    const wrapper = mount(RoomPage, { props: { viewModel: roomPageFixture } })

    try {
      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'connected',
        )
      })

      browserSerial.emitDisconnect()
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'disconnected',
        )
      })
      expect(browserSerial.cancel).toHaveBeenCalledTimes(1)
      expect(browserSerial.close).toHaveBeenCalledTimes(1)
    } finally {
      wrapper.unmount()
      restoreBrowserSerial()
    }
  })

  it('切断後にkeyboard／screen入力をOperationBufferから送信eventまで渡す', async () => {
    const browserSerial = createBrowserSerialFixture()
    const restoreBrowserSerial = installBrowserSerial(browserSerial.serial)
    const operationBuffer = createOperationBuffer()
    const submittedQueries: SubmitQueryRequest[] = []
    const submittedAnswers: Array<{ source: InputSource; answer: string }> = []
    const wrapper = mount(RoomPage, {
      props: { viewModel: roomPageFixture },
      attrs: {
        onUiEvent(event: RoomUiEvent) {
          if (event.type === 'condition-changed') {
            operationBuffer.append(event)
            return
          }

          if (event.type === 'query-submitted') {
            const operations = operationBuffer.snapshot()
            submittedQueries.push({ source: event.source, operations })
            operationBuffer.clear(operations)
            return
          }

          if (event.type === 'answer-submitted') {
            submittedAnswers.push({ source: event.source, answer: event.answer })
          }
        },
      },
      attachTo: document.body,
    })

    try {
      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'connected',
        )
      })

      browserSerial.emitDisconnect()
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'disconnected',
        )
      })

      await wrapper.get('[data-testid="use-keyboard"]').trigger('click')
      const activeElement = document.activeElement
      if (activeElement instanceof HTMLElement) activeElement.blur()
      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'ArrowUp', bubbles: true, cancelable: true }),
      )
      window.dispatchEvent(
        new KeyboardEvent('keydown', { key: 'Enter', bubbles: true, cancelable: true }),
      )
      await vi.waitFor(() => expect(submittedQueries).toHaveLength(1))

      await wrapper.get('[data-testid="use-screen"]').trigger('click')
      await wrapper.get('button[data-control="up"]').trigger('click')
      await wrapper.get('[data-testid="screen-submit-query"]').trigger('click')
      await vi.waitFor(() => expect(submittedQueries).toHaveLength(2))

      await wrapper.get('[data-testid="screen-answer-input"]').setValue('screen answer')
      await wrapper.get('[data-testid="screen-submit-answer"]').trigger('click')
      await vi.waitFor(() => expect(submittedAnswers).toHaveLength(1))

      expect(submittedQueries).toEqual([
        { source: 'keyboard', operations: [{ control: 'up', count: 1 }] },
        { source: 'mouse', operations: [{ control: 'up', count: 1 }] },
      ])
      expect(submittedAnswers).toEqual([{ source: 'mouse', answer: 'screen answer' }])
    } finally {
      wrapper.unmount()
      restoreBrowserSerial()
    }
  })

  it('distinguishes an initial denial from a failed retry', async () => {
    const requestPort = vi.fn<WebSerialLike['requestPort']>()
    requestPort.mockRejectedValue(new DOMException('selection denied', 'NotFoundError'))
    const restoreBrowserSerial = installBrowserSerial({ requestPort })
    const wrapper = mount(RoomPage, { props: { viewModel: roomPageFixture } })

    try {
      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'denied',
        )
      })

      await wrapper.get('[data-testid="serial-retry"]').trigger('click')
      await vi.waitFor(() => {
        expect(wrapper.get('[data-testid="serial-status-notice"]').attributes('data-status')).toBe(
          'retry-failed',
        )
      })
      expect(requestPort).toHaveBeenCalledTimes(2)
    } finally {
      wrapper.unmount()
      restoreBrowserSerial()
    }
  })
})
