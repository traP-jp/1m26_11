import { computed, getCurrentScope, onScopeDispose, readonly, ref, shallowRef } from 'vue'

import { createRawCaptureArtifacts, downloadBlob } from './capture'
import {
  enterMicroPythonRawRepl,
  launchUploadedScript,
  SerialByteBuffer,
  stopUploadedScript,
  type RawReplChannel,
} from './microPythonRawRepl'
import {
  SERIAL_POC_CAPTURE_LIMIT_BYTES,
  SERIAL_POC_OPEN_OPTIONS,
  SERIAL_POC_PORT_FILTER,
  SERIAL_POC_SCRIPT_PATH,
  type RawSerialChunk,
  type SerialConnectionEndReason,
  type SerialConnectionRecord,
  type SerialPocOperation,
  type SerialPocState,
} from './types'

interface SerialPortRequestOptionsLike {
  filters: readonly [typeof SERIAL_POC_PORT_FILTER]
}

export interface SerialPortLike {
  readonly readable: ReadableStream<Uint8Array> | null
  readonly writable: WritableStream<Uint8Array> | null
  open(options: typeof SERIAL_POC_OPEN_OPTIONS): Promise<void>
  close(): Promise<void>
  getInfo?(): { usbVendorId?: number; usbProductId?: number }
}

export interface WebSerialLike {
  requestPort(options: SerialPortRequestOptionsLike): Promise<SerialPortLike>
  addEventListener?(type: 'disconnect', listener: (event: Event) => void): void
  removeEventListener?(type: 'disconnect', listener: (event: Event) => void): void
}

interface UseDeviceSerialPocOptions {
  serial?: WebSerialLike | null
  secureContext?: boolean
  captureLimitBytes?: number
  now?: () => number
  wallClock?: () => Date
  origin?: string
  userAgent?: string
}

interface ReadLoopResult {
  reason: 'cancelled' | 'ended' | 'error'
  error?: unknown
}

function getBrowserSerial(): WebSerialLike | undefined {
  if (typeof navigator === 'undefined') return undefined
  return (navigator as Navigator & { serial?: WebSerialLike }).serial
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  return String(error)
}

function isPortPickerCancellation(error: unknown): boolean {
  return error instanceof DOMException && error.name === 'NotFoundError'
}

function isBusyState(state: SerialPocState): boolean {
  return ['selecting', 'opening', 'syncing-raw-repl', 'launching', 'running', 'stopping'].includes(
    state.phase,
  )
}

export function useDeviceSerialPoc(options: UseDeviceSerialPocOptions = {}) {
  const serial = options.serial === undefined ? getBrowserSerial() : (options.serial ?? undefined)
  const secureContext = options.secureContext ?? globalThis.isSecureContext === true
  const captureLimitBytes = options.captureLimitBytes ?? SERIAL_POC_CAPTURE_LIMIT_BYTES
  const now = options.now ?? (() => performance.now())
  const wallClock = options.wallClock ?? (() => new Date())
  const origin = options.origin ?? (typeof location === 'undefined' ? '' : location.origin)
  const userAgent =
    options.userAgent ?? (typeof navigator === 'undefined' ? '' : navigator.userAgent)

  const initialState: SerialPocState = !secureContext
    ? {
        phase: 'unsupported',
        message: 'Web SerialにはHTTPSまたはlocalhostのsecure contextが必要です。',
      }
    : serial
      ? { phase: 'idle', message: 'Picoは未接続です。' }
      : {
          phase: 'unsupported',
          message: 'このブラウザはWeb Serial APIに対応していません。',
        }

  const state = ref<SerialPocState>(initialState)
  const chunks = shallowRef<RawSerialChunk[]>([])
  const connections = shallowRef<SerialConnectionRecord[]>([])
  const decodedPreview = ref('')
  const totalBytes = ref(0)
  const capturedAt = ref<string>()
  const lastExportSha256 = ref<string>()
  const portActive = ref(false)
  const closeInProgress = ref(false)

  let captureStartedMonotonic = 0
  let generation = 0
  let activeConnectionId: number | undefined
  let currentPort: SerialPortLike | undefined
  let currentPortOpened = false
  let currentPortPhysicallyDisconnected = false
  let currentReader: ReadableStreamDefaultReader<Uint8Array> | undefined
  let currentBuffer: SerialByteBuffer | undefined
  let currentDecoder: TextDecoder | undefined
  let readLoopPromise: Promise<ReadLoopResult> | undefined
  let currentWritePromise: Promise<void> | undefined
  let closePromise: Promise<void> | undefined
  let closingRequested = false
  let commandSubmitted = false
  let disconnectListener: ((event: Event) => void) | undefined

  const canConnect = computed(
    () =>
      Boolean(serial) &&
      secureContext &&
      !portActive.value &&
      !closeInProgress.value &&
      !isBusyState(state.value) &&
      totalBytes.value < captureLimitBytes,
  )
  const canStop = computed(() => portActive.value && !closeInProgress.value)
  const canClear = computed(() => !portActive.value && chunks.value.length > 0)
  const canDownload = computed(() => chunks.value.length > 0 && !isBusyState(state.value))

  function ensureCaptureStarted(): void {
    if (capturedAt.value) return
    capturedAt.value = wallClock().toISOString()
    captureStartedMonotonic = now()
  }

  function appendChunk(
    connectionId: number,
    bytes: Uint8Array,
    buffer: SerialByteBuffer,
    decoder: TextDecoder,
  ): void {
    const copy = bytes.slice()
    ensureCaptureStarted()
    const chunk: RawSerialChunk = {
      sequence: chunks.value.length + 1,
      connectionId,
      offset: totalBytes.value,
      receivedElapsedMs: now() - captureStartedMonotonic,
      bytes: copy,
    }

    chunks.value = [...chunks.value, chunk]
    totalBytes.value += copy.length
    buffer.append(copy)
    decodedPreview.value += decoder.decode(copy, { stream: true })

    if (totalBytes.value >= captureLimitBytes && !closingRequested) {
      queueMicrotask(() => {
        void closeCurrentSession('capture-limit', commandSubmitted)
      })
    }
  }

  function updateConnection(connectionId: number, update: Partial<SerialConnectionRecord>): void {
    connections.value = connections.value.map((connection) =>
      connection.id === connectionId ? { ...connection, ...update } : connection,
    )
  }

  async function writeToPort(port: SerialPortLike, bytes: Uint8Array): Promise<void> {
    const writable = port.writable
    if (!writable) throw new Error('Picoのwritable streamを取得できませんでした')

    const writePromise = (async () => {
      const writer = writable.getWriter()
      try {
        await writer.write(bytes)
      } finally {
        writer.releaseLock()
      }
    })()
    currentWritePromise = writePromise

    try {
      await writePromise
    } finally {
      if (currentWritePromise === writePromise) currentWritePromise = undefined
    }
  }

  function createRawReplChannel(port: SerialPortLike, received: SerialByteBuffer): RawReplChannel {
    return { received, write: (bytes) => writeToPort(port, bytes) }
  }

  async function runReadLoop(
    reader: ReadableStreamDefaultReader<Uint8Array>,
    buffer: SerialByteBuffer,
    decoder: TextDecoder,
    connectionId: number,
  ): Promise<ReadLoopResult> {
    try {
      while (true) {
        const result = await reader.read()
        if (result.done) {
          buffer.close(new Error('Serial readable stream ended'))
          return { reason: closingRequested ? 'cancelled' : 'ended' }
        }
        if (result.value.length > 0) appendChunk(connectionId, result.value, buffer, decoder)
      }
    } catch (error) {
      buffer.close(error instanceof Error ? error : new Error(String(error)))
      return closingRequested ? { reason: 'cancelled' } : { reason: 'error', error }
    } finally {
      reader.releaseLock()
      if (currentReader === reader) currentReader = undefined
    }
  }

  function observeUnexpectedReadEnd(
    promise: Promise<ReadLoopResult>,
    sessionGeneration: number,
  ): void {
    void promise.then((result) => {
      if (sessionGeneration !== generation || closingRequested || result.reason === 'cancelled') {
        return
      }

      if (result.reason === 'error') {
        void closeCurrentSession('read-error', false, {
          phase: 'error',
          operation: 'read',
          message: `シリアル読取りに失敗しました: ${errorMessage(result.error)}`,
        })
        return
      }

      void closeCurrentSession('stream-ended', false, {
        phase: 'disconnected',
        message: 'シリアルstreamが終了しました。受信済みdataは保持されています。',
        incomplete: true,
      })
    })
  }

  function registerDisconnectListener(): void {
    if (!serial?.addEventListener) return

    disconnectListener = (event) => {
      const serialEvent = event as Event & { port?: SerialPortLike }
      const eventPort = serialEvent.port ?? serialEvent.target
      if (!currentPort || eventPort !== currentPort) return

      currentPortPhysicallyDisconnected = true
      if (closingRequested) return

      void closeCurrentSession('device-disconnected', false, {
        phase: 'disconnected',
        message: 'USB deviceが切断されました。受信済みdataは保持されています。',
        incomplete: true,
      })
    }
    serial.addEventListener('disconnect', disconnectListener)
  }

  function unregisterDisconnectListener(): void {
    if (disconnectListener) serial?.removeEventListener?.('disconnect', disconnectListener)
    disconnectListener = undefined
  }

  async function closeCurrentSession(
    reason: SerialConnectionEndReason,
    stopScript: boolean,
    finalState?: SerialPocState,
  ): Promise<void> {
    if (!currentPort && !readLoopPromise) return
    if (closePromise) return closePromise

    closingRequested = true
    closeInProgress.value = true
    generation += 1
    const cleanupGeneration = generation
    const portToClose = currentPort
    const portWasOpened = currentPortOpened
    const readerToCancel = currentReader
    const bufferToClose = currentBuffer
    const decoderToFlush = currentDecoder
    const loopToFinish = readLoopPromise
    const writeToFinish = currentWritePromise
    const connectionId = activeConnectionId

    const cleanupPromise = (async () => {
      let stopConfirmed = false
      let closeError: unknown

      try {
        await writeToFinish
      } catch {
        // The setup error is reported by the connect flow; cleanup still continues.
      }

      if (stopScript && portToClose && bufferToClose) {
        state.value = {
          phase: 'stopping',
          message: `${SERIAL_POC_SCRIPT_PATH}を停止しています。`,
        }
        if (connectionId !== undefined) {
          updateConnection(connectionId, { stopRequestedOffset: totalBytes.value })
        }
        try {
          stopConfirmed = await stopUploadedScript(createRawReplChannel(portToClose, bufferToClose))
          if (stopConfirmed && connectionId !== undefined) {
            updateConnection(connectionId, {
              stopCompletedObservedOffset: totalBytes.value,
            })
          }
        } catch {
          stopConfirmed = false
        }
      }

      try {
        await readerToCancel?.cancel()
      } catch {
        // A physically disconnected reader can reject cancel().
      }

      try {
        await loopToFinish
      } catch {
        // runReadLoop converts read failures to a result, but cleanup remains best-effort.
      }

      decodedPreview.value += decoderToFlush?.decode() ?? ''

      if (portWasOpened) {
        try {
          await portToClose?.close()
        } catch (error) {
          closeError = error
        }
      }

      bufferToClose?.close()

      const physicallyDisconnected =
        reason === 'device-disconnected' ||
        (currentPort === portToClose && currentPortPhysicallyDisconnected)
      const effectiveEndReason = physicallyDisconnected ? 'device-disconnected' : reason

      if (connectionId !== undefined) {
        updateConnection(connectionId, {
          endedAt: wallClock().toISOString(),
          endedOffset: totalBytes.value,
          endReason: effectiveEndReason,
          stopConfirmed,
        })
      }

      const retainPortForCloseRetry = Boolean(closeError) && !physicallyDisconnected
      if (!retainPortForCloseRetry) unregisterDisconnectListener()
      if (currentPort === portToClose && !retainPortForCloseRetry) {
        currentPort = undefined
        currentPortOpened = false
        currentPortPhysicallyDisconnected = false
        portActive.value = false
      }
      if (currentReader === readerToCancel) currentReader = undefined
      if (currentBuffer === bufferToClose) currentBuffer = undefined
      if (currentDecoder === decoderToFlush) currentDecoder = undefined
      if (readLoopPromise === loopToFinish) readLoopPromise = undefined
      if (activeConnectionId === connectionId) activeConnectionId = undefined
      currentWritePromise = undefined
      commandSubmitted = false

      if (cleanupGeneration !== generation) return
      if (physicallyDisconnected) {
        state.value = {
          phase: 'disconnected',
          message: 'USB deviceが切断されました。受信済みdataは保持されています。',
          incomplete: true,
        }
      } else if (closeError) {
        state.value = {
          phase: 'error',
          operation: 'close-port',
          message: `ポートを解放できませんでした。Stopで再試行してください: ${errorMessage(closeError)}`,
        }
      } else if (finalState) {
        state.value = finalState
      } else if (reason === 'user') {
        state.value = {
          phase: 'disconnected',
          message: !stopScript
            ? 'ポートを解放しました。'
            : stopConfirmed
              ? `${SERIAL_POC_SCRIPT_PATH}を停止し、ポートを解放しました。`
              : 'ポートを解放しましたが、スクリプトの停止応答は確認できませんでした。',
          incomplete: stopScript && !stopConfirmed,
        }
      } else if (reason === 'capture-limit') {
        state.value = {
          phase: 'disconnected',
          message:
            `capture停止目安 ${captureLimitBytes.toLocaleString()} byteに達したため停止しました。` +
            '受信済みchunkと停止応答はすべて保持しています。',
          incomplete: true,
        }
      }
    })()
    closePromise = cleanupPromise

    try {
      await cleanupPromise
    } finally {
      if (closePromise === cleanupPromise) {
        closePromise = undefined
        closingRequested = false
        closeInProgress.value = false
      }
    }
  }

  async function connect(): Promise<void> {
    if (!serial || !canConnect.value) return

    generation += 1
    const sessionGeneration = generation
    closingRequested = false
    commandSubmitted = false
    let operation: SerialPocOperation = 'request-port'
    let selectedPort: SerialPortLike | undefined
    let selectedPortOpened = false

    try {
      state.value = { phase: 'selecting', message: 'Picoを選択してください。' }
      selectedPort = await serial.requestPort({ filters: [SERIAL_POC_PORT_FILTER] })
      if (sessionGeneration !== generation) return

      currentPort = selectedPort
      currentPortOpened = false
      currentPortPhysicallyDisconnected = false
      portActive.value = true
      operation = 'open-port'
      state.value = {
        phase: 'opening',
        message: 'PicoをPoC用設定 115200 baudで開いています。',
      }
      await selectedPort.open(SERIAL_POC_OPEN_OPTIONS)
      selectedPortOpened = true
      if (currentPort === selectedPort) currentPortOpened = true
      if (sessionGeneration !== generation || closingRequested) {
        if (closePromise) await closePromise
        try {
          await selectedPort.close()
        } catch {
          // A concurrent cleanup may already have closed the selected port.
        }
        if (currentPort === selectedPort) {
          currentPort = undefined
          currentPortOpened = false
          currentPortPhysicallyDisconnected = false
          portActive.value = false
        }
        return
      }

      const readable = selectedPort.readable
      if (!readable || !selectedPort.writable) {
        throw new Error('Picoのreadable/writable streamを取得できませんでした')
      }

      ensureCaptureStarted()
      const connectionId = connections.value.length + 1
      activeConnectionId = connectionId
      const portInfo = selectedPort.getInfo?.() ?? {}
      connections.value = [
        ...connections.value,
        {
          id: connectionId,
          startedAt: wallClock().toISOString(),
          startedOffset: totalBytes.value,
          scriptPath: SERIAL_POC_SCRIPT_PATH,
          usbVendorId: portInfo.usbVendorId,
          usbProductId: portInfo.usbProductId,
        },
      ]

      currentBuffer = new SerialByteBuffer()
      currentDecoder = new TextDecoder()
      currentReader = readable.getReader()
      readLoopPromise = runReadLoop(currentReader, currentBuffer, currentDecoder, connectionId)
      observeUnexpectedReadEnd(readLoopPromise, sessionGeneration)
      registerDisconnectListener()

      operation = 'raw-repl-sync'
      state.value = {
        phase: 'syncing-raw-repl',
        message: 'MicroPython raw REPLへ切り替えています。',
      }
      const channel = createRawReplChannel(selectedPort, currentBuffer)
      await enterMicroPythonRawRepl(channel)
      if (sessionGeneration !== generation || closingRequested) return
      updateConnection(connectionId, { rawReplReadyObservedOffset: totalBytes.value })

      operation = 'launch-script'
      commandSubmitted = true
      updateConnection(connectionId, { scriptLaunchRequestedOffset: totalBytes.value })
      state.value = {
        phase: 'launching',
        message: `Upload済み ${SERIAL_POC_SCRIPT_PATH} を起動しています。`,
      }
      await launchUploadedScript(channel)
      if (sessionGeneration !== generation || closingRequested) return
      updateConnection(connectionId, { scriptActiveObservedOffset: totalBytes.value })

      state.value = {
        phase: 'running',
        message: 'raw byteをcapture中です。物理スイッチを操作できます。',
      }
    } catch (error) {
      if (sessionGeneration !== generation || closingRequested) return
      if (operation === 'request-port' && isPortPickerCancellation(error)) {
        state.value = { phase: 'idle', message: 'ポート選択をキャンセルしました。' }
        return
      }

      const failureState: SerialPocState = {
        phase: 'error',
        operation,
        message: `${errorMessage(error)}。受信済みdataは保持されています。`,
      }
      if (operation === 'open-port' && selectedPort && !selectedPortOpened) {
        if (currentPort === selectedPort) {
          currentPort = undefined
          currentPortOpened = false
          currentPortPhysicallyDisconnected = false
          portActive.value = false
        }
        state.value = failureState
        return
      }
      if (currentPort) {
        await closeCurrentSession('setup-error', commandSubmitted, failureState)
      } else {
        state.value = failureState
      }
    }
  }

  async function stop(): Promise<void> {
    if (!currentPort) return
    await closeCurrentSession('user', commandSubmitted)
  }

  function clearCapture(): void {
    if (currentPort || closePromise) return
    chunks.value = []
    connections.value = []
    decodedPreview.value = ''
    totalBytes.value = 0
    capturedAt.value = undefined
    lastExportSha256.value = undefined
    state.value = serial
      ? { phase: 'idle', message: 'Captureを消去しました。Picoは未接続です。' }
      : initialState
  }

  async function downloadCapturePart(part: 'raw' | 'metadata'): Promise<void> {
    if (!capturedAt.value || chunks.value.length === 0) return

    try {
      const artifacts = await createRawCaptureArtifacts({
        chunks: chunks.value,
        connections: connections.value,
        capturedAt: capturedAt.value,
        origin,
        userAgent,
        secureContext,
        captureLimitBytes,
      })
      if (part === 'raw') {
        downloadBlob(artifacts.raw, `${artifacts.baseName}.bin`)
      } else {
        downloadBlob(artifacts.metadata, `${artifacts.baseName}.json`)
      }
      lastExportSha256.value = artifacts.sha256
    } catch (error) {
      state.value = {
        phase: 'error',
        operation: 'export-capture',
        message: `CaptureのDownloadに失敗しました: ${errorMessage(error)}`,
      }
    }
  }

  const downloadRawCapture = (): Promise<void> => downloadCapturePart('raw')
  const downloadCaptureMetadata = (): Promise<void> => downloadCapturePart('metadata')

  if (getCurrentScope()) {
    onScopeDispose(() => {
      if (currentPort || readLoopPromise) {
        void closeCurrentSession('user', commandSubmitted)
      } else {
        generation += 1
      }
    })
  }

  return {
    state: readonly(state),
    chunks: readonly(chunks),
    connections: readonly(connections),
    decodedPreview: readonly(decodedPreview),
    totalBytes: readonly(totalBytes),
    capturedAt: readonly(capturedAt),
    lastExportSha256: readonly(lastExportSha256),
    captureLimitBytes,
    canConnect,
    canStop,
    canClear,
    canDownload,
    connect,
    stop,
    clearCapture,
    downloadRawCapture,
    downloadCaptureMetadata,
  }
}
