import { describe, expect, it, vi } from 'vitest'

import hardwareSample from '../__fixtures__/serial-protocol-v1-hardware-sample.jsonl?raw'
import contractSyntheticValidFrames from '../__fixtures__/serial-protocol-v1-valid.jsonl?raw'
import type { Control, InputAdapterDispatcher, InputAdapterEvent } from '../InputAdapter.types'
import { createOperationBuffer } from '../operationBuffer'
import { SERIAL_PROTOCOL_V1_CONTROLS } from '../serialFrameParser'
import { createWebSerialInputAdapter } from '../webSerialInputAdapter'

const encoder = new TextEncoder()
const upShort = '{"v":1,"control":"up","gesture":"short_press"}'
const downLong = '{"v":1,"control":"down","gesture":"long_press"}'

function createAdapter(isControlAllowed: (control: Control) => boolean = () => true) {
  const events: InputAdapterEvent[] = []
  const dispatch = vi.fn<(event: InputAdapterEvent) => boolean>((event) => {
    events.push(event)
    return true
  })
  const dispatcher: InputAdapterDispatcher = { busy: false, dispatch }
  const adapter = createWebSerialInputAdapter({ dispatcher, isControlAllowed })

  return { adapter, dispatch, events }
}

describe('WebSerialInputAdapter', () => {
  // This fixture is the single canonical frame observed by direct read from the production Pico
  // on 2026-09-03, normalized to an LF-terminated JSONL line. No raw capture is stored here.
  it('実機sampleをparserから共通eventまで再現する', () => {
    const { adapter, events } = createAdapter()

    adapter.pushChunk(encoder.encode(hardwareSample))

    expect(events).toEqual([
      { type: 'condition-changed', source: 'serial', control: 'up', count: 1 },
    ])
  })

  it('分割された複数frameをshort／longともcount 1の共通eventへ順番どおり変換する', () => {
    const { adapter, events } = createAdapter()

    adapter.pushChunk(encoder.encode(upShort.slice(0, 19)))
    expect(events).toEqual([])

    adapter.pushChunk(encoder.encode(`${upShort.slice(19)}\r\n${downLong}\n`))
    expect(events).toEqual([
      { type: 'condition-changed', source: 'serial', control: 'up', count: 1 },
      { type: 'condition-changed', source: 'serial', control: 'down', count: 1 },
    ])
  })

  it('contract-synthetic sampleの全controlをgestureなしの共通eventへ変換する', () => {
    const { adapter, events } = createAdapter()

    adapter.pushChunk(encoder.encode(contractSyntheticValidFrames))

    expect(events).toEqual(
      SERIAL_PROTOCOL_V1_CONTROLS.map((control) => ({
        type: 'condition-changed',
        source: 'serial',
        control,
        count: 1,
      })),
    )
    expect(events.every((event) => !('gesture' in event))).toBe(true)
  })

  it('invalid frameを無視し、同じvalid frameをfrontendでdebounce／重複除去しない', () => {
    const { adapter, events } = createAdapter()
    const buffer = createOperationBuffer()

    adapter.pushChunk(encoder.encode(`not-json\n${upShort}\n${upShort}\n`))
    for (const event of events) {
      if (event.type === 'condition-changed') buffer.append(event)
    }

    expect(events).toEqual([
      { type: 'condition-changed', source: 'serial', control: 'up', count: 1 },
      { type: 'condition-changed', source: 'serial', control: 'up', count: 1 },
    ])
    expect(buffer.snapshot()).toEqual([{ control: 'up', count: 2 }])
  })

  it('dispatcherがeventを拒否しても後続frameを処理する', () => {
    const { adapter, dispatch } = createAdapter()
    dispatch.mockReturnValueOnce(false)

    adapter.pushChunk(encoder.encode(`${upShort}\n${downLong}\n`))

    expect(dispatch).toHaveBeenCalledTimes(2)
    expect(dispatch.mock.calls[1]?.[0]).toEqual({
      type: 'condition-changed',
      source: 'serial',
      control: 'down',
      count: 1,
    })
  })

  it('現在の問題で許可されていないcontrolをdispatchしない', () => {
    const { adapter, events } = createAdapter((control) => control === 'up')

    adapter.pushChunk(encoder.encode(`${downLong}\n${upShort}\n`))

    expect(events).toEqual([
      { type: 'condition-changed', source: 'serial', control: 'up', count: 1 },
    ])
  })

  it('session resetで切断前のpartial frameを破棄する', () => {
    const { adapter, events } = createAdapter()

    adapter.pushChunk(encoder.encode(upShort.slice(0, 20)))
    adapter.resetSession()
    adapter.pushChunk(encoder.encode(`${downLong}\n`))

    expect(events).toEqual([
      { type: 'condition-changed', source: 'serial', control: 'down', count: 1 },
    ])
  })
})
