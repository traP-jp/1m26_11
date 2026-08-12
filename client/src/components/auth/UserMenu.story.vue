<script setup lang="ts">
import UserMenu from './UserMenu.vue'
</script>

<template>
  <Story title="UserMenu">
    <Variant title="Default">
      <UserMenu
        displayName="Shimeji"
        :logoutPending="false"
        @logout="console.log('Logout clicked')"
      />
    </Variant>
  </Story>
</template>
