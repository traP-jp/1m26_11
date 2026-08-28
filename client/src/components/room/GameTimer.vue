<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted, ref, watch } from 'vue'

export interface GameTimerProps {
  serverElapsedMs: number
  active: boolean
}

const props = defineProps<GameTimerProps>()

const TICK_INTERVAL_MS = 250

const normalizeElapsedMs = (elapsedMs: number): number => Math.max(0, elapsedMs)

const initialNow = Date.now()
const synchronizedAt = ref(initialNow)
const currentTime = ref(initialNow)
let timerId: number | undefined

const displayedElapsedMs = computed(() => {
  const elapsedSinceSynchronization = props.active
    ? Math.max(0, currentTime.value - synchronizedAt.value)
    : 0
  return normalizeElapsedMs(props.serverElapsedMs) + elapsedSinceSynchronization
})
const totalSeconds = computed(() => Math.floor(displayedElapsedMs.value / 1000))
const minutes = computed(() => Math.floor(totalSeconds.value / 60))
const seconds = computed(() => totalSeconds.value % 60)

function updateCurrentTime(): void {
  currentTime.value = Date.now()
}

function synchronizeWithServer(): void {
  const now = Date.now()
  synchronizedAt.value = now
  currentTime.value = now
}

function startTimer(): void {
  if (timerId !== undefined) return
  timerId = window.setInterval(updateCurrentTime, TICK_INTERVAL_MS)
}

function stopTimer(): void {
  if (timerId === undefined) return
  window.clearInterval(timerId)
  timerId = undefined
}

watch(
  [() => props.serverElapsedMs, () => props.active],
  ([, active]) => {
    synchronizeWithServer()
    if (active) {
      startTimer()
      return
    }

    stopTimer()
  },
  { flush: 'sync' },
)

onMounted(() => {
  if (props.active) startTimer()
})
onBeforeUnmount(stopTimer)
</script>

<template>
  <div class="game-timer" role="timer" :aria-label="`経過時間 ${minutes}分${seconds}秒`">
    <span :class="['game-timer__value', minutes < 10 ? 'w-[1ch]' : 'w-[2ch]']"> # 幅変わるのキモかったから、固定幅にしたよ
      {{ minutes }}
    </span>
    <span class="game-timer__unit">m</span>
    <span :class="['game-timer__value', seconds < 10 ? 'w-[1ch]' : 'w-[2ch]']">
      {{ seconds }}
    </span>
    <span class="game-timer__unit">s</span>
  </div>
</template>

<style scoped>
.game-timer {
  display: inline-flex;
  align-items: baseline;
  min-width: 10rem;
  justify-content: center;
  padding: 1rem 1.25rem;
  border: 1px solid #d8e1ee;
  border-radius: 1.25rem;
  color: #121a2a;
  background: #fff;
  box-shadow: 0 0.25rem 1.75rem rgb(21 34 56 / 12%);
  font-variant-numeric: tabular-nums;
}

.game-timer__value {
  font-size: 3rem;
  font-weight: 700;
  letter-spacing: -0.01em;
  line-height: 1;
}

.game-timer__unit {
  margin: 0 0.3rem 0 0.1rem;
  color: #65758d;
  font-size: 1.25rem;
  font-weight: 700;
  line-height: 1;
}

.game-timer__unit:last-child {
  margin-right: 0;
}
</style>
