<script setup lang="ts">
import UserMenuButton from './UserMenuButton.vue'
</script>

<template>
  <Story title="UserMenuButton">
    <Variant title="Default">
      <UserMenuButton displayName="Shimeji" :open="false" @toggle="console.log('Toggle clicked')" />
    </Variant>
  </Story>
</template>