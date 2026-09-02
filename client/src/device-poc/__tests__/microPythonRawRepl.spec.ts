import { describe, expect, it } from 'vitest'

import {
  enterMicroPythonRawRepl,
  launchUploadedScript,
  SerialByteBuffer,
  stopUploadedScript,
  type RawReplChannel,
} from '../microPythonRawRepl'

const encoder = new TextEncoder()

class FakeRawReplChannel implements RawReplChannel {
  readonly received = new SerialByteBuffer()
  readonly writes: number[][] = []

  async write(bytes: Uint8Array): Promise<void> {
    this.writes.push([...bytes])

    if (bytes[0] === 0x0d && bytes[1] === 0x01) {
      this.received.append(encoder.encode('raw RE'))
      this.received.append(encoder.encode('PL; CTRL-B to exit\r'))
      this.received.append(encoder.encode('\n>'))
      return
    }

    if (bytes[bytes.length - 1] === 0x04) {
      this.received.append(encoder.encode('O'))
      this.received.append(encoder.encode('K'))
      return
    }

    if (bytes.length === 1 && bytes[0] === 0x03) {
      this.received.append(new Uint8Array([0x04]))
      this.received.append(new Uint8Array([0x04]))
      this.received.append(new Uint8Array([0x3e]))
    }
  }
}

describe('SerialByteBuffer', () => {
  it('chunk境界をまたぐbyte列を検出する', async () => {
    const buffer = new SerialByteBuffer()
    const matchPromise = buffer.waitForSequence(encoder.encode('raw REPL'), 0, 1_000)

    buffer.append(encoder.encode('ra'))
    buffer.append(encoder.encode('w REP'))
    buffer.append(encoder.encode('L'))

    await expect(matchPromise).resolves.toBe(8)
  })

  it('期待位置のbyteが異なる場合はraw REPL protocol errorにする', async () => {
    const buffer = new SerialByteBuffer()
    buffer.append(encoder.encode('NO'))

    await expect(buffer.waitForExact(encoder.encode('OK'), 0, 100)).rejects.toThrow(
      'Unexpected raw REPL response',
    )
  })
})

describe('MicroPython raw REPL bootstrap', () => {
  it('raw REPLへ入り、Upload済みprotocol PoCを起動して停止する', async () => {
    const channel = new FakeRawReplChannel()

    await enterMicroPythonRawRepl(channel)
    await launchUploadedScript(channel)
    await expect(stopUploadedScript(channel)).resolves.toBe(true)

    expect(channel.writes[0]).toEqual([0x0d, 0x03])
    expect(channel.writes[1]).toEqual([0x0d, 0x01])
    expect(new TextDecoder().decode(new Uint8Array(channel.writes[2]!.slice(0, -1)))).toBe(
      "exec(open('/serial_protocol_poc.py').read())",
    )
    expect(channel.writes[2]![channel.writes[2]!.length - 1]).toBe(0x04)
    expect(channel.writes[3]).toEqual([0x03])
  })

  it('起動直後にstdout EOFを受信した場合はrunning扱いにしない', async () => {
    const received = new SerialByteBuffer()
    const channel: RawReplChannel = {
      received,
      write: async (bytes) => {
        if (bytes[bytes.length - 1] === 0x04) {
          received.append(
            new Uint8Array([
              0x4f,
              0x4b,
              ...encoder.encode('{"v":1,"control":"up","gesture":"short_press"}\r\n'),
              0x04,
              0x04,
              0x3e,
            ]),
          )
        }
      },
    }

    await expect(launchUploadedScript(channel)).rejects.toThrow('ended before it remained active')
  })

  it('仮の診断出力文言を起動契約として解釈しない', async () => {
    const received = new SerialByteBuffer()
    const channel: RawReplChannel = {
      received,
      write: async (bytes) => {
        if (bytes[bytes.length - 1] === 0x04) {
          received.append(encoder.encode('OKtemporary output may change\r\n'))
        }
      },
    }

    await expect(launchUploadedScript(channel)).resolves.toBeUndefined()
  })
})
