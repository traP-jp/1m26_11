<script setup>
import ExitButton from './ExitButton.vue'
</script>

<template>
  <Story title="Components/ExitButton">
    <Variant title="Default">
      <div class="p-8 bg-gray-50 min-h-screen">
        <ExitButton @exit="() => console.log('退出処理が実行されましたわ！')" />
      </div>
    </Variant>
  </Story>
</template>