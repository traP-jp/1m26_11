<script setup lang="ts">
import { computed } from 'vue'

export interface GameTimerProps {
  serverElapsedMs: number
}

const props = defineProps<GameTimerProps>()

const totalSeconds = computed(() => Math.floor(Math.max(0, props.serverElapsedMs) / 1000))
const minutes = computed(() => Math.floor(totalSeconds.value / 60))
const seconds = computed(() => totalSeconds.value % 60)
</script>

<template>
  <div class="game-timer" role="timer" :aria-label="`経過時間 ${minutes}分${seconds}秒`">
    <span class="game-timer__value">{{ minutes }}</span>
    <span class="game-timer__unit">m</span>
    <span class="game-timer__value">{{ seconds }}</span>
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
