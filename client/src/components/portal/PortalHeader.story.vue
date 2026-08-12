<script setup lang="ts">
import PortalHeader from './PortalHeader.vue'
import { portalHeaderFixtures } from './PortalHeader.fixture'
</script>

<template>
  <Story title="Portal/PortalHeader">
    <Variant title="未ログイン">
      <PortalHeader
        v-bind="portalHeaderFixtures.unauthenticated"
        @login="console.log('Login clicked')"
      />
    </Variant>

    <Variant title="Demo ログイン済み">
      <PortalHeader
        v-bind="portalHeaderFixtures.demoAuthenticated"
        @logout="console.log('Demo logout clicked')"
      />
    </Variant>

    <Variant title="NeoShowcase ログイン済み">
      <PortalHeader v-bind="portalHeaderFixtures.neoshowcaseAuthenticated" />
    </Variant>
  </Story>
</template>
