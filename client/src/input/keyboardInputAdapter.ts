import {
  keyboardInputSource,
  type Control,
  type InputAdapterDispatcher,
} from './InputAdapter.types'
import { isEditableInputTarget } from './inputGuard'

export const keyboardControlByKey: Readonly<Record<string, Control>> = {
  ArrowUp: 'up',
  ArrowDown: 'down',
  ArrowLeft: 'left',
  ArrowRight: 'right',
}

export interface KeyboardInputAdapterOptions {
  dispatcher: InputAdapterDispatcher
  isControlAllowed: (control: Control) => boolean
  target?: EventTarget
  getActiveElement?: () => EventTarget | null
}

export interface KeyboardInputAdapter {
  start(): void
  stop(): void
  submitAnswer(answer: string): boolean
}

function hasModifier(event: KeyboardEvent): boolean {
  return event.altKey || event.ctrlKey || event.metaKey || event.shiftKey
}

function getDefaultTarget(): EventTarget {
  return window
}

function getDefaultActiveElement(): EventTarget | null {
  return document.activeElement
}

function isActionTarget(target: EventTarget | null): boolean {
  if (!(target instanceof Element)) return false

  return target.closest('button, a[href], summary, [role="button"], [role="link"]') !== null
}

function shouldIgnoreShortcut(target: EventTarget | null): boolean {
  return isEditableInputTarget(target) || isActionTarget(target)
}

/** Converts keyboard shortcuts into the same semantic events as the other input adapters. */
export function createKeyboardInputAdapter(
  options: KeyboardInputAdapterOptions,
): KeyboardInputAdapter {
  const target = options.target ?? getDefaultTarget()
  const getActiveElement = options.getActiveElement ?? getDefaultActiveElement
  let started = false

  const handleKeydown: EventListener = (event) => {
    if (!(event instanceof KeyboardEvent)) return
    if (event.isComposing || hasModifier(event)) return
    if (shouldIgnoreShortcut(event.target) || shouldIgnoreShortcut(getActiveElement())) {
      return
    }

    const control = keyboardControlByKey[event.key]
    if (control !== undefined) {
      if (!options.isControlAllowed(control)) return

      event.preventDefault()
      if (event.repeat) return

      options.dispatcher.dispatch({
        type: 'condition-changed',
        source: keyboardInputSource,
        control,
        count: 1,
      })
      return
    }

    if (event.key !== 'Enter') return

    event.preventDefault()
    if (event.repeat) return

    options.dispatcher.dispatch({ type: 'query-submitted', source: keyboardInputSource })
  }

  return {
    start() {
      if (started) return

      target.addEventListener('keydown', handleKeydown)
      started = true
    },

    stop() {
      if (!started) return

      target.removeEventListener('keydown', handleKeydown)
      started = false
    },

    submitAnswer(answer) {
      return options.dispatcher.dispatch({
        type: 'answer-submitted',
        source: keyboardInputSource,
        answer,
      })
    },
  }
}
