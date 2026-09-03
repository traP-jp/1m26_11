import { describe, expect, it, vi } from 'vitest'

import type { InputAdapterDispatcher, InputAdapterEvent } from '../InputAdapter.types'
import { createKeyboardInputAdapter } from '../keyboardInputAdapter'

function createRecordingDispatcher(accepted = true): {
  dispatcher: InputAdapterDispatcher
  events: InputAdapterEvent[]
} {
  const events: InputAdapterEvent[] = []

  return {
    dispatcher: {
      busy: false,
      dispatch(event) {
        if (accepted) events.push(event)
        return accepted
      },
    },
    events,
  }
}

function dispatchKey(
  target: EventTarget,
  key: string,
  init: KeyboardEventInit = {},
): KeyboardEvent {
  const event = new KeyboardEvent('keydown', {
    key,
    bubbles: true,
    cancelable: true,
    ...init,
  })
  target.dispatchEvent(event)
  return event
}

describe('createKeyboardInputAdapter', () => {
  it('maps all seven controls and Enter to keyboard events', () => {
    const target = document.createElement('div')
    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })
    adapter.start()

    const keyEvents = [
      dispatchKey(target, 'ArrowUp'),
      dispatchKey(target, 'ArrowDown'),
      dispatchKey(target, 'ArrowLeft'),
      dispatchKey(target, 'ArrowRight'),
      dispatchKey(target, 'r'),
      dispatchKey(target, 'y'),
      dispatchKey(target, 'g'),
      dispatchKey(target, 'Enter'),
    ]

    expect(events).toEqual([
      { type: 'condition-changed', source: 'keyboard', control: 'up', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'down', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'left', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'right', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'red', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'yellow', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'green', count: 1 },
      { type: 'query-submitted', source: 'keyboard' },
    ])
    expect(keyEvents.every((event) => event.defaultPrevented)).toBe(true)
  })

  it('ignores controls that are unavailable for the current problem', () => {
    const target = document.createElement('div')
    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: (control) => control === 'up',
      target,
      getActiveElement: () => null,
    })
    adapter.start()

    const unavailable = dispatchKey(target, 'ArrowLeft')
    const available = dispatchKey(target, 'ArrowUp')
    const unrelated = dispatchKey(target, 'a')

    expect(events).toEqual([
      { type: 'condition-changed', source: 'keyboard', control: 'up', count: 1 },
    ])
    expect(unavailable.defaultPrevented).toBe(false)
    expect(available.defaultPrevented).toBe(true)
    expect(unrelated.defaultPrevented).toBe(false)
  })

  it('accepts color keys with Caps Lock without accepting Shift shortcuts', () => {
    const target = new EventTarget()
    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })
    adapter.start()

    const capsLockKeys = [
      dispatchKey(target, 'R'),
      dispatchKey(target, 'Y'),
      dispatchKey(target, 'G'),
    ]
    const shifted = dispatchKey(target, 'R', { shiftKey: true })

    expect(events).toEqual([
      { type: 'condition-changed', source: 'keyboard', control: 'red', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'yellow', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'green', count: 1 },
    ])
    expect(capsLockKeys.every((event) => event.defaultPrevented)).toBe(true)
    expect(shifted.defaultPrevented).toBe(false)
  })

  it('does not fire shortcuts from native or contenteditable text-entry targets', () => {
    const target = document.createElement('div')
    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })
    adapter.start()

    const input = document.createElement('input')
    const textarea = document.createElement('textarea')
    const select = document.createElement('select')
    const editable = document.createElement('div')
    const editableChild = document.createElement('span')
    editable.setAttribute('contenteditable', 'true')
    editable.append(editableChild)
    target.append(input, textarea, select, editable)

    const keyEvents = [input, textarea, select, editableChild].map((element) =>
      dispatchKey(element, 'ArrowUp'),
    )

    expect(events).toEqual([])
    expect(keyEvents.every((event) => !event.defaultPrevented)).toBe(true)
  })

  it('does not fire while an editable element is active elsewhere', () => {
    const target = new EventTarget()
    const textarea = document.createElement('textarea')
    const { dispatcher, events } = createRecordingDispatcher()
    let activeElement: EventTarget | null = textarea
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => activeElement,
    })
    adapter.start()

    const guardedEvent = dispatchKey(target, 'Enter')
    activeElement = null
    const acceptedEvent = dispatchKey(target, 'Enter')

    expect(events).toEqual([{ type: 'query-submitted', source: 'keyboard' }])
    expect(guardedEvent.defaultPrevented).toBe(false)
    expect(acceptedEvent.defaultPrevented).toBe(true)
  })

  it('leaves Enter to focused actions while keeping control shortcuts available', () => {
    const target = document.createElement('div')
    const button = document.createElement('button')
    const buttonLabel = document.createElement('span')
    const link = document.createElement('a')
    link.href = '/next'
    button.append(buttonLabel)
    target.append(button, link)

    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })
    adapter.start()

    const buttonEvent = dispatchKey(buttonLabel, 'Enter')
    const linkEvent = dispatchKey(link, 'Enter')
    const buttonControl = dispatchKey(buttonLabel, 'ArrowUp')
    const linkControl = dispatchKey(link, 'r')

    expect(events).toEqual([
      { type: 'condition-changed', source: 'keyboard', control: 'up', count: 1 },
      { type: 'condition-changed', source: 'keyboard', control: 'red', count: 1 },
    ])
    expect(buttonEvent.defaultPrevented).toBe(false)
    expect(linkEvent.defaultPrevented).toBe(false)
    expect(buttonControl.defaultPrevented).toBe(true)
    expect(linkControl.defaultPrevented).toBe(true)
  })

  it('uses the focused document element to guard window shortcuts', () => {
    const host = document.createElement('div')
    const input = document.createElement('input')
    const textarea = document.createElement('textarea')
    const button = document.createElement('button')
    host.append(input, textarea, button)
    document.body.append(host)

    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
    })

    try {
      adapter.start()

      input.focus()
      expect(document.activeElement).toBe(input)
      const inputEvent = dispatchKey(window, 'ArrowUp')

      textarea.focus()
      expect(document.activeElement).toBe(textarea)
      const textareaEvent = dispatchKey(window, 'Enter')

      button.focus()
      expect(document.activeElement).toBe(button)
      const buttonEvent = dispatchKey(window, 'Enter')
      const buttonArrowEvent = dispatchKey(window, 'ArrowUp')
      const buttonColorEvent = dispatchKey(window, 'g')

      expect(events).toEqual([
        { type: 'condition-changed', source: 'keyboard', control: 'up', count: 1 },
        { type: 'condition-changed', source: 'keyboard', control: 'green', count: 1 },
      ])
      expect(inputEvent.defaultPrevented).toBe(false)
      expect(textareaEvent.defaultPrevented).toBe(false)
      expect(buttonEvent.defaultPrevented).toBe(false)
      expect(buttonArrowEvent.defaultPrevented).toBe(true)
      expect(buttonColorEvent.defaultPrevented).toBe(true)
    } finally {
      adapter.stop()
      host.remove()
    }
  })

  it('ignores composition, modifiers, and repeated mapped keys', () => {
    const target = new EventTarget()
    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })
    adapter.start()

    const guardedEvents = [
      dispatchKey(target, 'ArrowUp', { isComposing: true }),
      dispatchKey(target, 'ArrowUp', { altKey: true }),
      dispatchKey(target, 'ArrowUp', { ctrlKey: true }),
      dispatchKey(target, 'ArrowUp', { metaKey: true }),
      dispatchKey(target, 'ArrowUp', { shiftKey: true }),
      dispatchKey(target, 'r', { altKey: true }),
      dispatchKey(target, 'r', { ctrlKey: true }),
      dispatchKey(target, 'r', { metaKey: true }),
      dispatchKey(target, 'R', { shiftKey: true }),
    ]
    const repeatedControl = dispatchKey(target, 'ArrowUp', { repeat: true })
    const repeatedColor = dispatchKey(target, 'r', { repeat: true })
    const repeatedSubmit = dispatchKey(target, 'Enter', { repeat: true })

    expect(events).toEqual([])
    expect(guardedEvents.every((event) => !event.defaultPrevented)).toBe(true)
    expect(repeatedControl.defaultPrevented).toBe(true)
    expect(repeatedColor.defaultPrevented).toBe(true)
    expect(repeatedSubmit.defaultPrevented).toBe(true)
  })

  it('forwards answers unchanged without adding them to the operation events', () => {
    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target: new EventTarget(),
      getActiveElement: () => null,
    })

    expect(adapter.submitAnswer('  answer\n')).toBe(true)
    expect(events).toEqual([{ type: 'answer-submitted', source: 'keyboard', answer: '  answer\n' }])

    const rejected = createRecordingDispatcher(false)
    const rejectedAdapter = createKeyboardInputAdapter({
      dispatcher: rejected.dispatcher,
      isControlAllowed: () => true,
      target: new EventTarget(),
      getActiveElement: () => null,
    })

    expect(rejectedAdapter.submitAnswer('answer')).toBe(false)
    expect(rejected.events).toEqual([])
  })

  it('starts and stops idempotently and removes its listener', () => {
    const target = new EventTarget()
    const addEventListener = vi.spyOn(target, 'addEventListener')
    const removeEventListener = vi.spyOn(target, 'removeEventListener')
    const { dispatcher, events } = createRecordingDispatcher()
    const adapter = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })

    adapter.start()
    adapter.start()
    dispatchKey(target, 'ArrowRight')
    adapter.stop()
    adapter.stop()
    dispatchKey(target, 'ArrowRight')

    expect(addEventListener).toHaveBeenCalledTimes(1)
    expect(removeEventListener).toHaveBeenCalledTimes(1)
    expect(events).toEqual([
      { type: 'condition-changed', source: 'keyboard', control: 'right', count: 1 },
    ])
  })
})
