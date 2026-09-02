import { SERIAL_POC_SCRIPT_PATH } from './types'

const textEncoder = new TextEncoder()

const RAW_REPL_BANNER = textEncoder.encode('raw REPL; CTRL-B to exit\r\n>')
const RAW_REPL_END = new Uint8Array([0x04, 0x04, 0x3e])
const RAW_REPL_COMMAND = textEncoder.encode(`exec(open('${SERIAL_POC_SCRIPT_PATH}').read())`)

const CTRL_C = new Uint8Array([0x03])
const INTERRUPT = new Uint8Array([0x0d, 0x03])
const ENTER_RAW_REPL = new Uint8Array([0x0d, 0x01])
const CTRL_D = new Uint8Array([0x04])
const OK = textEncoder.encode('OK')

interface BufferWaiter {
  resolve: () => void
}

export class SerialByteBuffer {
  private readonly bytes: number[] = []
  private readonly waiters = new Set<BufferWaiter>()
  private closedError: Error | undefined

  get length(): number {
    return this.bytes.length
  }

  append(chunk: Uint8Array): void {
    for (const byte of chunk) this.bytes.push(byte)
    this.notifyWaiters()
  }

  close(error = new Error('Serial byte stream closed')): void {
    this.closedError = error
    this.notifyWaiters()
  }

  async waitForQuiet(quietMs: number, timeoutMs: number): Promise<void> {
    const deadline = Date.now() + timeoutMs

    while (Date.now() < deadline) {
      const lengthBeforeWait = this.length
      const changed = await this.waitForUpdate(Math.min(quietMs, deadline - Date.now()))
      if (!changed && this.length === lengthBeforeWait) return
    }

    throw new Error('Timed out waiting for the serial stream to become quiet')
  }

  async waitForExact(expected: Uint8Array, offset: number, timeoutMs: number): Promise<number> {
    await this.waitUntil(() => this.length >= offset + expected.length, timeoutMs)

    for (let index = 0; index < expected.length; index += 1) {
      if (this.bytes[offset + index] !== expected[index]) {
        throw new Error(`Unexpected raw REPL response at byte ${offset + index}`)
      }
    }

    return offset + expected.length
  }

  async waitForSequence(expected: Uint8Array, offset: number, timeoutMs: number): Promise<number> {
    const match = await this.waitForAnySequence([expected], offset, timeoutMs)
    return match.endOffset
  }

  async waitForAnySequence(
    expectedSequences: readonly Uint8Array[],
    offset: number,
    timeoutMs: number,
  ): Promise<{ sequenceIndex: number; endOffset: number }> {
    let match = this.findAnySequence(expectedSequences, offset)
    if (match) return match

    await this.waitUntil(() => {
      match = this.findAnySequence(expectedSequences, offset)
      return match !== undefined
    }, timeoutMs)

    if (!match) throw new Error('Serial byte matcher completed without a match')
    return match
  }

  hasSequence(expected: Uint8Array, offset = 0): boolean {
    return this.findAnySequence([expected], offset) !== undefined
  }

  private findAnySequence(
    expectedSequences: readonly Uint8Array[],
    offset: number,
  ): { sequenceIndex: number; endOffset: number } | undefined {
    for (let byteOffset = offset; byteOffset < this.bytes.length; byteOffset += 1) {
      for (let sequenceIndex = 0; sequenceIndex < expectedSequences.length; sequenceIndex += 1) {
        const expected = expectedSequences[sequenceIndex]
        if (!expected || byteOffset + expected.length > this.bytes.length) continue

        let matches = true
        for (let index = 0; index < expected.length; index += 1) {
          if (this.bytes[byteOffset + index] !== expected[index]) {
            matches = false
            break
          }
        }

        if (matches) {
          return { sequenceIndex, endOffset: byteOffset + expected.length }
        }
      }
    }

    return undefined
  }

  private async waitUntil(predicate: () => boolean, timeoutMs: number): Promise<void> {
    const deadline = Date.now() + timeoutMs

    while (!predicate()) {
      if (this.closedError) throw this.closedError

      const remainingMs = deadline - Date.now()
      if (remainingMs <= 0) throw new Error('Timed out waiting for serial data')
      await this.waitForUpdate(remainingMs)
    }
  }

  private waitForUpdate(timeoutMs: number): Promise<boolean> {
    if (this.closedError) return Promise.reject(this.closedError)

    return new Promise((resolve) => {
      const waiter: BufferWaiter = {
        resolve: () => {
          clearTimeout(timer)
          this.waiters.delete(waiter)
          resolve(true)
        },
      }
      const timer = setTimeout(
        () => {
          this.waiters.delete(waiter)
          resolve(false)
        },
        Math.max(0, timeoutMs),
      )
      this.waiters.add(waiter)
    })
  }

  private notifyWaiters(): void {
    for (const waiter of this.waiters) waiter.resolve()
  }
}

export interface RawReplChannel {
  readonly received: SerialByteBuffer
  write(bytes: Uint8Array): Promise<void>
}

export async function enterMicroPythonRawRepl(channel: RawReplChannel): Promise<void> {
  await channel.write(INTERRUPT)
  await channel.received.waitForQuiet(100, 2_000)

  const bannerOffset = channel.received.length
  await channel.write(ENTER_RAW_REPL)
  await channel.received.waitForSequence(RAW_REPL_BANNER, bannerOffset, 5_000)
}

export async function launchUploadedScript(channel: RawReplChannel): Promise<void> {
  const responseOffset = channel.received.length
  const command = new Uint8Array(RAW_REPL_COMMAND.length + CTRL_D.length)
  command.set(RAW_REPL_COMMAND)
  command.set(CTRL_D, RAW_REPL_COMMAND.length)

  await channel.write(command)
  const stdoutOffset = await channel.received.waitForExact(OK, responseOffset, 1_000)
  await channel.received.waitForQuiet(100, 2_000)
  if (channel.received.hasSequence(CTRL_D, stdoutOffset)) {
    throw new Error(`${SERIAL_POC_SCRIPT_PATH} ended before it remained active`)
  }
}

export async function stopUploadedScript(channel: RawReplChannel): Promise<boolean> {
  const responseOffset = channel.received.length
  await channel.write(CTRL_C)

  try {
    await channel.received.waitForSequence(RAW_REPL_END, responseOffset, 3_000)
    return true
  } catch {
    const retryOffset = channel.received.length
    await channel.write(CTRL_C)

    try {
      await channel.received.waitForSequence(RAW_REPL_END, retryOffset, 1_000)
    } catch {
      // The port is closed below even when the script termination cannot be confirmed.
    }
    return false
  }
}
