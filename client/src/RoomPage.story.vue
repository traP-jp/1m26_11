<script setup lang="ts">
import { roomPageFixture } from './RoomPage.fixture'
import RoomPage from './RoomPage.vue'

const clearedViewModel = {
  ...roomPageFixture,
  serverElapsedMs: 754_000,
  clear: {
    ...roomPageFixture.clear,
    cleared: true,
    clearedCount: roomPageFixture.clear.requiredCount,
  },
}
</script>

<template>
  <Story title="Pages/RoomPage">
    <Variant title="Playing">
      <RoomPage :view-model="roomPageFixture" />
    </Variant>
    <Variant title="Cleared">
      <RoomPage :view-model="clearedViewModel" />
    </Variant>
  </Story>
</template>
