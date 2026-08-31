<script setup lang="ts">
import MinimalProgressSummary from './MinimalProgressSummary.vue'
import { minimalProgressSummaryFixtures } from './MinimalProgressSummary.fixture'
</script>

<template>
  <Story title="Portal/MinimalProgressSummary">
    <Variant v-for="(status, name) in minimalProgressSummaryFixtures" :key="name" :title="name">
      <MinimalProgressSummary :status="status" />
    </Variant>
  </Story>
</template>
