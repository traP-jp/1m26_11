<script setup lang="ts">
import AuthorProblemPage from './AuthorProblemPage.vue'

const roomId = '1411824c-d357-4941-af76-c76cb827dda6'
</script>

<template>
  <Story title="Pages/AuthorProblemPage">
    <Variant title="New problem form">
      <AuthorProblemPage :room-id="roomId" />
    </Variant>
  </Story>
</template>
