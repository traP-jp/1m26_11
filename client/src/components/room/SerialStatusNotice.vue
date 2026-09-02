<script setup lang="ts">
import { computed, useId } from 'vue'

import type { SerialStatus, SerialStatusNoticeProps } from './SerialStatusNotice.types'

const props = defineProps<SerialStatusNoticeProps>()
const emit = defineEmits<{
  retry: []
  'use-keyboard': []
  'use-screen': []
}>()

const titleId = useId()
const descriptionId = `${titleId}-description`

const presentationByStatus = {
  unsupported: {
    title: 'この環境ではSerialを利用できません',
    description:
      'Web Serial対応ブラウザでHTTPSまたはlocalhostから開くか、代替入力をご利用ください。',
    classes: 'border-amber-300 bg-amber-50 text-amber-950',
    role: 'status',
  },
  denied: {
    title: 'Serial接続が許可されませんでした',
    description: '接続が拒否またはキャンセルされました。再試行するか、代替入力をご利用ください。',
    classes: 'border-red-300 bg-red-50 text-red-900',
    role: 'alert',
  },
  connecting: {
    title: 'Serialへ接続中です',
    description: 'Serialデバイスを選択し、接続が完了するまでお待ちください。',
    classes: 'border-[#7aa7ff] bg-[#eef4ff] text-[#204f9f]',
    role: 'status',
  },
  connected: {
    title: 'Serialに接続しました',
    description: '物理コントローラーから入力できます。',
    classes: 'border-emerald-300 bg-emerald-50 text-emerald-900',
    role: 'status',
  },
  disconnected: {
    title: 'Serialは接続されていません',
    description: '接続するか、キーボードまたは画面ボタンへ切り替えてください。',
    classes: 'border-[#c8d5e8] bg-[#eef4ff] text-[#204f9f]',
    role: 'status',
  },
  'retry-failed': {
    title: 'Serialの再接続に失敗しました',
    description: 'もう一度再試行するか、代替入力をご利用ください。',
    classes: 'border-red-300 bg-red-50 text-red-900',
    role: 'alert',
  },
} as const satisfies Record<
  SerialStatus,
  { title: string; description: string; classes: string; role: 'alert' | 'status' }
>

const retryStatuses = new Set<SerialStatus>(['denied', 'disconnected', 'retry-failed'])
const presentation = computed(() => presentationByStatus[props.status])
const canRetry = computed(() => retryStatuses.has(props.status))
const actionsDisabled = computed(() => props.status === 'connecting')

function useKeyboard(): void {
  if (actionsDisabled.value) return
  emit('use-keyboard')
}

function useScreen(): void {
  if (actionsDisabled.value) return
  emit('use-screen')
}

function retry(): void {
  if (props.retryDisabled) return
  emit('retry')
}
</script>

<template>
  <section
    class="flex w-full flex-col gap-3 rounded-xl border px-4 py-3 shadow-sm sm:flex-row sm:items-center sm:justify-between"
    :class="presentation.classes"
    :role="presentation.role"
    :aria-labelledby="titleId"
    :aria-describedby="descriptionId"
    :aria-busy="status === 'connecting' ? 'true' : undefined"
    :aria-live="presentation.role === 'alert' ? 'assertive' : 'polite'"
    :data-status="status"
    data-testid="serial-status-notice"
  >
    <div class="min-w-0">
      <p :id="titleId" class="text-sm font-extrabold" data-testid="serial-status-title">
        {{ presentation.title }}
      </p>
      <p :id="descriptionId" class="mt-1 text-xs leading-5 sm:text-sm">
        {{ presentation.description }}
      </p>
    </div>

    <div class="flex shrink-0 flex-wrap gap-2" role="group" aria-label="入力方法を選択">
      <button
        v-if="canRetry"
        type="button"
        :disabled="retryDisabled"
        class="min-h-10 rounded-lg bg-[#2864e8] px-4 text-sm font-bold text-white transition-colors hover:bg-[#1f56cc] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:bg-[#8a97aa]"
        data-testid="serial-retry"
        @click="retry"
      >
        {{ retryLabel ?? (status === 'disconnected' ? '接続する' : '再試行する') }}
      </button>
      <button
        type="button"
        :disabled="actionsDisabled"
        class="min-h-10 rounded-lg border border-[#c8d5e8] bg-white px-4 text-sm font-bold text-[#36465f] transition-colors hover:border-[#8da9d2] hover:bg-[#f7f9fd] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:border-[#d7e1ef] disabled:bg-[#f1f4f8] disabled:text-[#8a97aa]"
        data-testid="use-keyboard"
        @click="useKeyboard"
      >
        キーボードを使う
      </button>
      <button
        type="button"
        :disabled="actionsDisabled"
        class="min-h-10 rounded-lg border border-[#c8d5e8] bg-white px-4 text-sm font-bold text-[#36465f] transition-colors hover:border-[#8da9d2] hover:bg-[#f7f9fd] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:border-[#d7e1ef] disabled:bg-[#f1f4f8] disabled:text-[#8a97aa]"
        data-testid="use-screen"
        @click="useScreen"
      >
        画面ボタンを使う
      </button>
    </div>
  </section>
</template>
