export const SERIAL_PROTOCOL_V1_CONTROLS = [
  'up',
  'down',
  'left',
  'right',
  'red',
  'yellow',
  'green',
] as const

export const SERIAL_PROTOCOL_V1_GESTURES = ['short_press', 'long_press'] as const

export const SERIAL_PROTOCOL_V1_MAX_FRAME_BYTES = 256

export type SerialProtocolV1Control = (typeof SERIAL_PROTOCOL_V1_CONTROLS)[number]
export type SerialProtocolV1Gesture = (typeof SERIAL_PROTOCOL_V1_GESTURES)[number]

export interface SerialProtocolV1Frame {
  v: 1
  control: SerialProtocolV1Control
  gesture: SerialProtocolV1Gesture
}

const LF = 0x0a
const CR = 0x0d
const FRAME_KEYS = new Set(['v', 'control', 'gesture'])
const CONTROLS = new Set<string>(SERIAL_PROTOCOL_V1_CONTROLS)
const GESTURES = new Set<string>(SERIAL_PROTOCOL_V1_GESTURES)
// The complete document is parsed as JSON before this matcher runs, so a
// permissive escaped-character token is enough to count original member names.
const JSON_MEMBER_NAME = /("(?:\\.|[^"\\])*")\s*:/g

function hasEachFrameKeyExactlyOnce(decoded: string): boolean {
  const seen = new Set<string>()

  for (const match of decoded.matchAll(JSON_MEMBER_NAME)) {
    const encodedKey = match[1]
    if (!encodedKey) continue

    const key = JSON.parse(encodedKey) as string
    if (!FRAME_KEYS.has(key)) continue
    if (seen.has(key)) return false
    seen.add(key)
  }

  return seen.size === FRAME_KEYS.size
}

function parseFrame(frameBytes: readonly number[]): SerialProtocolV1Frame | undefined {
  if (frameBytes.some((byte) => byte > 0x7f)) return undefined

  let decoded: string

  try {
    decoded = new TextDecoder('utf-8', { fatal: true }).decode(new Uint8Array(frameBytes))
  } catch {
    return undefined
  }

  let value: unknown
  try {
    value = JSON.parse(decoded)
  } catch {
    return undefined
  }

  if (typeof value !== 'object' || value === null || Array.isArray(value)) return undefined

  const record = value as Record<string, unknown>
  const keys = Object.keys(record)
  if (keys.length !== FRAME_KEYS.size || keys.some((key) => !FRAME_KEYS.has(key))) {
    return undefined
  }
  // JSON.parse keeps only the last value of a duplicate member, so inspect the
  // original member-name tokens before accepting the exact three-field schema.
  if (!hasEachFrameKeyExactlyOnce(decoded)) return undefined
  if (record.v !== 1) return undefined
  if (typeof record.control !== 'string' || !CONTROLS.has(record.control)) return undefined
  if (typeof record.gesture !== 'string' || !GESTURES.has(record.gesture)) return undefined

  return {
    v: 1,
    control: record.control as SerialProtocolV1Control,
    gesture: record.gesture as SerialProtocolV1Gesture,
  }
}

export class SerialFrameParser {
  private frameBytes: number[] = []
  private discardingFrame = false

  pushChunk(chunk: Uint8Array): SerialProtocolV1Frame[] {
    const frames: SerialProtocolV1Frame[] = []

    for (const byte of chunk) {
      if (byte === LF) {
        if (!this.discardingFrame) {
          const lastByte = this.frameBytes[this.frameBytes.length - 1]
          const frame = lastByte === CR ? this.frameBytes.slice(0, -1) : this.frameBytes
          const parsedFrame = parseFrame(frame)
          if (parsedFrame) frames.push(parsedFrame)
        }

        this.resetFrame()
        continue
      }

      if (this.discardingFrame) continue

      if (this.frameBytes.length < SERIAL_PROTOCOL_V1_MAX_FRAME_BYTES) {
        this.frameBytes.push(byte)
        continue
      }

      // One extra CR is retained only because it may be the optional CR in CRLF.
      if (this.frameBytes.length === SERIAL_PROTOCOL_V1_MAX_FRAME_BYTES && byte === CR) {
        this.frameBytes.push(byte)
        continue
      }

      this.frameBytes = []
      this.discardingFrame = true
    }

    return frames
  }

  resetSession(): void {
    this.resetFrame()
  }

  private resetFrame(): void {
    this.frameBytes = []
    this.discardingFrame = false
  }
}
