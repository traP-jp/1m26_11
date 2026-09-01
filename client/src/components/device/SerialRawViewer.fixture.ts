import type { RawSerialChunk, SerialPocState } from '../../device-poc/types'

const encoder = new TextEncoder()
const initialChunk = encoder.encode('{"v":1,"control":"up",')
const pressedChunk = encoder.encode('"gesture":"short_press"}\r\n')
const runningChunks: RawSerialChunk[] = [
  {
    sequence: 1,
    connectionId: 1,
    offset: 0,
    receivedElapsedMs: 12.4,
    bytes: initialChunk,
  },
  {
    sequence: 2,
    connectionId: 1,
    offset: initialChunk.length,
    receivedElapsedMs: 913.8,
    bytes: pressedChunk,
  },
]

export const serialRawViewerFixtures = {
  unsupported: {
    state: {
      phase: 'unsupported',
      message: 'このブラウザはWeb Serial APIに対応していません。',
    } satisfies SerialPocState,
    chunks: [] as RawSerialChunk[],
    decodedPreview: '',
    totalBytes: 0,
    captureLimitBytes: 1024 * 1024,
    canConnect: false,
    canStop: false,
    canClear: false,
    canDownload: false,
  },
  idle: {
    state: { phase: 'idle', message: 'Picoは未接続です。' } satisfies SerialPocState,
    chunks: [] as RawSerialChunk[],
    decodedPreview: '',
    totalBytes: 0,
    captureLimitBytes: 1024 * 1024,
    canConnect: true,
    canStop: false,
    canClear: false,
    canDownload: false,
  },
  selecting: {
    state: {
      phase: 'selecting',
      message: 'Picoを選択してください。',
    } satisfies SerialPocState,
    chunks: [] as RawSerialChunk[],
    decodedPreview: '',
    totalBytes: 0,
    captureLimitBytes: 1024 * 1024,
    canConnect: false,
    canStop: false,
    canClear: false,
    canDownload: false,
  },
  running: {
    state: {
      phase: 'running',
      message: 'raw byteをcapture中です。物理スイッチを操作できます。',
    } satisfies SerialPocState,
    chunks: runningChunks,
    decodedPreview: new TextDecoder().decode(initialChunk) + new TextDecoder().decode(pressedChunk),
    totalBytes: initialChunk.length + pressedChunk.length,
    captureLimitBytes: 1024 * 1024,
    canConnect: false,
    canStop: true,
    canClear: false,
    canDownload: false,
  },
  stopping: {
    state: {
      phase: 'stopping',
      message: '/serial_protocol_poc.pyを停止しています。',
    } satisfies SerialPocState,
    chunks: runningChunks,
    decodedPreview: new TextDecoder().decode(initialChunk) + new TextDecoder().decode(pressedChunk),
    totalBytes: initialChunk.length + pressedChunk.length,
    captureLimitBytes: 1024 * 1024,
    canConnect: false,
    canStop: false,
    canClear: false,
    canDownload: false,
  },
  disconnected: {
    state: {
      phase: 'disconnected',
      message: 'USB deviceが切断されました。受信済みdataは保持されています。',
      incomplete: true,
    } satisfies SerialPocState,
    chunks: runningChunks,
    decodedPreview: new TextDecoder().decode(initialChunk) + new TextDecoder().decode(pressedChunk),
    totalBytes: initialChunk.length + pressedChunk.length,
    captureLimitBytes: 1024 * 1024,
    canConnect: true,
    canStop: false,
    canClear: true,
    canDownload: true,
  },
  error: {
    state: {
      phase: 'error',
      operation: 'open-port',
      message: 'ポートを開けませんでした。MicroPicoの接続を確認してください。',
    } satisfies SerialPocState,
    chunks: [] as RawSerialChunk[],
    decodedPreview: '',
    totalBytes: 0,
    captureLimitBytes: 1024 * 1024,
    canConnect: true,
    canStop: false,
    canClear: false,
    canDownload: false,
  },
} as const
