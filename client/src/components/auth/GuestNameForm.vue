<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  submitPending: boolean
}>()

const emit = defineEmits<{
  (e: 'submit', displayName: string): void
}>()

const displayName = ref('')

const onSubmit = () => {
  if (props.submitPending) return
  emit('submit', displayName.value)
}
</script>

<template>
  <div class="bg-white rounded-xl shadow-md p-8 max-w-md w-full mx-auto">
    <h1 class="text-2xl font-bold text-gray-800 mb-2">名前を入力してはじめる</h1>
    <p class="text-gray-600 mb-6">ゲーム内で表示する名前を入力してください。</p>

    <form @submit.prevent="onSubmit" class="flex flex-col gap-4">
      <div>
        <label for="displayNameInput" class="block text-sm font-medium text-gray-700 mb-1">
          表示名
        </label>
        <p class="text-xs text-gray-500 mb-2">ゲーム内で使用する表示名です</p>

        <input
          id="displayNameInput"
          v-model="displayName"
          type="text"
          placeholder="名前を入力してください"
          :disabled="submitPending"
          class="text-gray-900 placeholder-gray-400 bg-white w-full border border-gray-300 rounded-md px-4 py-2 focus:outline-none focus:ring-2 focus:ring-blue-500 disabled:bg-gray-100 disabled:text-gray-400 disabled:cursor-not-allowed"
        />
      </div>

      <button
        type="submit"
        :disabled="submitPending"
        class="mt-4 w-full bg-blue-600 text-white font-semibold py-3 px-4 rounded-md hover:bg-blue-700 transition-colors focus:outline-none focus:ring-2 focus:ring-blue-500 focus:ring-offset-2 disabled:bg-blue-300 disabled:cursor-not-allowed"
      >
        この名前ではじめる
      </button>
    </form>
  </div>
</template>
