<script setup lang="ts">
import LogoutMenuButton from './LogoutMenuButton.vue'
</script>

<template>
  <Story title="LogoutMenuButton">
    <Variant title="Default">
      <LogoutMenuButton
        displayName="Shimeji"
        :open="false"
        @toggle="console.log('Toggle clicked')"
      />
    </Variant>
  </Story>
</template>
