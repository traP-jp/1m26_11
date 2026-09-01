<script setup lang="ts">
import ClearPage from './ClearPage.vue'
import { portalPageFixtures } from './PortalPage.fixture'
import PortalPage from './PortalPage.vue'
import RoomPage from './RoomPage.vue'
import { roomPageFixture } from './RoomPage.fixture'
</script>

<template>
  <Story title="App/Routes">
    <Variant title="Portal">
      <PortalPage v-bind="portalPageFixtures.demoAuthenticated" />
    </Variant>

    <Variant title="Room">
      <RoomPage :view-model="roomPageFixture" />
    </Variant>

    <Variant title="Clear">
      <ClearPage />
    </Variant>
  </Story>
</template>
