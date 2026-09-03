<script setup lang="ts">
import { portalPageFixtures } from './PortalPage.fixture'
import PortalPage from './PortalPage.vue'
import RoomPage from './RoomPage.vue'
import { roomPageFixture } from './RoomPage.fixture'

const clearedRoomPageFixture = {
  ...roomPageFixture,
  clear: {
    ...roomPageFixture.clear,
    cleared: true,
    clearedCount: roomPageFixture.clear.requiredCount,
  },
}
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
      <RoomPage :view-model="clearedRoomPageFixture" />
    </Variant>
  </Story>
</template>
