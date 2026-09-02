<script setup lang="ts">
import { computed, useId } from 'vue'

import type { SerialConnectionState } from '../../input/useWebSerialConnection'

const props = defineProps<{
  state: SerialConnectionState
  busy: boolean
  canConnect: boolean
  canRetry: boolean
  canDisconnect: boolean
}>()

const emit = defineEmits<{
  connect: []
  retry: []
  disconnect: []
  switchAlternative: []
}>()

const titleId = useId()
const statusTitle = computed(() => {
  switch (props.state.phase) {
    case 'unsupported':
      return 'Web Serial非対応'
    case 'idle':
      return 'Serial未接続'
    case 'requesting':
      return props.state.attempt === 'retry' ? 'Serial再接続中' : 'Serial接続中'
    case 'connected':
      return 'Serial接続済み'
    case 'disconnected':
      return 'Serial切断'
    case 'error':
      return 'Serial接続エラー'
  }
  return ''
})
const statusClasses = computed(() => {
  if (props.state.phase === 'error') return 'border-red-300 bg-red-50 text-red-900'
  if (props.state.phase === 'connected') return 'border-emerald-300 bg-emerald-50 text-emerald-900'
  if (props.state.phase === 'unsupported') return 'border-amber-300 bg-amber-50 text-amber-950'
  return 'border-slate-300 bg-slate-50 text-slate-800'
})
</script>

<template>
  <section
    class="rounded-xl border border-slate-200 bg-white p-4 shadow-sm"
    :aria-labelledby="titleId"
    data-testid="serial-connect-control"
  >
    <h2 :id="titleId" class="text-sm font-bold text-slate-950">物理コントローラー</h2>

    <div
      class="mt-3 rounded-lg border px-3 py-2"
      :class="statusClasses"
      :role="state.phase === 'error' ? 'alert' : 'status'"
      aria-live="polite"
    >
      <p class="text-xs font-bold">{{ statusTitle }}</p>
      <p class="mt-1 text-sm">{{ state.message }}</p>
    </div>

    <div class="mt-3 flex flex-wrap gap-2">
      <button
        v-if="state.phase === 'idle'"
        type="button"
        class="rounded-lg bg-blue-700 px-3 py-2 text-sm font-bold text-white hover:bg-blue-800 disabled:cursor-not-allowed disabled:bg-slate-300"
        :disabled="!canConnect"
        @click="emit('connect')"
      >
        Serialへ接続
      </button>

      <button
        v-if="state.phase === 'disconnected' || (state.phase === 'error' && !canDisconnect)"
        type="button"
        class="rounded-lg bg-blue-700 px-3 py-2 text-sm font-bold text-white hover:bg-blue-800 disabled:cursor-not-allowed disabled:bg-slate-300"
        :disabled="!canRetry"
        @click="emit('retry')"
      >
        再接続
      </button>

      <button
        v-if="canDisconnect"
        type="button"
        class="rounded-lg border border-slate-400 bg-white px-3 py-2 text-sm font-bold text-slate-800 hover:bg-slate-100 disabled:cursor-not-allowed disabled:text-slate-400"
        :disabled="busy"
        @click="emit('disconnect')"
      >
        {{ state.phase === 'error' ? 'ポート解放を再試行' : 'Serialを切断' }}
      </button>

      <button
        type="button"
        class="rounded-lg border border-slate-400 bg-white px-3 py-2 text-sm font-bold text-slate-800 hover:bg-slate-100"
        @click="emit('switchAlternative')"
      >
        キーボード／画面ボタンで続ける
      </button>
    </div>
  </section>
</template>
