import { createHash } from 'node:crypto'
import { readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

import { describe, expect, it } from 'vitest'

import { SerialProtocolPocParser, type SerialProtocolV1Event } from '../serialProtocolPoc'

const sampleDirectory = resolve(
  dirname(fileURLToPath(import.meta.url)),
  '../../../../device/samples/issue81-production',
)
const sampleStems = [
  '20260902-issue81-hard-reset-idle',
  '20260902-issue81-all-controls',
  '20260902-issue81-rapid',
  '20260902-issue81-long',
  '20260902-issue81-bounce-prone',
  '20260902-issue81-mixed-colors',
  '20260902-issue81-power-cycle',
  '20260902-issue81-power-cycle-idle',
  '20260902-issue81-held-power-cycle',
] as const

const LF = 0x0a
const CR = 0x0d
const asciiDecoder = new TextDecoder('utf-8', { fatal: true })

function readSample(stem: string, suffix: '.bin' | '.expected.jsonl'): Uint8Array {
  return new Uint8Array(readFileSync(resolve(sampleDirectory, `${stem}${suffix}`)))
}

function expectRecordedSha256(stem: string, rawBytes: Uint8Array): void {
  const record = readFileSync(resolve(sampleDirectory, `${stem}.sha256`), 'utf8')
  const rawSha256 = createHash('sha256').update(rawBytes).digest('hex')

  expect(record, `${stem}.sha256 must use the sha256sum text format`).toMatch(
    /^[0-9a-f]{64} {2}\S+\.bin\n$/,
  )
  expect(record, `${stem}.sha256 must match the unmodified raw capture`).toBe(
    `${rawSha256}  ${stem}.bin\n`,
  )
}

function parseCanonicalEventStream(bytes: Uint8Array, label: string): SerialProtocolV1Event[] {
  const parser = new SerialProtocolPocParser()
  const events = parser.pushChunk(bytes)

  expect(
    bytes.every((byte) => byte <= 0x7f),
    `${label} must contain only canonical ASCII JSON and LF/CRLF delimiters`,
  ).toBe(true)

  const lines: string[] = []
  let lineStart = 0

  for (let index = 0; index < bytes.length; index += 1) {
    if (bytes[index] !== LF) continue

    const hasCrTerminator = index > lineStart && bytes[index - 1] === CR
    const lineEnd = hasCrTerminator ? index - 1 : index
    const lineBytes = bytes.slice(lineStart, lineEnd)

    expect(lineBytes.length, `${label} must not contain empty lines`).toBeGreaterThan(0)
    lines.push(asciiDecoder.decode(lineBytes))
    lineStart = index + 1
  }

  expect(lineStart, `${label} must not end with a partial frame`).toBe(bytes.length)
  expect(lines, `${label} must contain only canonical event JSON lines`).toEqual(
    events.map((event) => JSON.stringify(event)),
  )

  return events
}

describe('SerialProtocolPocParser Issue #81 production hardware samples', () => {
  it.each(sampleStems)('%sのraw event列が期待列と完全一致する', (stem) => {
    const rawBytes = readSample(stem, '.bin')
    expectRecordedSha256(stem, rawBytes)

    const actualEvents = parseCanonicalEventStream(rawBytes, `${stem}.bin`)
    const expectedEvents = parseCanonicalEventStream(
      readSample(stem, '.expected.jsonl'),
      `${stem}.expected.jsonl`,
    )

    expect(actualEvents).toEqual(expectedEvents)
  })
})
