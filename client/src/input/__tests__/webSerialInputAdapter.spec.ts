import { describe, expect, it, vi } from 'vitest'

import contractSyntheticValidFrames from '../__fixtures__/serial-protocol-v1-valid.jsonl?raw'
import type { SerialProtocolV1Control, SerialProtocolV1Frame } from '../serialFrameParser'
import { createWebSerialInputAdapter } from '../webSerialInputAdapter'

const encoder = new TextEncoder()
const upShort = '{"v":1,"control":"up","gesture":"short_press"}'
const downLong = '{"v":1,"control":"down","gesture":"long_press"}'

function createAdapter(
  isControlAllowed: (control: SerialProtocolV1Control) => boolean = () => true,
) {
  const onFrame = vi.fn<(frame: SerialProtocolV1Frame) => void>()
  const adapter = createWebSerialInputAdapter({ onFrame, isControlAllowed })

  return { adapter, onFrame }
}

// All stream data in this file and the imported JSONL fixture is contract-synthetic.
describe('WebSerialInputAdapter', () => {
  it('split／複数frameをshort／longを保ったSerial frameとして順番どおり渡す', () => {
    const { adapter, onFrame } = createAdapter()

    adapter.pushChunk(encoder.encode(upShort.slice(0, 19)))
    expect(onFrame).not.toHaveBeenCalled()

    adapter.pushChunk(encoder.encode(`${upShort.slice(19)}\r\n${downLong}\n`))
    expect(onFrame.mock.calls.map(([frame]) => frame)).toEqual([
      {
        v: 1,
        control: 'up',
        gesture: 'short_press',
      },
      {
        v: 1,
        control: 'down',
        gesture: 'long_press',
      },
    ])
  })

  it('既存のcontract-synthetic sampleから全Serial frameを再現する', () => {
    const { adapter, onFrame } = createAdapter()

    adapter.pushChunk(encoder.encode(contractSyntheticValidFrames))

    expect(onFrame).toHaveBeenCalledTimes(7)
    expect(onFrame.mock.calls.map(([frame]) => frame.control)).toEqual([
      'up',
      'down',
      'left',
      'right',
      'red',
      'yellow',
      'green',
    ])
    expect(onFrame.mock.calls.map(([frame]) => frame.gesture)).toEqual([
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
    const { adapter, onFrame } = createAdapter()

    adapter.pushChunk(encoder.encode(`not-json\n${upShort}\n${upShort}\n`))

    expect(onFrame).toHaveBeenCalledTimes(2)
    expect(onFrame.mock.calls[0]?.[0]).toEqual(onFrame.mock.calls[1]?.[0])
  })

  it('現在の問題で許可されていないcontrolをhandlerへ渡さない', () => {
    const { adapter, onFrame } = createAdapter((control) => control === 'up')

    adapter.pushChunk(encoder.encode(`${downLong}\n${upShort}\n`))

    expect(onFrame).toHaveBeenCalledExactlyOnceWith({
      v: 1,
      control: 'up',
      gesture: 'short_press',
    })
  })

  it('session resetで切断前のpartial frameを破棄する', () => {
    const { adapter, onFrame } = createAdapter()

    adapter.pushChunk(encoder.encode(upShort.slice(0, 20)))
    adapter.resetSession()
    adapter.pushChunk(encoder.encode(`${downLong}\n`))

    expect(onFrame).toHaveBeenCalledExactlyOnceWith({
      v: 1,
      control: 'down',
      gesture: 'long_press',
    })
  })
})
