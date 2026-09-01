<script setup lang="ts">
import { computed, useId } from 'vue'

import type { RawSerialChunk, SerialPocState } from '../../device-poc/types'

const props = defineProps<{
  state: SerialPocState
  chunks: readonly RawSerialChunk[]
  decodedPreview: string
  totalBytes: number
  captureLimitBytes: number
  canConnect: boolean
  canStop: boolean
  canClear: boolean
  canDownload: boolean
  lastExportSha256?: string
}>()

const emit = defineEmits<{
  connect: []
  stop: []
  clear: []
  downloadRaw: []
  downloadMetadata: []
}>()

const titleId = useId()
const visibleChunks = computed(() => props.chunks.slice(-200))
const hiddenChunkCount = computed(() => props.chunks.length - visibleChunks.value.length)
const visibleDecodedPreview = computed(() => {
  const preview = props.decodedPreview.slice(-16_000)
  const visible = makeControlCharactersVisible(preview)
  return props.decodedPreview.length > preview.length
    ? `[... previewの先頭を省略しました ...]\n${visible}`
    : visible
})
const statusClasses = computed(() => {
  if (props.state.phase === 'error') return 'border-red-300 bg-red-50 text-red-900'
  if (props.state.phase === 'running') return 'border-emerald-300 bg-emerald-50 text-emerald-900'
  if (props.state.phase === 'unsupported') return 'border-amber-300 bg-amber-50 text-amber-950'
  return 'border-slate-300 bg-slate-50 text-slate-800'
})

function formatHex(bytes: Uint8Array): string {
  return [...bytes].map((byte) => byte.toString(16).padStart(2, '0')).join(' ')
}

function formatElapsed(milliseconds: number): string {
  return `${milliseconds.toFixed(1)} ms`
}

function makeControlCharactersVisible(text: string): string {
  let visible = ''

  for (const character of text) {
    if (character === '\r') {
      visible += '␍'
    } else if (character === '\n') {
      visible += '␊\n'
    } else {
      const codePoint = character.charCodeAt(0)
      visible +=
        codePoint <= 0x1f || codePoint === 0x7f
          ? `\\x${codePoint.toString(16).padStart(2, '0')}`
          : character
    }
  }

  return visible
}
</script>

<template>
  <section
    class="mx-auto w-full max-w-6xl rounded-2xl border border-slate-200 bg-white p-5 shadow-sm sm:p-8"
    :aria-labelledby="titleId"
  >
    <header class="flex flex-col gap-2 border-b border-slate-200 pb-5">
      <p class="text-xs font-bold uppercase tracking-[0.18em] text-blue-700">Development PoC</p>
      <h1 :id="titleId" class="text-2xl font-bold text-slate-950 sm:text-3xl">
        Device Web Serial raw viewer
      </h1>
      <p class="max-w-3xl text-sm leading-6 text-slate-600">
        Raspberry Pi Pico HからWire v1のraw byteを観測する診断画面です。frameの検証や操作列への
        変換は行わず、.binへ保存したbyte列を正本とします。
      </p>
    </header>

    <div class="mt-5 grid gap-4 lg:grid-cols-[minmax(0,1fr)_18rem]">
      <div
        class="rounded-xl border px-4 py-3"
        :class="statusClasses"
        :role="state.phase === 'error' ? 'alert' : 'status'"
        aria-live="polite"
        data-testid="serial-status"
      >
        <p class="text-xs font-bold uppercase tracking-wide">{{ state.phase }}</p>
        <p class="mt-1 text-sm">{{ state.message }}</p>
      </div>

      <dl class="grid grid-cols-2 gap-x-4 gap-y-2 rounded-xl bg-slate-950 p-4 text-xs text-white">
        <div>
          <dt class="text-slate-400">PoC open設定</dt>
          <dd class="mt-1 font-mono">115200 / 8-N-1</dd>
        </div>
        <div>
          <dt class="text-slate-400">Capture</dt>
          <dd class="mt-1 font-mono">{{ totalBytes.toLocaleString() }} byte</dd>
        </div>
        <div>
          <dt class="text-slate-400">Chunk</dt>
          <dd class="mt-1 font-mono">{{ chunks.length }}</dd>
        </div>
        <div>
          <dt class="text-slate-400">停止目安</dt>
          <dd class="mt-1 font-mono">{{ captureLimitBytes.toLocaleString() }} byte</dd>
        </div>
      </dl>
    </div>

    <div class="mt-5 flex flex-wrap gap-3" aria-label="Serial操作">
      <button
        type="button"
        class="rounded-lg bg-blue-700 px-4 py-2 text-sm font-bold text-white hover:bg-blue-800 disabled:cursor-not-allowed disabled:bg-slate-300"
        :disabled="!canConnect"
        @click="emit('connect')"
      >
        Connect &amp; start
      </button>
      <button
        type="button"
        class="rounded-lg bg-red-700 px-4 py-2 text-sm font-bold text-white hover:bg-red-800 disabled:cursor-not-allowed disabled:bg-slate-300"
        :disabled="!canStop"
        @click="emit('stop')"
      >
        Stop &amp; disconnect
      </button>
      <button
        type="button"
        class="rounded-lg border border-slate-400 bg-white px-4 py-2 text-sm font-bold text-slate-800 hover:bg-slate-100 disabled:cursor-not-allowed disabled:text-slate-400"
        :disabled="!canDownload"
        @click="emit('downloadRaw')"
      >
        Download capture.bin
      </button>
      <button
        type="button"
        class="rounded-lg border border-slate-400 bg-white px-4 py-2 text-sm font-bold text-slate-800 hover:bg-slate-100 disabled:cursor-not-allowed disabled:text-slate-400"
        :disabled="!canDownload"
        @click="emit('downloadMetadata')"
      >
        Download capture.json
      </button>
      <button
        type="button"
        class="rounded-lg border border-slate-300 bg-white px-4 py-2 text-sm text-slate-700 hover:bg-slate-100 disabled:cursor-not-allowed disabled:text-slate-400"
        :disabled="!canClear"
        @click="emit('clear')"
      >
        Clear capture
      </button>
    </div>

    <p v-if="lastExportSha256" class="mt-3 break-all font-mono text-xs text-slate-600">
      Last export SHA-256: {{ lastExportSha256 }}
    </p>

    <div class="mt-8 grid gap-6 xl:grid-cols-2">
      <section aria-labelledby="raw-chunks-title" class="min-w-0">
        <div class="flex items-end justify-between gap-4">
          <div>
            <h2 id="raw-chunks-title" class="text-lg font-bold text-slate-950">Raw chunks</h2>
            <p class="mt-1 text-xs text-slate-500">chunk境界はframe境界ではありません。</p>
          </div>
          <p v-if="hiddenChunkCount > 0" class="text-xs text-slate-500">
            古い{{ hiddenChunkCount }} chunkは表示のみ省略
          </p>
        </div>

        <div class="mt-3 max-h-[32rem] overflow-auto rounded-xl border border-slate-200">
          <table class="w-full border-collapse text-left text-xs">
            <thead class="sticky top-0 bg-slate-100 text-slate-600">
              <tr>
                <th class="px-3 py-2 font-bold"># / connection</th>
                <th class="px-3 py-2 font-bold">offset / time</th>
                <th class="px-3 py-2 font-bold">raw hex</th>
              </tr>
            </thead>
            <tbody class="divide-y divide-slate-100 font-mono text-slate-800">
              <tr v-for="chunk in visibleChunks" :key="chunk.sequence">
                <td class="whitespace-nowrap px-3 py-2 align-top">
                  {{ chunk.sequence }} / {{ chunk.connectionId }}
                </td>
                <td class="whitespace-nowrap px-3 py-2 align-top">
                  {{ chunk.offset }}<br />{{ formatElapsed(chunk.receivedElapsedMs) }}
                </td>
                <td class="break-all px-3 py-2 align-top" data-testid="raw-hex">
                  {{ formatHex(chunk.bytes) }}
                </td>
              </tr>
              <tr v-if="visibleChunks.length === 0">
                <td colspan="3" class="px-3 py-8 text-center font-sans text-slate-500">
                  受信dataはまだありません。
                </td>
              </tr>
            </tbody>
          </table>
        </div>
      </section>

      <section aria-labelledby="decoded-preview-title" class="min-w-0">
        <h2 id="decoded-preview-title" class="text-lg font-bold text-slate-950">UTF-8参考表示</h2>
        <p class="mt-1 text-xs text-slate-500">
          streaming decodeした派生表示です。.binのraw byteが正本です。
        </p>
        <pre
          class="mt-3 min-h-48 max-h-[32rem] overflow-auto whitespace-pre-wrap break-all rounded-xl bg-slate-950 p-4 font-mono text-xs leading-5 text-emerald-200"
          data-testid="decoded-preview"
          >{{ visibleDecodedPreview || '受信dataはまだありません。' }}</pre>
      </section>
    </div>
  </section>
</template>
