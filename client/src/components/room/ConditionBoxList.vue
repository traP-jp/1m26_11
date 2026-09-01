<script setup lang="ts">
import type { ConditionBoxItem } from './ConditionBoxList.types'

defineProps<{
  items: readonly ConditionBoxItem[]
  disabled: boolean
}>()

const emit = defineEmits<{
  add: []
  remove: [itemId: string]
  clear: []
}>()
</script>

<template>
  <section
    class="rounded-xl border border-[#c8d5e8] bg-[#eef4ff] p-4 text-[#121a2a]"
    aria-label="条件一覧"
  >
    <header class="flex flex-wrap items-center justify-between gap-3">
      <h2 class="text-base font-extrabold">条件一覧</h2>
      <div class="flex gap-2">
        <button
          type="button"
          :disabled="disabled"
          class="min-h-9 rounded-lg bg-[#2864e8] px-3 text-sm font-bold text-white transition-colors hover:bg-[#1f56cc] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:bg-[#a7b8d8]"
          @click="emit('add')"
        >
          条件を追加
        </button>
        <button
          type="button"
          :disabled="disabled || items.length === 0"
          class="min-h-9 rounded-lg border border-[#c8d5e8] bg-white px-3 text-sm font-bold text-[#52627a] transition-colors hover:bg-[#f7f9fd] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#2864e8] disabled:cursor-not-allowed disabled:bg-[#f1f4f8] disabled:text-[#8a97aa]"
          @click="emit('clear')"
        >
          全消去
        </button>
      </div>
    </header>

    <p v-if="items.length === 0" class="mt-4 text-sm text-[#65758d]">条件はありません。</p>
    <ul v-else class="mt-4 grid gap-2" aria-label="追加済みの条件">
      <li
        v-for="item in items"
        :key="item.id"
        class="flex min-w-0 items-center justify-between gap-3 rounded-lg border border-[#d7e1ef] bg-white px-3 py-2"
      >
        <span class="min-w-0 truncate text-sm font-bold">{{ item.label }}</span>
        <button
          type="button"
          :disabled="disabled"
          :aria-label="`${item.label}を削除`"
          class="shrink-0 rounded-md px-2 py-1 text-xs font-bold text-red-700 transition-colors hover:bg-red-50 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-red-700 disabled:cursor-not-allowed disabled:text-[#8a97aa]"
          @click="emit('remove', item.id)"
        >
          削除
        </button>
      </li>
    </ul>
  </section>
</template>
