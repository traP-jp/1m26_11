import { serialInputSource, type Control, type InputAdapterEvent } from './InputAdapter.types'
import { SerialFrameParser, type SerialProtocolV1Gesture } from './serialFrameParser'

type ConditionChangedInputEvent = Extract<InputAdapterEvent, { type: 'condition-changed' }>

export type SerialInputAdapterEvent = Omit<
  ConditionChangedInputEvent,
  'source' | 'count' | 'gesture'
> & {
  source: typeof serialInputSource
  count: 1
  gesture: SerialProtocolV1Gesture
}

export interface WebSerialInputAdapterOptions {
  dispatcher: {
    dispatch(event: InputAdapterEvent): boolean
  }
  isControlAllowed: (control: Control) => boolean
}

export interface WebSerialInputAdapter {
  pushChunk(chunk: Uint8Array): void
  resetSession(): void
}

/** Converts already debounced Wire v1 frames; port lifecycle remains the caller's responsibility. */
export function createWebSerialInputAdapter(
  options: WebSerialInputAdapterOptions,
): WebSerialInputAdapter {
  const parser = new SerialFrameParser()

  return {
    pushChunk(chunk) {
      for (const frame of parser.pushChunk(chunk)) {
        if (!options.isControlAllowed(frame.control)) continue

        const event: SerialInputAdapterEvent = {
          type: 'condition-changed',
          source: serialInputSource,
          control: frame.control,
          count: 1,
          gesture: frame.gesture,
        }
        options.dispatcher.dispatch(event)
      }
    },

    resetSession() {
      parser.resetSession()
    },
  }
}
