import { computed, getCurrentScope, onScopeDispose, readonly, ref } from 'vue'

export const WEB_SERIAL_OPEN_OPTIONS = {
  baudRate: 115_200,
  dataBits: 8,
  stopBits: 1,
  parity: 'none',
  flowControl: 'none',
} as const

export const RASPBERRY_PI_PICO_PORT_FILTER = {
  usbVendorId: 0x2e8a,
  usbProductId: 0x0005,
} as const

interface SerialPortRequestOptionsLike {
  filters: readonly [typeof RASPBERRY_PI_PICO_PORT_FILTER]
}

export interface SerialInputAdapterLike {
  pushChunk(chunk: Uint8Array): void
  resetSession(): void
}

export interface SerialReaderLike {
  read(): Promise<ReadableStreamReadResult<Uint8Array>>
  cancel(): Promise<void>
  releaseLock(): void
}

export interface SerialReadableLike {
  getReader(): SerialReaderLike
}

export interface SerialPortLike {
  readonly readable: SerialReadableLike | null
  open(options: typeof WEB_SERIAL_OPEN_OPTIONS): Promise<void>
  close(): Promise<void>
}

export interface WebSerialLike {
  requestPort(options: SerialPortRequestOptionsLike): Promise<SerialPortLike>
  addEventListener?(type: 'disconnect', listener: (event: Event) => void): void
  removeEventListener?(type: 'disconnect', listener: (event: Event) => void): void
}

export type SerialConnectionErrorOperation =
  'request-port' | 'open-port' | 'read' | 'reconnect' | 'close-port'

export type SerialConnectionState =
  | {
      phase: 'unsupported'
      reason: 'insecure-context' | 'api-unavailable'
      message: string
    }
  | { phase: 'idle'; message: string }
  | { phase: 'requesting'; attempt: 'connect' | 'retry'; message: string }
  | { phase: 'connected'; message: string }
  | {
      phase: 'disconnected'
      reason: 'user' | 'device-disconnected' | 'stream-ended'
      message: string
    }
  | {
      phase: 'error'
      operation: SerialConnectionErrorOperation
      message: string
    }

export interface UseWebSerialConnectionOptions {
  adapter: SerialInputAdapterLike
  serial?: WebSerialLike | null
  secureContext?: boolean
}

interface ReadLoopResult {
  reason: 'cancelled' | 'ended' | 'error'
  error?: unknown
}

type ConnectionAttempt = 'connect' | 'retry'
type ConnectionEndReason = 'user' | 'device-disconnected' | 'stream-ended' | 'read-error'

function getBrowserSerial(): WebSerialLike | undefined {
  if (typeof navigator === 'undefined') return undefined
  return (navigator as Navigator & { serial?: WebSerialLike }).serial
}

function errorMessage(error: unknown): string {
  if (error instanceof Error) return error.message
  return String(error)
}

export function useWebSerialConnection(options: UseWebSerialConnectionOptions) {
  const serial = options.serial === undefined ? getBrowserSerial() : (options.serial ?? undefined)
  const secureContext = options.secureContext ?? globalThis.isSecureContext === true
  const initialState: SerialConnectionState = !secureContext
    ? {
        phase: 'unsupported',
        reason: 'insecure-context',
        message: 'Web SerialにはHTTPSまたはlocalhostのsecure contextが必要です。',
      }
    : serial
      ? { phase: 'idle', message: 'Serial deviceは未接続です。' }
      : {
          phase: 'unsupported',
          reason: 'api-unavailable',
          message: 'このブラウザはWeb Serial APIに対応していません。',
        }

  const state = ref<SerialConnectionState>(initialState)
  const connecting = ref(false)
  const cleaningUp = ref(false)
  const ownsPort = ref(false)

  let generation = 0
  const disposed = ref(false)
  let currentPort: SerialPortLike | undefined
  let currentReader: SerialReaderLike | undefined
  let readLoopPromise: Promise<ReadLoopResult> | undefined
  let connectionPromise: Promise<void> | undefined
  let cleanupPromise: Promise<void> | undefined
  let currentPortPhysicallyDisconnected = false
  let disconnectListener: ((event: Event) => void) | undefined

  const busy = computed(() => connecting.value || cleaningUp.value)
  const canConnect = computed(
    () =>
      state.value.phase === 'idle' &&
      !disposed.value &&
      !busy.value &&
      !ownsPort.value &&
      Boolean(serial) &&
      secureContext,
  )
  const canRetry = computed(
    () =>
      (state.value.phase === 'disconnected' || state.value.phase === 'error') &&
      !disposed.value &&
      !busy.value &&
      !ownsPort.value &&
      Boolean(serial) &&
      secureContext,
  )
  const canDisconnect = computed(() => !disposed.value && ownsPort.value && !cleaningUp.value)

  function registerDisconnectListener(): void {
    if (disconnectListener || !serial?.addEventListener) return

    disconnectListener = (event) => {
      const serialEvent = event as Event & { port?: SerialPortLike }
      const disconnectedPort = serialEvent.port ?? serialEvent.target
      if (!currentPort || disconnectedPort !== currentPort) return

      currentPortPhysicallyDisconnected = true
      void closeCurrentConnection('device-disconnected')
    }
    serial.addEventListener('disconnect', disconnectListener)
  }

  function unregisterDisconnectListener(): void {
    if (disconnectListener) serial?.removeEventListener?.('disconnect', disconnectListener)
    disconnectListener = undefined
  }

  async function runReadLoop(
    reader: SerialReaderLike,
    sessionGeneration: number,
  ): Promise<ReadLoopResult> {
    try {
      while (true) {
        const result = await reader.read()
        if (result.done) {
          return { reason: sessionGeneration === generation ? 'ended' : 'cancelled' }
        }
        if (sessionGeneration !== generation || disposed.value) return { reason: 'cancelled' }
        if (result.value.length > 0) options.adapter.pushChunk(result.value)
      }
    } catch (error) {
      return sessionGeneration === generation ? { reason: 'error', error } : { reason: 'cancelled' }
    } finally {
      reader.releaseLock()
      if (currentReader === reader) currentReader = undefined
    }
  }

  function observeReadLoop(promise: Promise<ReadLoopResult>, sessionGeneration: number): void {
    void promise.then((result) => {
      if (sessionGeneration !== generation || result.reason === 'cancelled') return

      if (result.reason === 'error') {
        void closeCurrentConnection('read-error', {
          phase: 'error',
          operation: 'read',
          message: `Serialの読取りに失敗しました: ${errorMessage(result.error)}`,
        })
        return
      }

      void closeCurrentConnection('stream-ended')
    })
  }

  async function closeCurrentConnection(
    reason: ConnectionEndReason,
    finalState?: SerialConnectionState,
  ): Promise<void> {
    if (cleanupPromise) return cleanupPromise
    if (!currentPort && !currentReader && !readLoopPromise) return

    generation += 1
    cleaningUp.value = true
    const portToClose = currentPort
    const readerToCancel = currentReader
    const loopToFinish = readLoopPromise
    const shouldResetAdapter = Boolean(readerToCancel || loopToFinish)

    const cleanup = (async () => {
      let closeError: unknown

      try {
        await readerToCancel?.cancel()
      } catch {
        // A reader for a physically disconnected device can reject cancellation.
      }

      try {
        await loopToFinish
      } catch {
        // runReadLoop returns failures, but cleanup must remain best-effort.
      }

      if (readerToCancel && !loopToFinish) readerToCancel.releaseLock()
      if (shouldResetAdapter) options.adapter.resetSession()

      try {
        await portToClose?.close()
      } catch (error) {
        closeError = error
      }

      if (closeError && disposed.value && portToClose) {
        try {
          await portToClose.close()
          closeError = undefined
        } catch {
          // There is no UI after disposal; release local ownership after a final best-effort retry.
        }
      }

      const physicallyDisconnected =
        reason === 'device-disconnected' ||
        (currentPort === portToClose && currentPortPhysicallyDisconnected)
      const retainPortForCloseRetry =
        Boolean(closeError) && !physicallyDisconnected && !disposed.value

      if (retainPortForCloseRetry) {
        registerDisconnectListener()
      } else {
        unregisterDisconnectListener()
      }
      if (currentPort === portToClose && !retainPortForCloseRetry) {
        currentPort = undefined
        currentPortPhysicallyDisconnected = false
        ownsPort.value = false
      }
      if (currentReader === readerToCancel) currentReader = undefined
      if (readLoopPromise === loopToFinish) readLoopPromise = undefined

      if (disposed.value) return
      if (physicallyDisconnected) {
        state.value = {
          phase: 'disconnected',
          reason: 'device-disconnected',
          message: 'Serial deviceが切断されました。再接続または代替入力を選択してください。',
        }
      } else if (closeError) {
        state.value = {
          phase: 'error',
          operation: 'close-port',
          message: `Serial portを解放できませんでした: ${errorMessage(closeError)}`,
        }
      } else if (finalState) {
        state.value = finalState
      } else if (reason === 'stream-ended') {
        state.value = {
          phase: 'disconnected',
          reason: 'stream-ended',
          message: 'Serialの読取りが終了しました。再接続または代替入力を選択してください。',
        }
      } else {
        state.value = {
          phase: 'disconnected',
          reason: 'user',
          message: 'Serial portを解放しました。',
        }
      }
    })()
    cleanupPromise = cleanup

    try {
      await cleanup
    } finally {
      if (cleanupPromise === cleanup) {
        cleanupPromise = undefined
        cleaningUp.value = false
      }
    }
  }

  function connectionFailureState(
    attempt: ConnectionAttempt,
    operation: 'request-port' | 'open-port' | 'read',
    error: unknown,
  ): SerialConnectionState {
    if (attempt === 'retry') {
      return {
        phase: 'error',
        operation: 'reconnect',
        message: `Serialの再接続に失敗しました: ${errorMessage(error)}`,
      }
    }

    return {
      phase: 'error',
      operation,
      message:
        operation === 'request-port'
          ? `Serial portの選択が拒否またはキャンセルされました: ${errorMessage(error)}`
          : operation === 'open-port'
            ? `Serial portを開けませんでした: ${errorMessage(error)}`
            : `Serialの読取りを開始できませんでした: ${errorMessage(error)}`,
    }
  }

  async function closeStaleOpenedPort(port: SerialPortLike): Promise<void> {
    try {
      await port.close()
    } catch (error) {
      if (disposed.value) {
        try {
          await port.close()
          return
        } catch {
          // The component no longer has a retry path, so do not retain its global listener.
        }
        return
      }
      currentPort = port
      currentPortPhysicallyDisconnected = false
      ownsPort.value = true
      registerDisconnectListener()
      if (disposed.value) return
      state.value = {
        phase: 'error',
        operation: 'close-port',
        message: `不要になったSerial portを解放できませんでした: ${errorMessage(error)}`,
      }
    }
  }

  async function runConnection(attempt: ConnectionAttempt): Promise<void> {
    if (!serial || disposed.value) return

    connecting.value = true
    generation += 1
    const sessionGeneration = generation
    let operation: 'request-port' | 'open-port' | 'read' = 'request-port'
    let selectedPort: SerialPortLike | undefined
    let selectedPortOpened = false

    try {
      state.value = {
        phase: 'requesting',
        attempt,
        message:
          attempt === 'retry'
            ? 'Serial deviceを再選択しています。'
            : 'Serial deviceを選択してください。',
      }

      selectedPort = await serial.requestPort({ filters: [RASPBERRY_PI_PICO_PORT_FILTER] })
      if (sessionGeneration !== generation || disposed.value) return

      operation = 'open-port'
      await selectedPort.open(WEB_SERIAL_OPEN_OPTIONS)
      selectedPortOpened = true
      if (sessionGeneration !== generation || disposed.value) {
        await closeStaleOpenedPort(selectedPort)
        return
      }

      operation = 'read'
      const readable = selectedPort.readable
      if (!readable) throw new Error('Serial readable streamを取得できませんでした')

      const reader = readable.getReader()
      currentPort = selectedPort
      currentPortPhysicallyDisconnected = false
      ownsPort.value = true
      currentReader = reader
      readLoopPromise = runReadLoop(reader, sessionGeneration)
      registerDisconnectListener()
      state.value = { phase: 'connected', message: 'Serial deviceから入力を読取り中です。' }
      observeReadLoop(readLoopPromise, sessionGeneration)
    } catch (error) {
      if (sessionGeneration !== generation || disposed.value) return
      const failureState = connectionFailureState(attempt, operation, error)

      if (selectedPortOpened && currentPort === selectedPort) {
        await closeCurrentConnection('user', failureState)
      } else if (selectedPortOpened && selectedPort) {
        try {
          await selectedPort.close()
          if (sessionGeneration !== generation || disposed.value) return
          state.value = failureState
        } catch (closeError) {
          if (disposed.value) {
            try {
              await selectedPort.close()
            } catch {
              // The component no longer has a retry path; do not retain its listener.
            }
            return
          }
          if (sessionGeneration !== generation) {
            await closeStaleOpenedPort(selectedPort)
            return
          }
          currentPort = selectedPort
          currentPortPhysicallyDisconnected = false
          ownsPort.value = true
          registerDisconnectListener()
          state.value = {
            phase: 'error',
            operation: 'close-port',
            message: `Serial portを解放できませんでした: ${errorMessage(closeError)}`,
          }
        }
      } else {
        state.value = failureState
      }
    } finally {
      if (sessionGeneration === generation || !currentPort) connecting.value = false
    }
  }

  function startConnection(attempt: ConnectionAttempt): Promise<void> {
    if (disposed.value) return Promise.resolve()
    if (connectionPromise) return connectionPromise
    if (attempt === 'connect' ? !canConnect.value : !canRetry.value) return Promise.resolve()

    const running = runConnection(attempt)
    connectionPromise = running
    void running.finally(() => {
      if (connectionPromise === running) connectionPromise = undefined
      connecting.value = false
    })
    return running
  }

  function connect(): Promise<void> {
    return startConnection('connect')
  }

  function retry(): Promise<void> {
    return startConnection('retry')
  }

  async function disconnect(): Promise<void> {
    if (currentPort || currentReader || readLoopPromise) {
      await closeCurrentConnection('user')
      return
    }

    if (connectionPromise) {
      generation += 1
      if (!disposed.value) {
        state.value = {
          phase: 'disconnected',
          reason: 'user',
          message: 'Serial接続を中止しました。',
        }
      }
    }
  }

  if (getCurrentScope()) {
    onScopeDispose(() => {
      disposed.value = true
      generation += 1
      unregisterDisconnectListener()
      if (currentPort || currentReader || readLoopPromise) {
        void closeCurrentConnection('user')
      }
    })
  }

  return {
    state: readonly(state),
    busy,
    canConnect,
    canRetry,
    canDisconnect,
    connect,
    retry,
    disconnect,
  }
}
