<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{ finalElapsedMs: number }>()
const emit = defineEmits<{ backToPortal: [] }>()

const elapsedTime = computed(() => {
  const totalSeconds = Math.max(0, Math.floor(props.finalElapsedMs / 1000))
  const minutes = Math.floor(totalSeconds / 60)
  const seconds = totalSeconds % 60
  return `${minutes}:${String(seconds).padStart(2, '0')}`
})
</script>

<template>
  <main class="grid min-h-screen place-items-center bg-[#eef3fa] p-4 text-[#121a2a]">
    <section
      class="w-full max-w-xl rounded-3xl border border-[#d7e1ef] bg-white px-6 py-12 text-center shadow-xl sm:px-12"
      aria-labelledby="clear-title"
    >
      <p class="text-sm font-extrabold tracking-[0.2em] text-[#2764d8]">ROOM CLEAR</p>
      <h1 id="clear-title" class="mt-3 text-4xl font-black sm:text-5xl">クリア！</h1>
      <p class="mt-8 text-sm font-bold text-[#65758d]">最終タイム</p>
      <p
        class="mt-2 font-mono text-5xl font-black tabular-nums text-[#152238] sm:text-6xl"
        data-testid="final-elapsed-time"
      >
        {{ elapsedTime }}
      </p>
      <button
        type="button"
        class="mt-10 inline-flex min-h-12 items-center justify-center rounded-xl bg-[#2864e8] px-8 text-base font-bold text-white transition-colors hover:bg-[#1f56cc] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8]"
        @click="emit('backToPortal')"
      >
        ポータルへ戻る
      </button>
    </section>
  </main>
</template>
