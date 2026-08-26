<script setup lang="ts">
import AnswerPanel from './AnswerPanel.vue'
import { answerPanelFixture } from './AnswerPanel.fixture'
</script>

<template>
  <Story title="Room/AnswerPanel">
    <Variant title="入力可能">
      <AnswerPanel
        :max-length="answerPanelFixture.maxLength"
        :pending="false"
        :disabled="false"
        @submit="console.log('Submit answer:', $event)"
      />
    </Variant>

    <Variant title="送信中">
      <AnswerPanel :max-length="answerPanelFixture.maxLength" :pending="true" :disabled="false" />
    </Variant>

    <Variant title="入力不可">
      <AnswerPanel :max-length="answerPanelFixture.maxLength" :pending="false" :disabled="true" />
    </Variant>
  </Story>
</template>
