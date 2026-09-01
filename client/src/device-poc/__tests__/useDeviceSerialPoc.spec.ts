import { describe, expect, it, vi } from 'vitest'
import { effectScope } from 'vue'

import { useDeviceSerialPoc, type SerialPortLike, type WebSerialLike } from '../useDeviceSerialPoc'

const encoder = new TextEncoder()

class FakeSerialPort implements SerialPortLike {
  readonly events: string[] = []
  readonly writes: number[][] = []
  readonly readable: ReadableStream<Uint8Array>
  readonly writable: WritableStream<Uint8Array>
  openedWith: unknown
  isOpened = false
  private controller?: ReadableStreamDefaultController<Uint8Array>

  constructor() {
    this.readable = new ReadableStream({
      start: (controller) => {
        this.controller = controller
      },
      cancel: () => {
        this.events.push('reader-cancel')
      },
    })
    this.writable = new WritableStream({
      write: (bytes: Uint8Array) => {
        this.writes.push([...bytes])
        this.respondToWrite(bytes)
      },
    })
  }

  async open(options: unknown): Promise<void> {
    this.openedWith = options
    this.isOpened = true
    this.events.push('port-open')
  }

  async close(): Promise<void> {
    this.isOpened = false
    this.events.push('port-close')
  }

  getInfo(): { usbVendorId: number; usbProductId: number } {
    return { usbVendorId: 0x2e8a, usbProductId: 0x0005 }
  }

  emit(bytes: Uint8Array): void {
    this.controller?.enqueue(bytes)
  }

  failRead(error: Error): void {
    this.controller?.error(error)
  }

  private respondToWrite(bytes: Uint8Array): void {
    if (bytes[0] === 0x0d && bytes[1] === 0x01) {
      this.emit(encoder.encode('raw REPL; CTRL-B to exit\r\n>'))
      return
    }
    if (bytes[bytes.length - 1] === 0x04) {
      this.emit(encoder.encode('OK'))
      return
    }
    if (bytes.length === 1 && bytes[0] === 0x03) {
      this.emit(new Uint8Array([0x04, 0x04, 0x3e]))
    }
  }
}

class DeferredOpenSerialPort extends FakeSerialPort {
  private readonly openBarrier: Promise<void>
  private resolveOpen!: () => void
  private opening = false

  constructor() {
    super()
    this.openBarrier = new Promise((resolve) => {
      this.resolveOpen = resolve
    })
  }

  override async open(options: unknown): Promise<void> {
    this.openedWith = options
    this.opening = true
    this.events.push('port-open-start')
    await this.openBarrier
    this.opening = false
    this.isOpened = true
    this.events.push('port-open')
  }

  override async close(): Promise<void> {
    if (this.opening) {
      this.events.push('port-close-opening-rejected')
      throw new DOMException('Port is still opening', 'InvalidStateError')
    }
    await super.close()
  }

  finishOpening(): void {
    this.resolveOpen()
  }
}

class OpenRejectingSerialPort extends FakeSerialPort {
  override async open(options: unknown): Promise<void> {
    this.openedWith = options
    this.events.push('port-open-rejected')
    throw new DOMException('Port is already in use', 'NetworkError')
  }

  override async close(): Promise<void> {
    this.events.push('port-close-unopened')
    throw new DOMException('Port is not opened', 'InvalidStateError')
  }
}

class RetryableCloseSerialPort extends FakeSerialPort {
  rejectClose = true

  override async close(): Promise<void> {
    if (this.rejectClose) {
      this.events.push('port-close-rejected')
      throw new DOMException('Port is busy', 'InvalidStateError')
    }
    await super.close()
  }
}

class DeferredFailingCloseSerialPort extends FakeSerialPort {
  private readonly closeBarrier: Promise<void>
  private rejectClose!: (error: Error) => void

  constructor() {
    super()
    this.closeBarrier = new Promise((_, reject) => {
      this.rejectClose = reject
    })
  }

  override async close(): Promise<void> {
    this.events.push('port-close-start')
    await this.closeBarrier
  }

  finishCloseFailure(): void {
    this.rejectClose(new DOMException('Device was removed', 'NetworkError'))
  }
}

class FakeWebSerial implements WebSerialLike {
  readonly requestPort = vi.fn<WebSerialLike['requestPort']>()
  private readonly disconnectListeners = new Set<(event: Event) => void>()

  constructor(port: SerialPortLike) {
    this.requestPort.mockImplementation(async () => port)
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
}

function createSerial(port: FakeSerialPort): FakeWebSerial {
  return new FakeWebSerial(port)
}

describe('useDeviceSerialPoc', () => {
  it('secure contextでWeb Serialがない場合はunsupportedにする', () => {
    const serialPoc = useDeviceSerialPoc({ serial: null, secureContext: true })

    expect(serialPoc.state.value.phase).toBe('unsupported')
    expect(serialPoc.canConnect.value).toBe(false)
  })

  it('pickerのキャンセルをerrorにせずidleへ戻す', async () => {
    const serial: WebSerialLike = {
      requestPort: vi.fn<WebSerialLike['requestPort']>(async () => {
        throw new DOMException('User cancelled', 'NotFoundError')
      }),
    }
    const serialPoc = useDeviceSerialPoc({ serial, secureContext: true })

    await serialPoc.connect()

    expect(serialPoc.state.value).toMatchObject({ phase: 'idle' })
    expect(serialPoc.totalBytes.value).toBe(0)
  })

  it('Picoを選択し、raw REPLからprotocol PoCを起動して停止する', async () => {
    const port = new FakeSerialPort()
    const serial = createSerial(port)
    const serialPoc = useDeviceSerialPoc({
      serial,
      secureContext: true,
      wallClock: () => new Date('2026-08-29T00:00:00.000Z'),
    })

    await serialPoc.connect()

    expect(serial.requestPort).toHaveBeenCalledWith({
      filters: [{ usbVendorId: 0x2e8a, usbProductId: 0x0005 }],
    })
    expect(port.openedWith).toEqual({
      baudRate: 115_200,
      dataBits: 8,
      stopBits: 1,
      parity: 'none',
      flowControl: 'none',
    })
    expect(serialPoc.state.value.phase).toBe('running')
    expect(serialPoc.totalBytes.value).toBeGreaterThan(0)

    port.emit(new Uint8Array([0xe3, 0x81]))
    port.emit(new Uint8Array([0x82]))
    await vi.waitFor(() => expect(serialPoc.decodedPreview.value).toContain('あ'))
    expect(serialPoc.decodedPreview.value).not.toContain('�')

    await serialPoc.stop()

    expect(serialPoc.state.value).toMatchObject({ phase: 'disconnected', incomplete: false })
    expect(port.events).toEqual(['port-open', 'reader-cancel', 'port-close'])
    expect(port.writes[port.writes.length - 1]).toEqual([0x03])
    expect(serialPoc.canDownload.value).toBe(true)
  })

  it('open失敗時は未open portをcloseせず再接続可能にする', async () => {
    const port = new OpenRejectingSerialPort()
    const serialPoc = useDeviceSerialPoc({
      serial: createSerial(port),
      secureContext: true,
    })

    await serialPoc.connect()

    expect(serialPoc.state.value).toMatchObject({ phase: 'error', operation: 'open-port' })
    expect(serialPoc.canConnect.value).toBe(true)
    expect(serialPoc.canStop.value).toBe(false)
    expect(port.events).toEqual(['port-open-rejected'])
  })

  it('route破棄中にportのopenが完了してもstale sessionを閉じる', async () => {
    const port = new DeferredOpenSerialPort()
    const serial = createSerial(port)
    const scope = effectScope()
    const serialPoc = scope.run(() => useDeviceSerialPoc({ serial, secureContext: true }))!

    const connectPromise = serialPoc.connect()
    await vi.waitFor(() => expect(serialPoc.state.value.phase).toBe('opening'))

    scope.stop()
    port.finishOpening()
    await connectPromise
    await vi.waitFor(() => expect(port.isOpened).toBe(false))

    expect(port.writes).toEqual([])
    expect(port.events).toEqual(['port-open-start', 'port-open', 'port-close'])
  })

  it('route破棄後にport pickerが解決してもportをopenしない', async () => {
    const port = new FakeSerialPort()
    let resolvePort!: (port: SerialPortLike) => void
    const selectedPort = new Promise<SerialPortLike>((resolve) => {
      resolvePort = resolve
    })
    const serial: WebSerialLike = {
      requestPort: vi.fn<WebSerialLike['requestPort']>(() => selectedPort),
    }
    const scope = effectScope()
    const serialPoc = scope.run(() => useDeviceSerialPoc({ serial, secureContext: true }))!

    const connectPromise = serialPoc.connect()
    await vi.waitFor(() => expect(serialPoc.state.value.phase).toBe('selecting'))
    scope.stop()
    resolvePort(port)
    await connectPromise

    expect(port.openedWith).toBeUndefined()
    expect(port.events).toEqual([])
  })

  it('別portのdisconnect eventでは現在のcaptureを停止しない', async () => {
    const port = new FakeSerialPort()
    const otherPort = new FakeSerialPort()
    const serial = createSerial(port)
    const serialPoc = useDeviceSerialPoc({ serial, secureContext: true })
    await serialPoc.connect()

    serial.emitDisconnect(otherPort)
    await Promise.resolve()

    expect(serialPoc.state.value.phase).toBe('running')
    await serialPoc.stop()
  })

  it('現在のportのdisconnect eventではpartial captureを保持する', async () => {
    const port = new FakeSerialPort()
    const serial = createSerial(port)
    const serialPoc = useDeviceSerialPoc({ serial, secureContext: true })
    await serialPoc.connect()
    const capturedBytes = serialPoc.totalBytes.value

    serial.emitDisconnect(port)
    await vi.waitFor(() => expect(serialPoc.state.value.phase).toBe('disconnected'))

    expect(serialPoc.state.value).toMatchObject({ incomplete: true })
    expect(serialPoc.totalBytes.value).toBe(capturedBytes)
  })

  it('read errorのcleanup中にUSB切断された場合はclose失敗へ固着しない', async () => {
    const port = new DeferredFailingCloseSerialPort()
    const serial = createSerial(port)
    const serialPoc = useDeviceSerialPoc({ serial, secureContext: true })
    await serialPoc.connect()

    port.failRead(new DOMException('Device was removed', 'NetworkError'))
    await vi.waitFor(() => expect(port.events).toContain('port-close-start'))
    serial.emitDisconnect(port)
    port.finishCloseFailure()
    await vi.waitFor(() => expect(serialPoc.state.value.phase).toBe('disconnected'))

    expect(serialPoc.state.value).toMatchObject({ incomplete: true })
    expect(serialPoc.canConnect.value).toBe(true)
    expect(serialPoc.canStop.value).toBe(false)
  })

  it('close失敗を成功扱いせず、同じportで再試行できる', async () => {
    const port = new RetryableCloseSerialPort()
    const serialPoc = useDeviceSerialPoc({
      serial: createSerial(port),
      secureContext: true,
    })
    await serialPoc.connect()

    await serialPoc.stop()

    expect(serialPoc.state.value).toMatchObject({ phase: 'error', operation: 'close-port' })
    expect(serialPoc.canConnect.value).toBe(false)
    expect(serialPoc.canStop.value).toBe(true)

    const digest = vi
      .spyOn(globalThis.crypto.subtle, 'digest')
      .mockRejectedValueOnce(new Error('hash failed'))
    await serialPoc.downloadRawCapture()
    digest.mockRestore()

    expect(serialPoc.state.value).toMatchObject({ phase: 'error', operation: 'export-capture' })
    expect(serialPoc.canStop.value).toBe(true)

    port.rejectClose = false
    await serialPoc.stop()

    expect(serialPoc.state.value).toMatchObject({ phase: 'disconnected' })
    expect(serialPoc.canConnect.value).toBe(true)
  })
})
