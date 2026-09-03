import type {
  InputAdapterDispatcher,
  InputAdapterEvent,
  InputAdapterEventHandler,
} from './InputAdapter.types'

export interface InputGuardOptions {
  isDisabled?: () => boolean
  isBusy?: () => boolean
  onError: (error: unknown) => void
}

function isSubmissionEvent(event: InputAdapterEvent): boolean {
  return event.type === 'query-submitted' || event.type === 'answer-submitted'
}

/**
 * Applies one shared synchronous latch to keyboard and screen submissions.
 * The latch remains held until an asynchronous handler settles.
 */
export function createGuardedInputDispatcher(
  handleEvent: InputAdapterEventHandler,
  options: InputGuardOptions,
): InputAdapterDispatcher {
  const isDisabled = options.isDisabled ?? (() => false)
  const isExternallyBusy = options.isBusy ?? (() => false)
  let submitting = false

  function reportAsyncResult(result: Promise<void>): void {
    void result.catch(options.onError)
  }

  return {
    get busy() {
      return submitting || isExternallyBusy()
    },

    dispatch(event) {
      if (isDisabled() || submitting || isExternallyBusy()) return false

      if (!isSubmissionEvent(event)) {
        try {
          reportAsyncResult(handleEvent(event))
        } catch (error) {
          options.onError(error)
        }
        return true
      }

      submitting = true
      let result: Promise<void>
      try {
        result = handleEvent(event)
      } catch (error) {
        submitting = false
        options.onError(error)
        return true
      }

      void Promise.resolve(result).then(
        () => {
          submitting = false
        },
        (error: unknown) => {
          submitting = false
          options.onError(error)
        },
      )
      return true
    },
  }
}

/** Returns true for native and contenteditable text-entry targets, including descendants. */
export function isEditableInputTarget(target: EventTarget | null): boolean {
  let element = target instanceof Element ? target : null

  while (element !== null) {
    if (
      element instanceof HTMLInputElement ||
      element instanceof HTMLTextAreaElement ||
      element instanceof HTMLSelectElement
    ) {
      return true
    }

    if (element instanceof HTMLElement) {
      const contentEditable = element.getAttribute('contenteditable')
      if (contentEditable !== null) {
        const normalizedValue = contentEditable.trim().toLowerCase()
        if (normalizedValue === 'false') return false
        if (
          normalizedValue === '' ||
          normalizedValue === 'true' ||
          normalizedValue === 'plaintext-only'
        ) {
          return true
        }
      }
    }

    element = element.parentElement
  }

  return false
}
