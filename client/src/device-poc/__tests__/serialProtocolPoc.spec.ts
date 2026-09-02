import { describe, expect, it } from 'vitest'

import contractSyntheticValidFrames from '../__fixtures__/serial-protocol-v1-valid.jsonl?raw'
import {
  SERIAL_PROTOCOL_POC_MAX_FRAME_BYTES,
  SERIAL_PROTOCOL_V1_CONTROLS,
  SerialProtocolPocParser,
  type SerialProtocolV1Event,
} from '../serialProtocolPoc'

const encoder = new TextEncoder()
const validUpShort = '{"v":1,"control":"up","gesture":"short_press"}'
const validGreenLong = '{"v":1,"control":"green","gesture":"long_press"}'

// All data in this file and the imported JSONL fixture is contract-synthetic. It is not
// hardware-measured serial data and must not be used as evidence of device behavior.
describe('SerialProtocolPocParser contract-synthetic stream', () => {
  it('splitされたframeをchunk間で保持する', () => {
    const parser = new SerialProtocolPocParser()

    expect(parser.pushChunk(encoder.encode(validUpShort.slice(0, 17)))).toEqual([])
    expect(parser.pushChunk(encoder.encode(`${validUpShort.slice(17)}\n`))).toEqual([
      { v: 1, control: 'up', gesture: 'short_press' },
    ])
  })

  it('1 chunk内の複数frameを順番どおり返す', () => {
    const parser = new SerialProtocolPocParser()

    expect(parser.pushChunk(encoder.encode(`${validUpShort}\n${validGreenLong}\n`))).toEqual([
      { v: 1, control: 'up', gesture: 'short_press' },
      { v: 1, control: 'green', gesture: 'long_press' },
    ])
  })

  it('CRLFとLFをdelimiterとして同じstream内で受理する', () => {
    const parser = new SerialProtocolPocParser()

    expect(parser.pushChunk(encoder.encode(`${validUpShort}\r\n${validGreenLong}\n`))).toEqual([
      { v: 1, control: 'up', gesture: 'short_press' },
      { v: 1, control: 'green', gesture: 'long_press' },
    ])
  })

  it('valid JSONL fixtureで全7 controlと両gestureを検証する', () => {
    const parser = new SerialProtocolPocParser()
    const events = parser.pushChunk(encoder.encode(contractSyntheticValidFrames))

    expect(events.map((event) => event.control)).toEqual(SERIAL_PROTOCOL_V1_CONTROLS)
    expect(new Set(events.map((event) => event.gesture))).toEqual(
      new Set(['short_press', 'long_press']),
    )
  })

  it('frame単位でinvalid UTF-8を捨て、次のLF後から再同期する', () => {
    const parser = new SerialProtocolPocParser()
    const validFrame = encoder.encode(`${validUpShort}\n`)
    const chunk = new Uint8Array([0x7b, 0xff, 0x7d, 0x0a, ...validFrame])

    expect(parser.pushChunk(chunk)).toEqual([{ v: 1, control: 'up', gesture: 'short_press' }])
  })

  it('UTF-8 BOMを含む非ASCII payloadを捨てる', () => {
    const parser = new SerialProtocolPocParser()
    const bomPrefixedFrame = new Uint8Array([
      0xef,
      0xbb,
      0xbf,
      ...encoder.encode(`${validUpShort}\n${validGreenLong}\n`),
    ])

    expect(parser.pushChunk(bomPrefixedFrame)).toEqual([
      { v: 1, control: 'green', gesture: 'long_press' },
    ])
  })

  it.each([
    ['unknown field', '{"v":1,"control":"up","gesture":"short_press","extra":true}'],
    ['unknown control', '{"v":1,"control":"blue","gesture":"short_press"}'],
    ['unknown gesture', '{"v":1,"control":"up","gesture":"repeat"}'],
    ['wrong version', '{"v":2,"control":"up","gesture":"short_press"}'],
    ['wrong value type', '{"v":"1","control":1,"gesture":"short_press"}'],
    ['missing field', '{"v":1,"control":"up"}'],
    ['duplicate field', '{"v":1,"control":"down","\\u0063ontrol":"up","gesture":"short_press"}'],
    ['malformed JSON', '{"v":1,"control":"up","gesture":"short_press"'],
  ])('%sを含むlineを捨てる', (_caseName, invalidFrame) => {
    const parser = new SerialProtocolPocParser()

    expect(parser.pushChunk(encoder.encode(`${invalidFrame}\n${validUpShort}\n`))).toEqual([
      { v: 1, control: 'up', gesture: 'short_press' },
    ])
  })

  it('256 byteのframeを受理し、overlong lineを捨てて次のLF後から再同期する', () => {
    const parser = new SerialProtocolPocParser()
    const paddedFrame = validUpShort.padEnd(SERIAL_PROTOCOL_POC_MAX_FRAME_BYTES, ' ')

    expect(encoder.encode(paddedFrame)).toHaveLength(SERIAL_PROTOCOL_POC_MAX_FRAME_BYTES)
    expect(parser.pushChunk(encoder.encode(`${paddedFrame}\r\n`))).toEqual([
      { v: 1, control: 'up', gesture: 'short_press' },
    ])

    const overlongLine = ' '.repeat(SERIAL_PROTOCOL_POC_MAX_FRAME_BYTES + 1)
    expect(parser.pushChunk(encoder.encode(`${overlongLine}\n${validGreenLong}\n`))).toEqual([
      { v: 1, control: 'green', gesture: 'long_press' },
    ])
  })

  it('disconnect時のsession resetでpartial frameとoverlong破棄状態を捨てる', () => {
    const parser = new SerialProtocolPocParser()
    const partialFrame = '{"v":1,"control":"up"'

    expect(parser.pushChunk(encoder.encode(partialFrame))).toEqual([])
    parser.resetSession()
    expect(parser.pushChunk(encoder.encode(`${validGreenLong}\n`))).toEqual([
      { v: 1, control: 'green', gesture: 'long_press' },
    ])

    parser.pushChunk(encoder.encode('x'.repeat(SERIAL_PROTOCOL_POC_MAX_FRAME_BYTES + 1)))
    parser.resetSession()
    const events: SerialProtocolV1Event[] = parser.pushChunk(encoder.encode(`${validUpShort}\n`))
    expect(events).toEqual([{ v: 1, control: 'up', gesture: 'short_press' }])
  })
})
