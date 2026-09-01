<script setup lang="ts">
import AuthActionButton from '../auth/AuthActionButton.vue'
import PortalLoginPrompt from './PortalLoginPrompt.vue'
import { portalLoginPromptFixture } from './PortalLoginPrompt.fixture'
</script>

<template>
  <Story title="Portal/PortalLoginPrompt">
    <Variant title="Default">
      <PortalLoginPrompt>
        <template #action>
          <AuthActionButton action="login" :label="portalLoginPromptFixture.actionLabel" />
        </template>
      </PortalLoginPrompt>
    </Variant>
  </Story>
</template>
