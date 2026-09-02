import { describe, expect, expectTypeOf, it, vi } from 'vitest'

import contractSyntheticValidFrames from '../__fixtures__/serial-protocol-v1-valid.jsonl?raw'
import type { Control, InputAdapterEvent } from '../InputAdapter.types'
import { createWebSerialInputAdapter, type SerialInputAdapterEvent } from '../webSerialInputAdapter'

const encoder = new TextEncoder()
const upShort = '{"v":1,"control":"up","gesture":"short_press"}'
const downLong = '{"v":1,"control":"down","gesture":"long_press"}'

function createAdapter(isControlAllowed: (control: Control) => boolean = () => true) {
  const dispatch = vi.fn<(event: InputAdapterEvent) => boolean>(() => true)
  const adapter = createWebSerialInputAdapter({ dispatcher: { dispatch }, isControlAllowed })

  return { adapter, dispatch }
}

// All stream data in this file and the imported JSONL fixture is contract-synthetic.
describe('WebSerialInputAdapter', () => {
  it('split／複数frameをshort／longを保った共通eventへ順番どおり変換する', () => {
    const { adapter, dispatch } = createAdapter()

    adapter.pushChunk(encoder.encode(upShort.slice(0, 19)))
    expect(dispatch).not.toHaveBeenCalled()

    adapter.pushChunk(encoder.encode(`${upShort.slice(19)}\r\n${downLong}\n`))
    expect(dispatch.mock.calls.map(([event]) => event)).toEqual([
      {
        type: 'condition-changed',
        source: 'serial',
        control: 'up',
        count: 1,
        gesture: 'short_press',
      },
      {
        type: 'condition-changed',
        source: 'serial',
        control: 'down',
        count: 1,
        gesture: 'long_press',
      },
    ])
  })

  it('既存のcontract-synthetic sampleから全frameを共通eventとして再現する', () => {
    const { adapter, dispatch } = createAdapter()

    adapter.pushChunk(encoder.encode(contractSyntheticValidFrames))

    expect(dispatch).toHaveBeenCalledTimes(7)
    expect(
      dispatch.mock.calls.map(([event]) =>
        event.type === 'condition-changed' ? event.control : undefined,
      ),
    ).toEqual(['up', 'down', 'left', 'right', 'red', 'yellow', 'green'])
    expect(
      dispatch.mock.calls.map(([event]) =>
        event.type === 'condition-changed' ? event.gesture : undefined,
      ),
    ).toEqual([
      'short_press',
      'long_press',
      'short_press',
      'long_press',
      'short_press',
      'long_press',
      'short_press',
    ])
  })

  it('invalid frameを無視し、同じvalid frameの連続入力を重複排除しない', () => {
    const { adapter, dispatch } = createAdapter()

    adapter.pushChunk(encoder.encode(`not-json\n${upShort}\n${upShort}\n`))

    expect(dispatch).toHaveBeenCalledTimes(2)
    expect(dispatch.mock.calls[0]?.[0]).toEqual(dispatch.mock.calls[1]?.[0])
  })

  it('dispatcherがeventを拒否しても後続frameを処理する', () => {
    const { adapter, dispatch } = createAdapter()
    dispatch.mockReturnValueOnce(false)

    adapter.pushChunk(encoder.encode(`${upShort}\n${downLong}\n`))

    expect(dispatch).toHaveBeenCalledTimes(2)
    expect(dispatch.mock.calls[1]?.[0]).toMatchObject({ control: 'down', gesture: 'long_press' })
  })

  it('現在の問題で許可されていないcontrolをdispatchしない', () => {
    const { adapter, dispatch } = createAdapter((control) => control === 'up')

    adapter.pushChunk(encoder.encode(`${downLong}\n${upShort}\n`))

    expect(dispatch).toHaveBeenCalledExactlyOnceWith({
      type: 'condition-changed',
      source: 'serial',
      control: 'up',
      count: 1,
      gesture: 'short_press',
    })
  })

  it('session resetで切断前のpartial frameを破棄する', () => {
    const { adapter, dispatch } = createAdapter()

    adapter.pushChunk(encoder.encode(upShort.slice(0, 20)))
    adapter.resetSession()
    adapter.pushChunk(encoder.encode(`${downLong}\n`))

    expect(dispatch).toHaveBeenCalledExactlyOnceWith({
      type: 'condition-changed',
      source: 'serial',
      control: 'down',
      count: 1,
      gesture: 'long_press',
    })
  })

  it('serial eventを共通InputAdapterEventとして扱える', () => {
    expectTypeOf<SerialInputAdapterEvent>().toMatchTypeOf<InputAdapterEvent>()
    expectTypeOf<SerialInputAdapterEvent['gesture']>().toEqualTypeOf<'short_press' | 'long_press'>()
  })
})
