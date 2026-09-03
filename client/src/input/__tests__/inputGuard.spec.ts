import { describe, expect, it, vi } from 'vitest'

import type { InputAdapterEvent, InputAdapterEventHandler } from '../InputAdapter.types'
import { createGuardedInputDispatcher, isEditableInputTarget } from '../inputGuard'

function createDeferred(): {
  promise: Promise<void>
  resolve: () => void
  reject: (error: unknown) => void
} {
  let resolve!: () => void
  let reject!: (error: unknown) => void
  const promise = new Promise<void>((resolvePromise, rejectPromise) => {
    resolve = resolvePromise
    reject = rejectPromise
  })

  return { promise, resolve, reject }
}

const conditionChangedEvent = {
  type: 'condition-changed',
  source: 'keyboard',
  control: 'up',
  count: 1,
} satisfies InputAdapterEvent

const querySubmittedEvent = {
  type: 'query-submitted',
  source: 'keyboard',
} satisfies InputAdapterEvent

const answerSubmittedEvent = {
  type: 'answer-submitted',
  source: 'mouse',
  answer: 'answer',
} satisfies InputAdapterEvent

describe('createGuardedInputDispatcher', () => {
  it('rejects every event while disabled and responds to later state changes', () => {
    let disabled = true
    const handleEvent = vi.fn<InputAdapterEventHandler>(() => Promise.resolve())
    const onError = vi.fn<(error: unknown) => void>()
    const dispatcher = createGuardedInputDispatcher(handleEvent, {
      isDisabled: () => disabled,
      onError,
    })

    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(false)
    expect(dispatcher.dispatch(querySubmittedEvent)).toBe(false)
    expect(dispatcher.busy).toBe(false)
    expect(handleEvent).not.toHaveBeenCalled()

    disabled = false

    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(true)
    expect(handleEvent).toHaveBeenCalledExactlyOnceWith(conditionChangedEvent)
    expect(onError).not.toHaveBeenCalled()
  })

  it('rejects every event while externally busy and exposes the combined busy state', () => {
    let externallyBusy = true
    const handleEvent = vi.fn<InputAdapterEventHandler>(() => Promise.resolve())
    const dispatcher = createGuardedInputDispatcher(handleEvent, {
      isBusy: () => externallyBusy,
      onError: vi.fn<(error: unknown) => void>(),
    })

    expect(dispatcher.busy).toBe(true)
    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(false)
    expect(dispatcher.dispatch(answerSubmittedEvent)).toBe(false)
    expect(handleEvent).not.toHaveBeenCalled()

    externallyBusy = false

    expect(dispatcher.busy).toBe(false)
    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(true)
    expect(handleEvent).toHaveBeenCalledExactlyOnceWith(conditionChangedEvent)
  })

  it('uses one synchronous latch for query and answer submissions from either source', () => {
    const deferred = createDeferred()
    const handleEvent = vi.fn<InputAdapterEventHandler>(() => deferred.promise)
    const dispatcher = createGuardedInputDispatcher(handleEvent, {
      onError: vi.fn<(error: unknown) => void>(),
    })

    expect(dispatcher.dispatch(querySubmittedEvent)).toBe(true)
    expect(dispatcher.busy).toBe(true)
    expect(dispatcher.dispatch(answerSubmittedEvent)).toBe(false)
    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(false)
    expect(handleEvent).toHaveBeenCalledExactlyOnceWith(querySubmittedEvent)
  })

  it('releases the submission latch after asynchronous success', async () => {
    const firstSubmission = createDeferred()
    const handleEvent = vi
      .fn<InputAdapterEventHandler>()
      .mockReturnValueOnce(firstSubmission.promise)
      .mockResolvedValue(undefined)
    const dispatcher = createGuardedInputDispatcher(handleEvent, {
      onError: vi.fn<(error: unknown) => void>(),
    })

    expect(dispatcher.dispatch(querySubmittedEvent)).toBe(true)
    expect(dispatcher.dispatch(answerSubmittedEvent)).toBe(false)

    firstSubmission.resolve()
    await firstSubmission.promise
    await Promise.resolve()

    expect(dispatcher.busy).toBe(false)
    expect(dispatcher.dispatch(answerSubmittedEvent)).toBe(true)
    expect(handleEvent).toHaveBeenNthCalledWith(2, answerSubmittedEvent)
  })

  it('reports asynchronous failure and releases the submission latch', async () => {
    const firstSubmission = createDeferred()
    const error = new Error('submission failed')
    const onError = vi.fn<(error: unknown) => void>()
    const handleEvent = vi
      .fn<InputAdapterEventHandler>()
      .mockReturnValueOnce(firstSubmission.promise)
      .mockResolvedValue(undefined)
    const dispatcher = createGuardedInputDispatcher(handleEvent, { onError })

    expect(dispatcher.dispatch(answerSubmittedEvent)).toBe(true)
    firstSubmission.reject(error)
    await expect(firstSubmission.promise).rejects.toBe(error)
    await vi.waitFor(() => {
      expect(onError).toHaveBeenCalledExactlyOnceWith(error)
    })

    expect(dispatcher.busy).toBe(false)
    expect(dispatcher.dispatch(querySubmittedEvent)).toBe(true)
    expect(handleEvent).toHaveBeenNthCalledWith(2, querySubmittedEvent)
  })

  it('reports a synchronous submission error and immediately releases the latch', () => {
    const error = new Error('synchronous failure')
    const onError = vi.fn<(error: unknown) => void>()
    const handleEvent = vi
      .fn<InputAdapterEventHandler>()
      .mockImplementationOnce(() => {
        throw error
      })
      .mockResolvedValue(undefined)
    const dispatcher = createGuardedInputDispatcher(handleEvent, { onError })

    expect(dispatcher.dispatch(querySubmittedEvent)).toBe(true)
    expect(onError).toHaveBeenCalledExactlyOnceWith(error)
    expect(dispatcher.busy).toBe(false)
    expect(dispatcher.dispatch(answerSubmittedEvent)).toBe(true)
    expect(handleEvent).toHaveBeenNthCalledWith(2, answerSubmittedEvent)
  })

  it('does not latch non-submission events and reports their asynchronous failures', async () => {
    const conditionResult = createDeferred()
    const error = new Error('condition handler failed')
    const onError = vi.fn<(error: unknown) => void>()
    const handleEvent = vi
      .fn<InputAdapterEventHandler>()
      .mockReturnValueOnce(conditionResult.promise)
      .mockResolvedValue(undefined)
    const dispatcher = createGuardedInputDispatcher(handleEvent, { onError })

    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(true)
    expect(dispatcher.busy).toBe(false)
    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(true)
    expect(handleEvent).toHaveBeenCalledTimes(2)

    conditionResult.reject(error)
    await expect(conditionResult.promise).rejects.toBe(error)
    await vi.waitFor(() => {
      expect(onError).toHaveBeenCalledExactlyOnceWith(error)
    })
  })

  it('reports a synchronous non-submission error without latching later input', () => {
    const error = new Error('condition handler threw')
    const onError = vi.fn<(error: unknown) => void>()
    const handleEvent = vi
      .fn<InputAdapterEventHandler>()
      .mockImplementationOnce(() => {
        throw error
      })
      .mockResolvedValue(undefined)
    const dispatcher = createGuardedInputDispatcher(handleEvent, { onError })

    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(true)
    expect(onError).toHaveBeenCalledExactlyOnceWith(error)
    expect(dispatcher.busy).toBe(false)
    expect(dispatcher.dispatch(conditionChangedEvent)).toBe(true)
    expect(handleEvent).toHaveBeenCalledTimes(2)
  })
})

describe('isEditableInputTarget', () => {
  it.each(['input', 'textarea', 'select'])(
    'recognizes a native %s element and its descendants as editable',
    (tagName) => {
      const element = document.createElement(tagName)
      const descendant = document.createElement('span')
      element.append(descendant)

      expect(isEditableInputTarget(element)).toBe(true)
      expect(isEditableInputTarget(descendant)).toBe(true)
    },
  )

  it.each(['', 'true', 'plaintext-only'])(
    'recognizes a descendant of contenteditable=%j as editable',
    (contentEditable) => {
      const editor = document.createElement('div')
      const child = document.createElement('span')
      const descendant = document.createElement('strong')
      editor.setAttribute('contenteditable', contentEditable)
      editor.append(child)
      child.append(descendant)

      expect(isEditableInputTarget(editor)).toBe(true)
      expect(isEditableInputTarget(descendant)).toBe(true)
    },
  )

  it('does not treat ordinary, explicitly non-editable, null, or non-element targets as editable', () => {
    const ordinaryElement = document.createElement('button')
    const nonEditableElement = document.createElement('div')
    nonEditableElement.setAttribute('contenteditable', 'false')

    const uppercaseNonEditableElement = document.createElement('div')
    uppercaseNonEditableElement.setAttribute('contenteditable', 'FALSE')

    expect(isEditableInputTarget(ordinaryElement)).toBe(false)
    expect(isEditableInputTarget(nonEditableElement)).toBe(false)
    expect(isEditableInputTarget(uppercaseNonEditableElement)).toBe(false)
    expect(isEditableInputTarget(null)).toBe(false)
    expect(isEditableInputTarget(new EventTarget())).toBe(false)
  })

  it('stops contenteditable inheritance at an explicitly non-editable island', () => {
    const editor = document.createElement('div')
    const nonEditableIsland = document.createElement('div')
    const descendant = document.createElement('span')
    editor.setAttribute('contenteditable', 'true')
    nonEditableIsland.setAttribute('contenteditable', 'false')
    editor.append(nonEditableIsland)
    nonEditableIsland.append(descendant)

    expect(isEditableInputTarget(editor)).toBe(true)
    expect(isEditableInputTarget(nonEditableIsland)).toBe(false)
    expect(isEditableInputTarget(descendant)).toBe(false)
  })
})
