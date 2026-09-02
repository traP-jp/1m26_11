import {
  SerialFrameParser,
  type SerialProtocolV1Control,
  type SerialProtocolV1Frame,
} from './serialFrameParser'

export interface WebSerialInputAdapterOptions {
  onFrame: (frame: SerialProtocolV1Frame) => void
  isControlAllowed: (control: SerialProtocolV1Control) => boolean
}

export interface WebSerialInputAdapter {
  pushChunk(chunk: Uint8Array): void
  resetSession(): void
}

/**
 * Parses received chunks and forwards valid Wire v1 frames without deciding how gestures map to
 * common input events. Port lifecycle remains the caller's responsibility.
 */
export function createWebSerialInputAdapter(
  options: WebSerialInputAdapterOptions,
): WebSerialInputAdapter {
  const parser = new SerialFrameParser()

  return {
    pushChunk(chunk) {
      for (const frame of parser.pushChunk(chunk)) {
        if (!options.isControlAllowed(frame.control)) continue
        options.onFrame(frame)
      }
    },

    resetSession() {
      parser.resetSession()
    },
  }
}
