import { serialInputSource, type Control, type InputAdapterDispatcher } from './InputAdapter.types'
import { SerialFrameParser } from './serialFrameParser'

export interface WebSerialInputAdapterOptions {
  dispatcher: InputAdapterDispatcher
  isControlAllowed: (control: Control) => boolean
}

export interface WebSerialInputAdapter {
  pushChunk(chunk: Uint8Array): void
  resetSession(): void
}

/** Converts valid, firmware-debounced Wire v1 frames into shared input events. */
export function createWebSerialInputAdapter(
  options: WebSerialInputAdapterOptions,
): WebSerialInputAdapter {
  const parser = new SerialFrameParser()

  return {
    pushChunk(chunk) {
      for (const frame of parser.pushChunk(chunk)) {
        if (!options.isControlAllowed(frame.control)) continue

        options.dispatcher.dispatch({
          type: 'condition-changed',
          source: serialInputSource,
          control: frame.control,
          count: 1,
        })
      }
    },

    resetSession() {
      parser.resetSession()
    },
  }
}
