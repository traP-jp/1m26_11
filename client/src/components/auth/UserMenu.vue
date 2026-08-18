<script setup lang="ts">
import { Menu, MenuButton, MenuItems, MenuItem } from '@headlessui/vue'
import LogoutMenuButton from './LogoutMenuButton.vue'

defineProps<{
  displayName: string
  logoutPending: boolean
}>()

const emit = defineEmits<{
  (e: 'logout'): void
}>()
</script>

<template>
  <Menu as="div" class="relative inline-block text-left" v-slot="{ open }">
    <MenuButton as="LogoutMenuButton">
      <LogoutMenuButton :displayName="displayName" :open="open" />
    </MenuButton>

    <MenuItems
      class="absolute right-0 mt-2 w-auto bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5"
    >
      <MenuItem v-slot="{ active }">
        <button
          type="button"
          :disabled="logoutPending"
          @click="emit('logout')"
          :class="[
            active ? 'bg-blue-500 text-white' : 'text-gray-900',
            'block w-full text-left px-4 py-2 text-sm',
          ]"
        >
          ログアウト
        </button>
      </MenuItem>
    </MenuItems>
  </Menu>
</template>
