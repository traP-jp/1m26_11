<script setup lang="ts">
import { provide } from 'vue'

import ClearPage from './ClearPage.vue'
import PortalPage from './PortalPage.vue'
import RoomPage from './RoomPage.vue'
import { roomPageFixture } from './RoomPage.fixture'
import { createFixtureApiClient } from './api/client.fixture'
import { authApiClientKey } from './utils/auth'

provide(authApiClientKey, createFixtureApiClient())
</script>

<template>
  <Story title="App/Routes">
    <Variant title="Portal">
      <PortalPage />
    </Variant>

    <Variant title="Room">
      <RoomPage :view-model="roomPageFixture" />
    </Variant>

    <Variant title="Clear">
      <ClearPage />
    </Variant>
  </Story>
</template>
