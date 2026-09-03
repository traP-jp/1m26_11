import { mount } from '@vue/test-utils'
import { defineComponent, h, ref } from 'vue'
import { describe, expect, it, vi } from 'vitest'

import type {
  InputAdapterDispatcher,
  InputAdapterEvent,
  InputAdapterEventHandler,
} from '../InputAdapter.types'
import { createGuardedInputDispatcher } from '../inputGuard'
import { createKeyboardInputAdapter } from '../keyboardInputAdapter'
import { createOperationBuffer } from '../operationBuffer'
import {
  createScreenButtonInputAdapter,
  type ScreenButtonInputAdapter,
} from '../screenButtonInputAdapter'

function createClickHost(adapter: ScreenButtonInputAdapter, isUnavailable = () => false) {
  return defineComponent({
    setup() {
      const answer = ref('')

      return () =>
        h('div', [
          h(
            'button',
            {
              type: 'button',
              'data-testid': 'control',
              disabled: isUnavailable(),
              onClick: () => adapter.pressControl('up'),
            },
            'up',
          ),
          h(
            'button',
            {
              type: 'button',
              'data-testid': 'query',
              disabled: isUnavailable(),
              onClick: () => adapter.submitQuery(),
            },
            'query',
          ),
          h('textarea', {
            value: answer.value,
            onInput: (event: Event) => {
              if (event.target instanceof HTMLTextAreaElement) answer.value = event.target.value
            },
          }),
          h(
            'button',
            {
              type: 'button',
              'data-testid': 'answer',
              disabled: isUnavailable(),
              onClick: () => adapter.submitAnswer(answer.value),
            },
            'answer',
          ),
        ])
    },
  })
}

function createRecordingDispatcher(events: InputAdapterEvent[]): InputAdapterDispatcher {
  return {
    busy: false,
    dispatch(event) {
      events.push(event)
      return true
    },
  }
}

function createBufferingHandler(
  events: InputAdapterEvent[],
  append: (event: Extract<InputAdapterEvent, { type: 'condition-changed' }>) => void,
): InputAdapterEventHandler {
  return (event) => {
    events.push(event)
    if (event.type === 'condition-changed') append(event)
    return Promise.resolve()
  }
}

describe('createScreenButtonInputAdapter', () => {
  it('converts screen actions to shared mouse-source events through real clicks', async () => {
    const events: InputAdapterEvent[] = []
    const adapter = createScreenButtonInputAdapter({
      dispatcher: createRecordingDispatcher(events),
      isControlAllowed: () => true,
    })
    const wrapper = mount(createClickHost(adapter), { attachTo: document.body })

    wrapper.get<HTMLButtonElement>('[data-testid="control"]').element.click()
    wrapper.get<HTMLButtonElement>('[data-testid="query"]').element.click()
    await wrapper.get('textarea').setValue('  screen answer\n')
    wrapper.get<HTMLButtonElement>('[data-testid="answer"]').element.click()

    expect(events).toEqual([
      { type: 'condition-changed', source: 'mouse', control: 'up', count: 1 },
      { type: 'query-submitted', source: 'mouse' },
      { type: 'answer-submitted', source: 'mouse', answer: '  screen answer\n' },
    ])

    wrapper.unmount()
  })

  it('rejects a control that is not currently allowed', () => {
    const events: InputAdapterEvent[] = []
    const isControlAllowed = vi.fn<(control: string) => boolean>((control) => control !== 'left')
    const adapter = createScreenButtonInputAdapter({
      dispatcher: createRecordingDispatcher(events),
      isControlAllowed,
    })

    expect(adapter.pressControl('left')).toBe(false)
    expect(adapter.pressControl('right')).toBe(true)
    expect(isControlAllowed).toHaveBeenCalledTimes(2)
    expect(events).toEqual([
      { type: 'condition-changed', source: 'mouse', control: 'right', count: 1 },
    ])
  })

  it.each([
    { disabled: true, busy: false },
    { disabled: false, busy: true },
  ])('guards forced clicks while unavailable: %o', async ({ disabled, busy }) => {
    const events: InputAdapterEvent[] = []
    const dispatcher = createGuardedInputDispatcher(
      (event) => {
        events.push(event)
        return Promise.resolve()
      },
      {
        isDisabled: () => disabled,
        isBusy: () => busy,
        onError: vi.fn<(error: unknown) => void>(),
      },
    )
    const adapter = createScreenButtonInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
    })
    const wrapper = mount(createClickHost(adapter, () => disabled || dispatcher.busy))

    await wrapper.get('textarea').setValue('blocked answer')
    for (const selector of [
      '[data-testid="control"]',
      '[data-testid="query"]',
      '[data-testid="answer"]',
    ]) {
      const button = wrapper.get<HTMLButtonElement>(selector)
      expect(button.attributes('disabled')).toBeDefined()
      button.element.dispatchEvent(new MouseEvent('click', { bubbles: true }))
    }

    expect(events).toEqual([])
  })

  it('shares one submission latch across real screen clicks and keyboard Enter', async () => {
    const events: InputAdapterEvent[] = []
    let resolveSubmission: (() => void) | undefined
    const pendingSubmission = new Promise<void>((resolve) => {
      resolveSubmission = resolve
    })
    const dispatcher = createGuardedInputDispatcher(
      (event) => {
        events.push(event)
        return pendingSubmission
      },
      { onError: vi.fn<(error: unknown) => void>() },
    )
    const target = new EventTarget()
    const keyboard = createKeyboardInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })
    const screen = createScreenButtonInputAdapter({
      dispatcher,
      isControlAllowed: () => true,
    })
    const wrapper = mount(createClickHost(screen), { attachTo: document.body })
    const queryButton = wrapper.get<HTMLButtonElement>('[data-testid="query"]')
    const answerButton = wrapper.get<HTMLButtonElement>('[data-testid="answer"]')

    await wrapper.get('textarea').setValue('cross-source answer')
    keyboard.start()

    queryButton.element.click()
    queryButton.element.click()
    target.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', cancelable: true }))
    answerButton.element.click()

    expect(events).toEqual([{ type: 'query-submitted', source: 'mouse' }])

    if (resolveSubmission === undefined) throw new Error('submission resolver was not created')
    resolveSubmission()
    await pendingSubmission
    await Promise.resolve()

    answerButton.element.click()

    expect(events).toEqual([
      { type: 'query-submitted', source: 'mouse' },
      { type: 'answer-submitted', source: 'mouse', answer: 'cross-source answer' },
    ])

    keyboard.stop()
    wrapper.unmount()
  })

  it('produces the same operation sequence as the keyboard adapter', () => {
    const keyboardEvents: InputAdapterEvent[] = []
    const screenEvents: InputAdapterEvent[] = []
    const keyboardBuffer = createOperationBuffer()
    const screenBuffer = createOperationBuffer()
    const target = new EventTarget()
    const keyboard = createKeyboardInputAdapter({
      dispatcher: createGuardedInputDispatcher(
        createBufferingHandler(keyboardEvents, (event) => keyboardBuffer.append(event)),
        { onError: vi.fn<(error: unknown) => void>() },
      ),
      isControlAllowed: () => true,
      target,
      getActiveElement: () => null,
    })
    const screen = createScreenButtonInputAdapter({
      dispatcher: createGuardedInputDispatcher(
        createBufferingHandler(screenEvents, (event) => screenBuffer.append(event)),
        { onError: vi.fn<(error: unknown) => void>() },
      ),
      isControlAllowed: () => true,
    })

    keyboard.start()
    for (const key of ['ArrowUp', 'ArrowUp', 'ArrowRight', 'ArrowDown']) {
      target.dispatchEvent(new KeyboardEvent('keydown', { key, cancelable: true }))
    }
    keyboard.stop()

    screen.pressControl('up')
    screen.pressControl('up')
    screen.pressControl('right')
    screen.pressControl('down')

    expect(keyboardBuffer.snapshot()).toEqual([
      { control: 'up', count: 2 },
      { control: 'right', count: 1 },
      { control: 'down', count: 1 },
    ])
    expect(screenBuffer.snapshot()).toEqual(keyboardBuffer.snapshot())
    expect(keyboardEvents.every((event) => event.source === 'keyboard')).toBe(true)
    expect(screenEvents.every((event) => event.source === 'mouse')).toBe(true)
  })

  it('keeps answer text out of the operation buffer', () => {
    const events: InputAdapterEvent[] = []
    const buffer = createOperationBuffer()
    const adapter = createScreenButtonInputAdapter({
      dispatcher: createGuardedInputDispatcher(
        createBufferingHandler(events, (event) => buffer.append(event)),
        { onError: vi.fn<(error: unknown) => void>() },
      ),
      isControlAllowed: () => true,
    })

    adapter.pressControl('down')
    adapter.submitAnswer('answer text is not an operation')

    expect(events).toEqual([
      { type: 'condition-changed', source: 'mouse', control: 'down', count: 1 },
      {
        type: 'answer-submitted',
        source: 'mouse',
        answer: 'answer text is not an operation',
      },
    ])
    expect(buffer.snapshot()).toEqual([{ control: 'down', count: 1 }])
  })
})
