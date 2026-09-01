<script setup lang="ts">
import PortalPage from './PortalPage.vue'
import { portalPageFixtures } from './PortalPage.fixture'
</script>

<template>
  <Story title="Pages/PortalPage">
    <Variant v-for="(fixture, name) in portalPageFixtures" :key="name" :title="name">
      <PortalPage v-bind="fixture" />
    </Variant>
  </Story>
</template>
