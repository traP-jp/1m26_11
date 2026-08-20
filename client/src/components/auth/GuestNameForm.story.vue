<script setup lang="ts">
import GuestNameForm from './GuestNameForm.vue'
</script>

<template>
  <Story title="GuestNameForm">
    <Variant title="通常状態（入力可能）">
      <GuestNameForm :submit-pending="false" @submit="console.log('submit:', $event)" />
    </Variant>

    <Variant title="送信中（操作不可）">
      <GuestNameForm :submit-pending="true" @submit="console.log('submit:', $event)" />
    </Variant>
  </Story>
</template>
