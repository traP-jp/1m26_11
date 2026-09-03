<script setup lang="ts">
export interface Props {
  problemNumber: number
  title: string
  problemType: 'small' | 'final'
  smallIndex?: number
  smallTotal?: number
}

withDefaults(defineProps<Props>(), {
  smallTotal: 3,
})
</script>

<template>
  <div class="flex items-center w-full px-8 py-6 bg-white border-y border-gray-200">
    <div class="flex flex-col items-center justify-center shrink-0 w-24 h-24 bg-blue-50 rounded-xl">
      <span class="mb-1 text-xs font-bold tracking-widest text-blue-600"> QUESTION </span>
      <span class="text-3xl font-bold text-gray-900"> 謎 {{ problemNumber }} </span>
    </div>

    <div class="flex-1 px-8">
      <h2 class="text-xl font-bold text-gray-900">
        {{ title }}
      </h2>
    </div>

    <div class="shrink-0">
      <div v-if="problemType === 'small'" class="text-sm font-bold text-blue-600">
        小なぞ {{ smallIndex }} / {{ smallTotal }}
      </div>
      <div v-else class="text-sm font-bold text-blue-600">大なぞ</div>
    </div>
  </div>
</template>
