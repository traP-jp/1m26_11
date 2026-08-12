<script setup lang="ts">
import { ref } from 'vue'
import LogoutMenuButton from './LogoutMenuButton.vue'

defineProps<{
  displayName: string
  logoutPending: boolean
}>()

const emit = defineEmits<{
  (e: 'logout'): void
}>()

const open = ref(false)
</script>

<template>
  <div>
    <LogoutMenuButton :displayName="displayName" :open="open" @toggle="open = !open" />

    <div v-if="open">
      <button type="button" :disabled="logoutPending" @click="emit('logout')">ログアウト</button>
    </div>
  </div>
</template>
