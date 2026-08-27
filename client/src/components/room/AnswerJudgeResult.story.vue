<script setup lang="ts">
import AnswerJudgeResult from './AnswerJudgeResult.vue'
import { answerJudgeResultFixtures } from './AnswerJudgeResult.fixture'
</script>

<template>
  <Story title="Room/AnswerJudgeResult">
    <Variant title="判定待ち">
      <div class="w-80 bg-[#eef4ff] p-4">
        <AnswerJudgeResult v-bind="answerJudgeResultFixtures.idle" />
      </div>
    </Variant>

    <Variant title="判定中">
      <div class="w-80 bg-[#eef4ff] p-4">
        <AnswerJudgeResult v-bind="answerJudgeResultFixtures.pending" />
      </div>
    </Variant>

    <Variant title="正解">
      <div class="w-80 bg-[#eef4ff] p-4">
        <AnswerJudgeResult v-bind="answerJudgeResultFixtures.correct" />
      </div>
    </Variant>

    <Variant title="不正解">
      <div class="w-80 bg-[#eef4ff] p-4">
        <AnswerJudgeResult v-bind="answerJudgeResultFixtures.incorrect" />
      </div>
    </Variant>

    <Variant title="エラー">
      <div class="w-80 bg-[#eef4ff] p-4">
        <AnswerJudgeResult v-bind="answerJudgeResultFixtures.error" />
      </div>
    </Variant>
  </Story>
</template>
