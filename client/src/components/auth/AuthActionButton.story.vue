<script setup lang="ts">
import AuthActionButton from './AuthActionButton.vue'
import { authActionButtonFixtures } from './AuthActionButton.fixture'
</script>

<template>
  <Story title="Auth/AuthActionButton">
    <Variant title="Login">
      <AuthActionButton v-bind="authActionButtonFixtures.login" />
    </Variant>
    <Variant title="Logout">
      <AuthActionButton v-bind="authActionButtonFixtures.logout" />
    </Variant>
    <Variant title="Disabled">
      <AuthActionButton v-bind="authActionButtonFixtures.busy" />
    </Variant>
  </Story>
</template>
