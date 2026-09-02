<script setup lang="ts">
import { onBeforeUnmount, ref } from 'vue'

import type { RoomUiEvent, RoomViewModel } from './RoomPage.types'
import RoomView from './RoomView.vue'
import SerialConnectControl from './components/room/SerialConnectControl.vue'
import type { Control, InputAdapterDispatcher, InputAdapterEvent } from './input/InputAdapter.types'
import { createKeyboardInputAdapter } from './input/keyboardInputAdapter'
import { createScreenButtonInputAdapter } from './input/screenButtonInputAdapter'
import { useWebSerialConnection } from './input/useWebSerialConnection'
import { createWebSerialInputAdapter } from './input/webSerialInputAdapter'

type InputMode = 'serial' | 'alternative'

const props = defineProps<{ viewModel: RoomViewModel }>()
const emit = defineEmits<{ uiEvent: [event: RoomUiEvent] }>()

const inputMode = ref<InputMode>('serial')
const roomDispatcher: InputAdapterDispatcher = {
  busy: false,
  dispatch(event: InputAdapterEvent) {
    emit('uiEvent', event)
    return true
  },
}

function createModeDispatcher(mode: InputMode): InputAdapterDispatcher {
  return {
    get busy() {
      return roomDispatcher.busy
    },
    dispatch(event) {
      if (inputMode.value !== mode) return false
      return roomDispatcher.dispatch(event)
    },
  }
}

function isControlAllowed(control: Control): boolean {
  return props.viewModel.queryInput.allowedControls.includes(control)
}

const keyboardInputAdapter = createKeyboardInputAdapter({
  dispatcher: createModeDispatcher('alternative'),
  isControlAllowed,
})
const screenInputAdapter = createScreenButtonInputAdapter({
  dispatcher: createModeDispatcher('alternative'),
  isControlAllowed,
})
const serialInputAdapter = createWebSerialInputAdapter({
  dispatcher: createModeDispatcher('serial'),
  isControlAllowed,
})
const serialConnection = useWebSerialConnection({ adapter: serialInputAdapter })

async function activateSerial(action: 'connect' | 'retry'): Promise<void> {
  keyboardInputAdapter.stop()
  inputMode.value = 'serial'
  await serialConnection[action]()
}

function switchToAlternativeInput(): void {
  inputMode.value = 'alternative'
  keyboardInputAdapter.start()
  void serialConnection.disconnect()
}

function pressScreenControl(control: Control): void {
  screenInputAdapter.pressControl(control)
}

const controlLabelByValue: Readonly<Record<string, string>> = {
  up: '上',
  down: '下',
  left: '左',
  right: '右',
  red: '赤',
  yellow: '黄',
  green: '緑',
}

function controlLabel(control: string): string {
  return controlLabelByValue[control] ?? control
}

onBeforeUnmount(() => keyboardInputAdapter.stop())
</script>

<template>
  <RoomView :view-model="viewModel" @ui-event="emit('uiEvent', $event)" />
  <aside class="mx-auto mt-4 w-full max-w-6xl px-4" aria-label="入力方法">
    <SerialConnectControl
      :state="serialConnection.state.value"
      :busy="serialConnection.busy.value"
      :can-connect="serialConnection.canConnect.value"
      :can-retry="serialConnection.canRetry.value"
      :can-disconnect="serialConnection.canDisconnect.value"
      @connect="activateSerial('connect')"
      @retry="activateSerial('retry')"
      @disconnect="serialConnection.disconnect"
      @switch-alternative="switchToAlternativeInput"
    />

    <section
      v-if="inputMode === 'alternative'"
      class="mt-3 rounded-xl border border-slate-200 bg-white p-4 shadow-sm"
      data-input-mode="alternative"
      aria-label="代替入力"
    >
      <p class="text-sm font-bold text-slate-950">キーボード／画面ボタン入力</p>
      <p class="mt-1 text-sm text-slate-600">矢印キーまたは表示中のボタンで操作できます。</p>
      <div class="mt-3 flex flex-wrap gap-2" role="group" aria-label="画面操作ボタン">
        <button
          v-for="control in viewModel.queryInput.allowedControls"
          :key="control"
          type="button"
          class="min-h-10 min-w-16 rounded-lg bg-blue-700 px-4 text-sm font-bold text-white hover:bg-blue-800 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-blue-700"
          :data-control="control"
          @click="pressScreenControl(control)"
        >
          {{ controlLabel(control) }}
        </button>
      </div>
    </section>
  </aside>
</template>
