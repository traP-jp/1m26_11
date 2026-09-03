import {
  screenButtonInputSource,
  type Control,
  type InputAdapterDispatcher,
} from './InputAdapter.types'

export interface ScreenButtonInputAdapterOptions {
  dispatcher: InputAdapterDispatcher
  isControlAllowed: (control: Control) => boolean
}

export interface ScreenButtonInputAdapter {
  pressControl(control: Control): boolean
  submitQuery(): boolean
  submitAnswer(answer: string): boolean
}

/** Converts screen-button actions into the shared input event contract. */
export function createScreenButtonInputAdapter(
  options: ScreenButtonInputAdapterOptions,
): ScreenButtonInputAdapter {
  const { dispatcher, isControlAllowed } = options

  return {
    pressControl(control) {
      if (!isControlAllowed(control)) return false

      return dispatcher.dispatch({
        type: 'condition-changed',
        source: screenButtonInputSource,
        control,
        count: 1,
      })
    },

    submitQuery() {
      return dispatcher.dispatch({
        type: 'query-submitted',
        source: screenButtonInputSource,
      })
    },

    submitAnswer(answer) {
      return dispatcher.dispatch({
        type: 'answer-submitted',
        source: screenButtonInputSource,
        answer,
      })
    },
  }
}
