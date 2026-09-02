import { describe, expect, it } from 'vitest'

import { concatenateRawChunks, createRawCaptureArtifacts } from '../capture'
import type { RawSerialChunk } from '../types'

const chunks: RawSerialChunk[] = [
  {
    sequence: 1,
    connectionId: 1,
    offset: 0,
    receivedElapsedMs: 1.5,
    bytes: new Uint8Array([0xff, 0xfe]),
  },
  {
    sequence: 2,
    connectionId: 1,
    offset: 2,
    receivedElapsedMs: 2.5,
    bytes: new Uint8Array([0x80, 0x0a]),
  },
]

describe('raw capture', () => {
  it('decodeや改行変換をせずchunkを単純連結する', () => {
    expect([...concatenateRawChunks(chunks)]).toEqual([0xff, 0xfe, 0x80, 0x0a])
  })

  it('正本binとchunk境界を記録したmetadataを生成する', async () => {
    const artifacts = await createRawCaptureArtifacts({
      chunks,
      connections: [
        {
          id: 1,
          startedAt: '2026-08-29T00:00:00.000Z',
          startedOffset: 0,
          scriptPath: '/serial_protocol_poc.py',
          rawReplReadyObservedOffset: 1,
          scriptLaunchRequestedOffset: 1,
          scriptActiveObservedOffset: 2,
          stopRequestedOffset: 2,
          stopCompletedObservedOffset: 4,
          endedAt: '2026-08-29T00:00:01.000Z',
          endedOffset: 4,
          endReason: 'user',
          stopConfirmed: true,
        },
      ],
      capturedAt: '2026-08-29T00:00:00.000Z',
      origin: 'http://localhost:5173',
      userAgent: 'test-browser',
      secureContext: true,
      captureLimitBytes: 1024,
    })

    expect([...new Uint8Array(await artifacts.raw.arrayBuffer())]).toEqual([0xff, 0xfe, 0x80, 0x0a])
    const metadata = JSON.parse(await artifacts.metadata.text()) as {
      captureStopThresholdBytes: number
      captureSchema: string
      totalBytes: number
      rawSha256: string
      connections: Array<{ scriptActiveObservedOffset: number; scriptPath: string }>
      chunks: Array<{ offset: number; length: number }>
    }
    expect(metadata).toMatchObject({
      captureSchema: 'web-serial-raw-capture/v2',
      captureStopThresholdBytes: 1024,
      totalBytes: 4,
      rawSha256: '38a27c80957be56cb326a25a2e3bc468362db68db71b33018e968716a4da4c07',
      connections: [{ scriptActiveObservedOffset: 2, scriptPath: '/serial_protocol_poc.py' }],
      chunks: [
        { offset: 0, length: 2 },
        { offset: 2, length: 2 },
      ],
    })
    expect(artifacts.sha256).toBe(metadata.rawSha256)
  })
})
