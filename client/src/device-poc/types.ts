export const SERIAL_POC_OPEN_OPTIONS = {
  baudRate: 115_200,
  dataBits: 8,
  stopBits: 1,
  parity: 'none',
  flowControl: 'none',
} as const

export const SERIAL_POC_PORT_FILTER = {
  usbVendorId: 0x2e8a,
  usbProductId: 0x0005,
} as const

export const SERIAL_POC_CAPTURE_LIMIT_BYTES = 1024 * 1024

export type SerialPocOperation =
  | 'request-port'
  | 'open-port'
  | 'raw-repl-sync'
  | 'launch-script'
  | 'read'
  | 'stop-script'
  | 'close-port'
  | 'export-capture'

export type SerialPocState =
  | { phase: 'unsupported'; message: string }
  | { phase: 'idle'; message: string }
  | { phase: 'selecting'; message: string }
  | { phase: 'opening'; message: string }
  | { phase: 'syncing-raw-repl'; message: string }
  | { phase: 'launching'; message: string }
  | { phase: 'running'; message: string }
  | { phase: 'stopping'; message: string }
  | { phase: 'disconnected'; message: string; incomplete: boolean }
  | { phase: 'error'; operation: SerialPocOperation; message: string }

export interface RawSerialChunk {
  sequence: number
  connectionId: number
  offset: number
  receivedElapsedMs: number
  bytes: Uint8Array
}

export type SerialConnectionEndReason =
  'user' | 'capture-limit' | 'stream-ended' | 'read-error' | 'device-disconnected' | 'setup-error'

export interface SerialConnectionRecord {
  id: number
  startedAt: string
  startedOffset: number
  usbVendorId?: number
  usbProductId?: number
  rawReplReadyObservedOffset?: number
  buttonTestLaunchRequestedOffset?: number
  buttonTestActiveObservedOffset?: number
  stopRequestedOffset?: number
  stopCompletedObservedOffset?: number
  endedAt?: string
  endedOffset?: number
  endReason?: SerialConnectionEndReason
  stopConfirmed?: boolean
}
