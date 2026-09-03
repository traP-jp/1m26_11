<script setup lang="ts">
import { Menu, MenuButton, MenuItems, MenuItem } from '@headlessui/vue'
import LogoutMenuButton from './LogoutMenuButton.vue'

withDefaults(
  defineProps<{
    displayName: string
    logoutHref?: string | null
    logoutPending: boolean
  }>(),
  { logoutHref: null },
)

const emit = defineEmits<{
  (e: 'logout'): void
}>()
</script>

<template>
  <Menu as="div" class="relative inline-block text-left" v-slot="{ open }">
    <MenuButton
      :as="LogoutMenuButton"
      :display-name="displayName"
      :open="open"
      :disabled="logoutPending"
    />

    <MenuItems
      class="absolute right-0 mt-2 w-auto bg-white rounded-md shadow-lg ring-1 ring-black ring-opacity-5 overflow-hidden"
    >
      <MenuItem v-slot="{ active }">
        <a
          v-if="logoutHref && !logoutPending"
          :href="logoutHref"
          :class="[
            active ? 'bg-blue-500 text-white' : 'text-gray-900',
            'block w-full px-4 py-2 text-left text-sm no-underline',
          ]"
          @click.prevent="emit('logout')"
        >
          ログアウト
        </a>
        <button
          v-else
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
