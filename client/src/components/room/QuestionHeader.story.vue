<script setup lang="ts">
import QuestionHeader from './QuestionHeader.vue'

const smallProblemMock = {
  problem_id: '52ed5a58-bc88-4e0f-97a4-0f64a112acd4',
  room_id: '1411824c-d357-4941-af76-c76cb827dda6',
  number: 1,
  problem_type: 'small' as const,
  title: '最初の封筒',
  body_markdown:
    '机の上に、数字と記号が並んだ古い封筒が置かれている。\n条件Boxの内容を組み合わせ、封筒を開くための答えを導いてください。',
}

const finalProblemMock = {
  problem_id: '9ca65619-6ad2-4e74-bf4a-4f146b238067',
  room_id: '1411824c-d357-4941-af76-c76cb827dda6',
  number: 4,
  problem_type: 'final' as const,
  title: '大なぞ',
  body_markdown: 'これまでの手がかりを全て組み合わせ、最後の答えを導き出してください。',
}
</script>

<template>
  <Story title="QuestionHeader">
    <Variant title="小なぞ (Small Problem)">
      <QuestionHeader :problem="smallProblemMock" />
    </Variant>

    <Variant title="大なぞ (Final Problem)">
      <QuestionHeader :problem="finalProblemMock" />
    </Variant>
  </Story>
</template>
