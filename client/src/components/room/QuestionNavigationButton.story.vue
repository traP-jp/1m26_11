<script setup lang="ts">
import QuestionNavigationButton from './QuestionNavigationButton.vue'
import { questionNavigationButtonFixture } from './QuestionNavigationButton.fixture'
</script>

<template>
  <Story title="Room/QuestionNavigationButton">
    <Variant title="前の問題">
      <QuestionNavigationButton
        v-bind="questionNavigationButtonFixture.previous"
        :disabled="false"
        @select="console.log('Select problem:', $event)"
      />
    </Variant>

    <Variant title="次の問題">
      <QuestionNavigationButton
        v-bind="questionNavigationButtonFixture.next"
        :disabled="false"
        @select="console.log('Select problem:', $event)"
      />
    </Variant>

    <Variant title="無効">
      <div class="flex gap-3">
        <QuestionNavigationButton v-bind="questionNavigationButtonFixture.previous" disabled />
        <QuestionNavigationButton v-bind="questionNavigationButtonFixture.next" disabled />
      </div>
    </Variant>
  </Story>
</template>
