<script setup lang="ts">
import UserMenu from './UserMenu.vue'
</script>

<template>
  <Story title="UserMenu">
    <Variant title="Default">
      <div class="h-48 p-4">
      <UserMenu
        displayName="Shimeji"
        :logoutPending="false"
        @logout="console.log('Logout clicked')"
      />
      </div>
    </Variant>
  </Story>
</template>
