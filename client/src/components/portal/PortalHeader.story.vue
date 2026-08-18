<script setup lang="ts">
import PortalHeader from './PortalHeader.vue'
import { portalHeaderFixtures } from './PortalHeader.fixture'
</script>

<template>
  <Story title="Portal/PortalHeader">
    <Variant title="未ログイン">
      <div class="min-h-40">
        <PortalHeader
          v-bind="portalHeaderFixtures.unauthenticated"
          @login="console.log('Login clicked')"
        />
      </div>
    </Variant>

    <Variant title="Demo ログイン済み">
      <div class="min-h-40">
        <PortalHeader
          v-bind="portalHeaderFixtures.demoAuthenticated"
          @logout="console.log('Demo logout clicked')"
        />
      </div>
    </Variant>

    <Variant title="NeoShowcase ログイン済み">
      <div class="min-h-40">
        <PortalHeader v-bind="portalHeaderFixtures.neoshowcaseAuthenticated" />
      </div>
    </Variant>
  </Story>
</template>
