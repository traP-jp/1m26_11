import { effectScope } from 'vue'
import { describe, expect, it, vi } from 'vitest'

import {
  RASPBERRY_PI_PICO_PORT_FILTER,
  useWebSerialConnection,
  WEB_SERIAL_OPEN_OPTIONS,
  type SerialPortLike,
  type SerialReaderLike,
  type WebSerialLike,
} from '../useWebSerialConnection'

interface PendingRead {
  resolve: (result: ReadableStreamReadResult<Uint8Array>) => void
  reject: (error: unknown) => void
}

class FakeSerialReader implements SerialReaderLike {
  private pending?: PendingRead
  private queued?: ReadableStreamReadResult<Uint8Array>

  constructor(private readonly events: string[]) {}

  read(): Promise<ReadableStreamReadResult<Uint8Array>> {
    if (this.queued) {
      const result = this.queued
      this.queued = undefined
      return Promise.resolve(result)
    }

    return new Promise((resolve, reject) => {
      this.pending = { resolve, reject }
    })
  }

  async cancel(): Promise<void> {
    this.events.push('reader-cancel')
    this.finish({ done: true, value: undefined })
  }

  releaseLock(): void {
    this.events.push('reader-release')
  }

  emit(bytes: Uint8Array): void {
    this.finish({ done: false, value: bytes })
  }

  end(): void {
    this.finish({ done: true, value: undefined })
  }

  fail(error: unknown): void {
    const pending = this.pending
    this.pending = undefined
    pending?.reject(error)
  }

  private finish(result: ReadableStreamReadResult<Uint8Array>): void {
    const pending = this.pending
    this.pending = undefined
    if (pending) {
      pending.resolve(result)
    } else {
      this.queued = result
    }
  }
}

class FakeSerialPort implements SerialPortLike {
  readonly events: string[] = []
  readonly reader = new FakeSerialReader(this.events)
  readonly readable = {
    getReader: (): SerialReaderLike => {
      this.events.push('reader-get')
      return this.reader
    },
  }
  openedWith: unknown
  openError?: unknown
  closeFailureCount = 0
  isOpen = false

  async open(options: unknown): Promise<void> {
    this.openedWith = options
    this.events.push('port-open')
    if (this.openError) throw this.openError
    this.isOpen = true
  }

  async close(): Promise<void> {
    this.events.push('port-close')
    if (this.closeFailureCount > 0) {
      this.closeFailureCount -= 1
      throw new DOMException('Port is busy', 'InvalidStateError')
    }
    this.isOpen = false
  }
}

class DeferredOpenSerialPort extends FakeSerialPort {
  private readonly openBarrier: Promise<void>
  private resolveOpen!: () => void

  constructor() {
    super()
    this.openBarrier = new Promise((resolve) => {
      this.resolveOpen = resolve
    })
  }

  override async open(options: unknown): Promise<void> {
    this.openedWith = options
    this.events.push('port-open-start')
    await this.openBarrier
    this.isOpen = true
    this.events.push('port-open')
  }

  finishOpening(): void {
    this.resolveOpen()
  }
}

class FakeWebSerial implements WebSerialLike {
  readonly requestPort = vi.fn<WebSerialLike['requestPort']>()
  private readonly disconnectListeners = new Set<(event: Event) => void>()

  constructor(port?: SerialPortLike) {
    if (port) this.requestPort.mockResolvedValue(port)
  }

  addEventListener(_type: 'disconnect', listener: (event: Event) => void): void {
    this.disconnectListeners.add(listener)
  }

  removeEventListener(_type: 'disconnect', listener: (event: Event) => void): void {
    this.disconnectListeners.delete(listener)
  }

  emitDisconnect(port: SerialPortLike): void {
    const event = { target: port } as unknown as Event
    for (const listener of this.disconnectListeners) listener(event)
  }

  deferDisconnect(port: SerialPortLike): () => void {
    const event = { target: port } as unknown as Event
    const listeners = [...this.disconnectListeners]
    return () => {
      for (const listener of listeners) listener(event)
    }
  }

  get listenerCount(): number {
    return this.disconnectListeners.size
  }
}

function createAdapter() {
  return {
    pushChunk: vi.fn<(chunk: Uint8Array) => void>(),
    resetSession: vi.fn<() => void>(),
  }
}

function createConnection(serial: WebSerialLike | null, adapter = createAdapter()) {
  const connection = useWebSerialConnection({ serial, secureContext: true, adapter })
  return { connection, adapter }
}

describe('useWebSerialConnection', () => {
  it('非対応状態を区別し、構築だけではportを要求しない', () => {
    const port = new FakeSerialPort()
    const serial = new FakeWebSerial(port)
    const { connection } = createConnection(serial)
    const insecure = useWebSerialConnection({
      serial,
      secureContext: false,
      adapter: createAdapter(),
    })
    const unavailable = createConnection(null).connection

    expect(connection.state.value.phase).toBe('idle')
    expect(serial.requestPort).not.toHaveBeenCalled()
    expect(insecure.state.value).toMatchObject({
      phase: 'unsupported',
      reason: 'insecure-context',
    })
    expect(unavailable.state.value).toMatchObject({
      phase: 'unsupported',
      reason: 'api-unavailable',
    })
  })

  it('利用者のconnect呼出しからPicoをopenし、chunkをAdapterへ順番どおり渡す', async () => {
    const port = new FakeSerialPort()
    const serial = new FakeWebSerial(port)
    const { connection, adapter } = createConnection(serial)

    const connectPromise = connection.connect()
    expect(serial.requestPort).toHaveBeenCalledExactlyOnceWith({
      filters: [RASPBERRY_PI_PICO_PORT_FILTER],
    })
    await connectPromise

    expect(port.openedWith).toEqual(WEB_SERIAL_OPEN_OPTIONS)
    expect(connection.state.value.phase).toBe('connected')

    const first = new Uint8Array([1, 2])
    const second = new Uint8Array([3])
    port.reader.emit(first)
    await vi.waitFor(() => expect(adapter.pushChunk).toHaveBeenCalledTimes(1))
    port.reader.emit(second)
    await vi.waitFor(() => expect(adapter.pushChunk).toHaveBeenCalledTimes(2))
    expect(adapter.pushChunk.mock.calls.map(([chunk]) => [...chunk])).toEqual([[1, 2], [3]])

    await connection.disconnect()
  })

  it('port選択拒否とopen失敗を区別し、未open portをcloseしない', async () => {
    const rejectedSerial = new FakeWebSerial()
    rejectedSerial.requestPort.mockRejectedValue(new DOMException('Denied', 'NotFoundError'))
    const rejected = createConnection(rejectedSerial).connection

    await rejected.connect()
    expect(rejected.state.value).toMatchObject({ phase: 'error', operation: 'request-port' })
    expect(rejected.canRetry.value).toBe(true)

    const port = new FakeSerialPort()
    port.openError = new DOMException('In use', 'NetworkError')
    const openFailed = createConnection(new FakeWebSerial(port)).connection

    await openFailed.connect()
    expect(openFailed.state.value).toMatchObject({ phase: 'error', operation: 'open-port' })
    expect(port.events).toEqual(['port-open'])
    expect(openFailed.canRetry.value).toBe(true)
  })

  it('read開始失敗後にportをcloseできない場合も物理切断を検知する', async () => {
    const port = new FakeSerialPort()
    Object.defineProperty(port, 'readable', { value: null })
    port.closeFailureCount = 1
    const serial = new FakeWebSerial(port)
    const { connection } = createConnection(serial)

    await connection.connect()
    expect(connection.state.value).toMatchObject({ phase: 'error', operation: 'close-port' })
    expect(connection.canDisconnect.value).toBe(true)

    serial.emitDisconnect(port)
    await vi.waitFor(() => expect(connection.state.value.phase).toBe('disconnected'))
    expect(connection.state.value).toMatchObject({ reason: 'device-disconnected' })
    expect(connection.canDisconnect.value).toBe(false)
  })

  it('reader取得失敗後にportをcloseできない場合も物理切断を検知する', async () => {
    const port = new FakeSerialPort()
    Object.defineProperty(port, 'readable', {
      value: {
        getReader() {
          throw new DOMException('Reader unavailable', 'InvalidStateError')
        },
      },
    })
    port.closeFailureCount = 1
    const serial = new FakeWebSerial(port)
    const { connection } = createConnection(serial)

    await connection.connect()
    expect(connection.state.value).toMatchObject({ phase: 'error', operation: 'close-port' })
    expect(connection.canDisconnect.value).toBe(true)

    serial.emitDisconnect(port)
    await vi.waitFor(() => expect(connection.state.value.phase).toBe('disconnected'))
    expect(connection.state.value).toMatchObject({ reason: 'device-disconnected' })
    expect(connection.canDisconnect.value).toBe(false)
  })

  it('requesting中の二重接続でport選択を重複させない', async () => {
    const port = new FakeSerialPort()
    let resolvePort!: (port: SerialPortLike) => void
    const selectedPort = new Promise<SerialPortLike>((resolve) => {
      resolvePort = resolve
    })
    const serial = new FakeWebSerial()
    serial.requestPort.mockImplementation(() => selectedPort)
    const { connection } = createConnection(serial)

    const first = connection.connect()
    const second = connection.connect()
    expect(serial.requestPort).toHaveBeenCalledTimes(1)

    resolvePort(port)
    await Promise.all([first, second])
    expect(port.events.filter((event) => event === 'port-open')).toHaveLength(1)

    await connection.disconnect()
  })

  it('正常終了でreaderをcancel・解放してからportをcloseし、処理を一度だけ行う', async () => {
    const port = new FakeSerialPort()
    const { connection, adapter } = createConnection(new FakeWebSerial(port))
    await connection.connect()

    await Promise.all([connection.disconnect(), connection.disconnect()])

    expect(port.events).toEqual([
      'port-open',
      'reader-get',
      'reader-cancel',
      'reader-release',
      'port-close',
    ])
    expect(adapter.resetSession).toHaveBeenCalledTimes(1)
    expect(connection.state.value).toMatchObject({ phase: 'disconnected', reason: 'user' })
    expect(connection.canRetry.value).toBe(true)
  })

  it('disconnect開始前にreadが解決していても古いsessionのchunkを渡さない', async () => {
    const port = new FakeSerialPort()
    const { connection, adapter } = createConnection(new FakeWebSerial(port))
    await connection.connect()

    port.reader.emit(new Uint8Array([1, 2, 3]))
    await connection.disconnect()

    expect(adapter.pushChunk).not.toHaveBeenCalled()
    expect(adapter.resetSession).toHaveBeenCalledTimes(1)
  })

  it('stream終了とread失敗を別の状態として通知し、resourceを解放する', async () => {
    const endedPort = new FakeSerialPort()
    const ended = createConnection(new FakeWebSerial(endedPort))
    await ended.connection.connect()
    endedPort.reader.end()
    await vi.waitFor(() => expect(ended.connection.state.value.phase).toBe('disconnected'))
    expect(ended.connection.state.value).toMatchObject({ reason: 'stream-ended' })
    expect(endedPort.events).toContain('port-close')
    expect(ended.adapter.resetSession).toHaveBeenCalledTimes(1)

    const failedPort = new FakeSerialPort()
    const failed = createConnection(new FakeWebSerial(failedPort))
    await failed.connection.connect()
    failedPort.reader.fail(new DOMException('Read failed', 'UnknownError'))
    await vi.waitFor(() => expect(failed.connection.state.value.phase).toBe('error'))
    expect(failed.connection.state.value).toMatchObject({ operation: 'read' })
    expect(failedPort.events).toContain('port-close')
    expect(failed.adapter.resetSession).toHaveBeenCalledTimes(1)
  })

  it('現在のportの物理切断だけを扱い、自動再接続しない', async () => {
    const port = new FakeSerialPort()
    const otherPort = new FakeSerialPort()
    const serial = new FakeWebSerial(port)
    const { connection } = createConnection(serial)
    await connection.connect()

    serial.emitDisconnect(otherPort)
    await Promise.resolve()
    expect(connection.state.value.phase).toBe('connected')

    serial.emitDisconnect(port)
    await vi.waitFor(() => expect(connection.state.value.phase).toBe('disconnected'))
    expect(connection.state.value).toMatchObject({ reason: 'device-disconnected' })
    expect(serial.requestPort).toHaveBeenCalledTimes(1)
  })

  it('利用者のretryだけが新しいportを選択し、再接続失敗を区別する', async () => {
    const firstPort = new FakeSerialPort()
    const serial = new FakeWebSerial(firstPort)
    const { connection } = createConnection(serial)
    await connection.connect()
    await connection.disconnect()

    serial.requestPort.mockRejectedValueOnce(new DOMException('Still missing', 'NotFoundError'))
    expect(serial.requestPort).toHaveBeenCalledTimes(1)
    await connection.retry()

    expect(serial.requestPort).toHaveBeenCalledTimes(2)
    expect(connection.state.value).toMatchObject({ phase: 'error', operation: 'reconnect' })
  })

  it('再接続では解放済みportを暗黙に再利用せず、再選択したportを使う', async () => {
    const firstPort = new FakeSerialPort()
    const secondPort = new FakeSerialPort()
    const serial = new FakeWebSerial()
    serial.requestPort.mockResolvedValueOnce(firstPort).mockResolvedValueOnce(secondPort)
    const { connection } = createConnection(serial)

    await connection.connect()
    await connection.disconnect()
    await connection.retry()

    expect(firstPort.events.filter((event) => event === 'port-open')).toHaveLength(1)
    expect(secondPort.events.filter((event) => event === 'port-open')).toHaveLength(1)
    expect(connection.state.value.phase).toBe('connected')

    await connection.disconnect()
  })

  it('再接続後に遅着した旧sessionの物理切断で同じportの新sessionを閉じない', async () => {
    const port = new FakeSerialPort()
    const serial = new FakeWebSerial(port)
    const { connection, adapter } = createConnection(serial)

    await connection.connect()
    const emitLateDisconnect = serial.deferDisconnect(port)
    await connection.disconnect()
    await connection.retry()
    expect(connection.state.value.phase).toBe('connected')

    const eventsBeforeLateDisconnect = [...port.events]
    emitLateDisconnect()
    await Promise.resolve()

    expect(connection.state.value.phase).toBe('connected')
    expect(connection.busy.value).toBe(false)
    expect(port.events).toEqual(eventsBeforeLateDisconnect)
    expect(adapter.resetSession).toHaveBeenCalledTimes(1)

    const liveChunk = new Uint8Array([7, 8, 9])
    port.reader.emit(liveChunk)
    await vi.waitFor(() => expect(adapter.pushChunk).toHaveBeenCalledExactlyOnceWith(liveChunk))

    serial.emitDisconnect(port)
    await vi.waitFor(() => expect(connection.state.value.phase).toBe('disconnected'))
    expect(connection.state.value).toMatchObject({ reason: 'device-disconnected' })
  })

  it('画面破棄時にreaderとportをcleanupする', async () => {
    const port = new FakeSerialPort()
    const serial = new FakeWebSerial(port)
    const adapter = createAdapter()
    const scope = effectScope()
    const connection = scope.run(() =>
      useWebSerialConnection({ serial, secureContext: true, adapter }),
    )!
    await connection.connect()

    scope.stop()

    await vi.waitFor(() => expect(port.events).toContain('port-close'))
    expect(port.events.indexOf('reader-release')).toBeLessThan(port.events.indexOf('port-close'))
    expect(adapter.resetSession).toHaveBeenCalledTimes(1)
    expect(serial.listenerCount).toBe(0)
  })

  it('画面破棄中にcloseが失敗し続けてもport参照とlistenerを残さない', async () => {
    const port = new FakeSerialPort()
    port.closeFailureCount = 2
    const serial = new FakeWebSerial(port)
    const scope = effectScope()
    const connection = scope.run(() =>
      useWebSerialConnection({
        serial,
        secureContext: true,
        adapter: createAdapter(),
      }),
    )!
    await connection.connect()

    scope.stop()

    await vi.waitFor(() =>
      expect(port.events.filter((event) => event === 'port-close')).toHaveLength(2),
    )
    expect(port.isOpen).toBe(true)
    expect(connection.canDisconnect.value).toBe(false)
    expect(serial.listenerCount).toBe(0)
  })

  it('open待機中の画面破棄後にopenされても、portのcloseを再試行する', async () => {
    const port = new DeferredOpenSerialPort()
    port.closeFailureCount = 1
    const scope = effectScope()
    const connection = scope.run(() =>
      useWebSerialConnection({
        serial: new FakeWebSerial(port),
        secureContext: true,
        adapter: createAdapter(),
      }),
    )!

    const connectPromise = connection.connect()
    await vi.waitFor(() => expect(port.events).toContain('port-open-start'))
    scope.stop()
    port.finishOpening()
    await connectPromise

    expect(port.events.filter((event) => event === 'port-close')).toHaveLength(2)
    expect(port.isOpen).toBe(false)
    expect(connection.canDisconnect.value).toBe(false)
  })

  it('接続選択中に終了した場合、後から選択されたportをopenしない', async () => {
    const port = new FakeSerialPort()
    let resolvePort!: (port: SerialPortLike) => void
    const selectedPort = new Promise<SerialPortLike>((resolve) => {
      resolvePort = resolve
    })
    const serial = new FakeWebSerial()
    serial.requestPort.mockImplementation(() => selectedPort)
    const { connection } = createConnection(serial)

    const connectPromise = connection.connect()
    await connection.disconnect()
    resolvePort(port)
    await connectPromise

    expect(port.events).toEqual([])
    expect(connection.state.value).toMatchObject({ phase: 'disconnected', reason: 'user' })
  })

  it('open待機中の接続中止でcloseに失敗しても、物理切断を検知して解放する', async () => {
    const port = new DeferredOpenSerialPort()
    port.closeFailureCount = 1
    const serial = new FakeWebSerial(port)
    const { connection } = createConnection(serial)

    const connectPromise = connection.connect()
    await vi.waitFor(() => expect(port.events).toContain('port-open-start'))
    await connection.disconnect()
    port.finishOpening()
    await connectPromise

    expect(connection.state.value).toMatchObject({ phase: 'error', operation: 'close-port' })
    expect(connection.canDisconnect.value).toBe(true)

    serial.emitDisconnect(port)
    await vi.waitFor(() => expect(connection.state.value.phase).toBe('disconnected'))
    expect(connection.state.value).toMatchObject({ reason: 'device-disconnected' })
    expect(connection.canDisconnect.value).toBe(false)
  })

  it('readのNetworkErrorを物理切断として扱い、disconnect eventが遅着してもcleanupを重複しない', async () => {
    const port = new FakeSerialPort()
    const serial = new FakeWebSerial(port)
    const { connection, adapter } = createConnection(serial)
    await connection.connect()

    const emitLateDisconnect = serial.deferDisconnect(port)
    port.reader.fail(new DOMException('Device lost', 'NetworkError'))
    await vi.waitFor(() => expect(connection.state.value.phase).toBe('disconnected'))
    expect(connection.state.value).toMatchObject({ reason: 'device-disconnected' })
    expect(adapter.resetSession).toHaveBeenCalledTimes(1)
    expect(port.events.filter((event) => event === 'port-close')).toHaveLength(1)

    emitLateDisconnect()
    await vi.waitFor(() => expect(connection.busy.value).toBe(false))
    expect(connection.state.value).toMatchObject({
      phase: 'disconnected',
      reason: 'device-disconnected',
    })
    expect(connection.canDisconnect.value).toBe(false)
    expect(serial.listenerCount).toBe(0)
    expect(adapter.resetSession).toHaveBeenCalledTimes(1)
    expect(port.events.filter((event) => event === 'port-close')).toHaveLength(1)
  })

  it('port close失敗で保持したsessionも現在の物理切断eventで解放する', async () => {
    const port = new FakeSerialPort()
    port.closeFailureCount = 1
    const serial = new FakeWebSerial(port)
    const { connection, adapter } = createConnection(serial)
    await connection.connect()

    await connection.disconnect()
    expect(connection.state.value).toMatchObject({ phase: 'error', operation: 'close-port' })
    expect(connection.canDisconnect.value).toBe(true)
    expect(serial.listenerCount).toBe(1)

    serial.emitDisconnect(port)
    await vi.waitFor(() => expect(connection.state.value.phase).toBe('disconnected'))

    expect(connection.state.value).toMatchObject({ reason: 'device-disconnected' })
    expect(connection.canDisconnect.value).toBe(false)
    expect(serial.listenerCount).toBe(0)
    expect(adapter.resetSession).toHaveBeenCalledTimes(1)
    expect(port.events.filter((event) => event === 'port-close')).toHaveLength(2)
  })

  it('port close失敗を成功扱いせず、解放を再試行できる', async () => {
    const port = new FakeSerialPort()
    port.closeFailureCount = 1
    const { connection } = createConnection(new FakeWebSerial(port))
    await connection.connect()

    await connection.disconnect()
    expect(connection.state.value).toMatchObject({ phase: 'error', operation: 'close-port' })
    expect(connection.canRetry.value).toBe(false)
    expect(connection.canDisconnect.value).toBe(true)

    await connection.disconnect()
    expect(connection.state.value).toMatchObject({ phase: 'disconnected', reason: 'user' })
    expect(port.events.filter((event) => event === 'port-close')).toHaveLength(2)
  })
})
