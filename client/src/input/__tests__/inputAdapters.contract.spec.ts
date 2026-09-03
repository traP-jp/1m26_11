import { describe, expect, it } from 'vitest'

import hardwareSample from '../__fixtures__/serial-protocol-v1-hardware-sample.jsonl?raw'
import type { InputAdapterDispatcher, InputAdapterEvent } from '../InputAdapter.types'
import { createKeyboardInputAdapter } from '../keyboardInputAdapter'
import { createOperationBuffer } from '../operationBuffer'
import { createScreenButtonInputAdapter } from '../screenButtonInputAdapter'
import { createWebSerialInputAdapter } from '../webSerialInputAdapter'

const encoder = new TextEncoder()

function createRecorder(): {
  dispatcher: InputAdapterDispatcher
  events: InputAdapterEvent[]
  snapshot: () => ReturnType<ReturnType<typeof createOperationBuffer>['snapshot']>
} {
  const events: InputAdapterEvent[] = []
  const buffer = createOperationBuffer()

  return {
    dispatcher: {
      busy: false,
      dispatch(event) {
        events.push(event)
        if (event.type === 'condition-changed') buffer.append(event)
        return true
      },
    },
    events,
    snapshot: () => buffer.snapshot(),
  }
}

describe('input adapter event contract', () => {
  it('uses one control/count shape for serial, keyboard, and screen input', () => {
    const serial = createRecorder()
    const keyboard = createRecorder()
    const screen = createRecorder()
    const keyboardTarget = new EventTarget()

    createWebSerialInputAdapter({
      dispatcher: serial.dispatcher,
      isControlAllowed: () => true,
    }).pushChunk(encoder.encode(hardwareSample))

    const keyboardAdapter = createKeyboardInputAdapter({
      dispatcher: keyboard.dispatcher,
      isControlAllowed: () => true,
      target: keyboardTarget,
      getActiveElement: () => null,
    })
    keyboardAdapter.start()
    keyboardTarget.dispatchEvent(new KeyboardEvent('keydown', { key: 'ArrowUp' }))
    keyboardAdapter.stop()

    createScreenButtonInputAdapter({
      dispatcher: screen.dispatcher,
      isControlAllowed: () => true,
    }).pressControl('up')

    expect(serial.events).toEqual([
      { type: 'condition-changed', source: 'serial', control: 'up', count: 1 },
    ])
    expect(keyboard.events).toEqual([
      { type: 'condition-changed', source: 'keyboard', control: 'up', count: 1 },
    ])
    expect(screen.events).toEqual([
      { type: 'condition-changed', source: 'mouse', control: 'up', count: 1 },
    ])
    expect([serial.snapshot(), keyboard.snapshot(), screen.snapshot()]).toEqual([
      [{ control: 'up', count: 1 }],
      [{ control: 'up', count: 1 }],
      [{ control: 'up', count: 1 }],
    ])
  })
})
