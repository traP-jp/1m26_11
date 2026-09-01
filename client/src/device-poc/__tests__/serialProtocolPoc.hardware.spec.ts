import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

import {
  SERIAL_PROTOCOL_V1_CONTROLS,
  SerialProtocolPocParser,
  type SerialProtocolV1Event,
} from '../serialProtocolPoc'

interface CaptureConnection {
  id: number
  startedOffset: number
  scriptActiveObservedOffset?: number
  stopRequestedOffset?: number
  endedOffset?: number
}

interface CaptureChunk {
  sequence: number
  connectionId: number
  offset: number
  length: number
}

interface CaptureMetadata {
  captureSchema: string
  rawSha256: string
  totalChunks: number
  totalBytes: number
  connections: CaptureConnection[]
  chunks: CaptureChunk[]
}

interface ParsedChunk {
  sequence: number
  events: SerialProtocolV1Event[]
}

interface ParsedConnection {
  connectionId: number
  chunks: ParsedChunk[]
  events: SerialProtocolV1Event[]
}

interface HardwareCapture {
  metadata: CaptureMetadata
  rawBytes: Uint8Array
}

const sampleDirectory = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../samples/web-serial/protocol-v1',
)

function loadHardwareCapture(stem: string): HardwareCapture {
  const metadata = JSON.parse(
    readFileSync(resolve(sampleDirectory, `${stem}.json`), 'utf8'),
  ) as CaptureMetadata
  const rawBytes = new Uint8Array(readFileSync(resolve(sampleDirectory, `${stem}.bin`)))

  expect(metadata.captureSchema).toBe('web-serial-raw-capture/v2')
  expect(rawBytes).toHaveLength(metadata.totalBytes)
  expect(metadata.chunks).toHaveLength(metadata.totalChunks)
  expect(createHash('sha256').update(rawBytes).digest('hex')).toBe(metadata.rawSha256)

  let expectedOffset = 0
  for (const chunk of metadata.chunks) {
    expect(chunk.offset).toBe(expectedOffset)
    expectedOffset += chunk.length
  }
  expect(expectedOffset).toBe(metadata.totalBytes)

  return { metadata, rawBytes }
}

function parseConnections(
  capture: HardwareCapture,
  selectedConnectionIds: readonly number[],
): ParsedConnection[] {
  const parser = new SerialProtocolPocParser()

  return selectedConnectionIds.map((connectionId) => {
    const connection = capture.metadata.connections.find(({ id }) => id === connectionId)
    if (!connection) throw new Error(`connection ${connectionId} is missing from capture metadata`)

    const payloadStart = connection.scriptActiveObservedOffset
    const payloadEnd = connection.stopRequestedOffset ?? connection.endedOffset
    if (payloadStart === undefined || payloadEnd === undefined) {
      throw new Error(`connection ${connectionId} does not have a complete payload offset range`)
    }
    if (payloadStart < connection.startedOffset || payloadStart > payloadEnd) {
      throw new Error(`connection ${connectionId} has an invalid payload offset range`)
    }

    parser.resetSession()
    const chunks = capture.metadata.chunks
      .filter(
        (chunk) =>
          chunk.connectionId === connectionId &&
          chunk.offset < payloadEnd &&
          chunk.offset + chunk.length > payloadStart,
      )
      .map((chunk): ParsedChunk => {
        const start = Math.max(chunk.offset, payloadStart)
        const end = Math.min(chunk.offset + chunk.length, payloadEnd)
        const events = parser.pushChunk(capture.rawBytes.slice(start, end))

        return { sequence: chunk.sequence, events }
      })

    return {
      connectionId,
      chunks,
      events: chunks.flatMap(({ events }) => events),
    }
  })
}

const normalCaptureStem = 'web-serial-raw-2026-09-01T15-46-21-325Z'
const disconnectCaptureStem = 'web-serial-raw-2026-09-01T15-55-41-921Z'

const upShort: SerialProtocolV1Event = { v: 1, control: 'up', gesture: 'short_press' }

describe('SerialProtocolPocParser hardware-derived capture', () => {
  it('normal captureのconnection 3を実測chunk境界のままactual sequenceへ復元する', () => {
    const [connection] = parseConnections(loadHardwareCapture(normalCaptureStem), [3])

    expect(connection?.events).toEqual([
      upShort,
      { v: 1, control: 'up', gesture: 'long_press' },
      { v: 1, control: 'down', gesture: 'short_press' },
      { v: 1, control: 'left', gesture: 'short_press' },
      { v: 1, control: 'down', gesture: 'short_press' },
      { v: 1, control: 'right', gesture: 'short_press' },
      { v: 1, control: 'red', gesture: 'short_press' },
      { v: 1, control: 'yellow', gesture: 'short_press' },
      { v: 1, control: 'green', gesture: 'short_press' },
      { v: 1, control: 'green', gesture: 'short_press' },
      upShort,
      upShort,
      upShort,
      { v: 1, control: 'right', gesture: 'short_press' },
      { v: 1, control: 'right', gesture: 'short_press' },
      { v: 1, control: 'right', gesture: 'short_press' },
      { v: 1, control: 'right', gesture: 'short_press' },
      { v: 1, control: 'up', gesture: 'long_press' },
    ])
    expect(new Set(connection?.events.map(({ control }) => control))).toEqual(
      new Set(SERIAL_PROTOCOL_V1_CONTROLS),
    )
    expect(new Set(connection?.events.map(({ gesture }) => gesture))).toEqual(
      new Set(['short_press', 'long_press']),
    )

    const eventsBySequence = new Map(
      connection?.chunks.map(({ sequence, events }) => [sequence, events] as const),
    )
    expect(eventsBySequence.get(23)).toEqual([])
    expect(eventsBySequence.get(24)).toEqual([{ v: 1, control: 'down', gesture: 'short_press' }])
    expect(eventsBySequence.get(38)).toEqual([])
    expect(eventsBySequence.get(39)).toEqual([])
    expect(eventsBySequence.get(40)).toEqual([upShort])
  })

  it('USB切断captureをconnectionごとにresetし、両方のup shortを復元する', () => {
    const connections = parseConnections(loadHardwareCapture(disconnectCaptureStem), [1, 2])

    expect(connections.map(({ connectionId, events }) => ({ connectionId, events }))).toEqual([
      { connectionId: 1, events: [upShort] },
      { connectionId: 2, events: [upShort] },
    ])

    const firstConnectionEvents = new Map(
      connections[0]?.chunks.map(({ sequence, events }) => [sequence, events] as const),
    )
    expect(firstConnectionEvents.get(5)).toEqual([])
    expect(firstConnectionEvents.get(6)).toEqual([upShort])
  })
})
