<script setup lang="ts">
import type { RoomUiEvent, RoomViewModel } from './RoomPage.types'
import RoomView from './RoomView.vue'
import SerialConnectControl from './components/room/SerialConnectControl.vue'
import { useWebSerialConnection } from './input/useWebSerialConnection'
import { createWebSerialInputAdapter } from './input/webSerialInputAdapter'

const props = defineProps<{ viewModel: RoomViewModel }>()
const emit = defineEmits<{ uiEvent: [event: RoomUiEvent] }>()

const serialInputAdapter = createWebSerialInputAdapter({
  dispatcher: {
    dispatch(event) {
      emit('uiEvent', event)
      return true
    },
  },
  isControlAllowed: (control) => props.viewModel.queryInput.allowedControls.includes(control),
})
const { state, busy, canConnect, canRetry, canDisconnect, connect, retry, disconnect } =
  useWebSerialConnection({ adapter: serialInputAdapter })

function switchToAlternativeInput(): void {
  void disconnect()
}
</script>

<template>
  <RoomView :view-model="viewModel" @ui-event="emit('uiEvent', $event)" />
  <aside class="mx-auto mt-4 w-full max-w-6xl px-4" aria-label="入力方法">
    <SerialConnectControl
      :state="state"
      :busy="busy"
      :can-connect="canConnect"
      :can-retry="canRetry"
      :can-disconnect="canDisconnect"
      @connect="connect"
      @retry="retry"
      @disconnect="disconnect"
      @switch-alternative="switchToAlternativeInput"
    />
  </aside>
</template>
