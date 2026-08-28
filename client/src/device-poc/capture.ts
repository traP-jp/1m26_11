import type { RawSerialChunk, SerialConnectionRecord } from './types'
import { SERIAL_POC_OPEN_OPTIONS } from './types'

export interface RawCaptureArtifacts {
  raw: Blob
  metadata: Blob
  sha256: string
  baseName: string
}

interface CaptureArtifactInput {
  chunks: readonly RawSerialChunk[]
  connections: readonly SerialConnectionRecord[]
  capturedAt: string
  origin: string
  userAgent: string
  secureContext: boolean
  captureLimitBytes: number
}

export function concatenateRawChunks(chunks: readonly RawSerialChunk[]): Uint8Array<ArrayBuffer> {
  const totalBytes = chunks.reduce((total, chunk) => total + chunk.bytes.length, 0)
  const raw = new Uint8Array(totalBytes)
  let offset = 0

  for (const chunk of chunks) {
    raw.set(chunk.bytes, offset)
    offset += chunk.bytes.length
  }

  return raw
}

export async function createRawCaptureArtifacts(
  input: CaptureArtifactInput,
): Promise<RawCaptureArtifacts> {
  const rawBytes = concatenateRawChunks(input.chunks)
  const digest = await globalThis.crypto.subtle.digest('SHA-256', rawBytes)
  const sha256 = [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
  const timestamp = input.capturedAt.replace(/:/g, '-').replace(/\./g, '-')
  const baseName = `web-serial-raw-${timestamp}`

  const metadata = {
    captureSchema: 'web-serial-raw-capture/v1',
    source: 'web-serial-hardware',
    capturedAt: input.capturedAt,
    environment: {
      origin: input.origin,
      secureContext: input.secureContext,
      userAgent: input.userAgent,
    },
    device: {
      expected: {
        board: 'Raspberry Pi Pico H',
        mcu: 'RP2040',
        firmware: 'MicroPython v1.29.0',
      },
      verification:
        'operator confirmation required; the viewer does not identify board or firmware',
    },
    openOptions: SERIAL_POC_OPEN_OPTIONS,
    captureStopThresholdBytes: input.captureLimitBytes,
    captureMayExceedStopThreshold: true,
    totalChunks: input.chunks.length,
    totalBytes: rawBytes.length,
    rawSha256: sha256,
    connections: input.connections,
    chunks: input.chunks.map((chunk) => ({
      sequence: chunk.sequence,
      connectionId: chunk.connectionId,
      offset: chunk.offset,
      length: chunk.bytes.length,
      receivedElapsedMs: chunk.receivedElapsedMs,
    })),
    notes: [
      'capture.bin is authoritative; decoded text is intentionally not exported.',
      'capture.bin contains received raw REPL bootstrap responses as well as script output.',
      'The metadata connection offsets identify the observed bootstrap and script phases.',
      'Read chunk boundaries are transport observations, not protocol frame boundaries.',
      'The raw REPL bootstrap is PoC transport control, not the production serial protocol.',
    ],
  }

  return {
    raw: new Blob([rawBytes], { type: 'application/octet-stream' }),
    metadata: new Blob([JSON.stringify(metadata, null, 2) + '\n'], {
      type: 'application/json',
    }),
    sha256,
    baseName,
  }
}

export function downloadBlob(blob: Blob, fileName: string): void {
  const url = URL.createObjectURL(blob)
  const anchor = document.createElement('a')
  anchor.href = url
  anchor.download = fileName
  anchor.click()
  setTimeout(() => URL.revokeObjectURL(url), 0)
}
