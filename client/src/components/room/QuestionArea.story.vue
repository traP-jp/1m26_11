<script setup lang="ts">
import QuestionArea from './QuestionArea.vue'
import { problemBodyAssetsFixture } from './ProblemBodyAssets.fixture'

const previewAsset = {
  type: 'image',
  url: '/favicon.ico',
  alt: problemBodyAssetsFixture.assets[0]?.alt ?? '問題資料',
}

const finalMarkdown = `## 最後の問題

これまでに集めた手掛かりを使って、答えを導いてください。`
</script>

<template>
  <Story title="Room/QuestionArea">
    <Variant title="小なぞ・問題資料あり">
      <div class="h-120 max-w-5xl">
        <QuestionArea
          :problem-number="1"
          title="最初の封筒"
          problem-type="small"
          :small-index="1"
          :small-total="3"
          :body-markdown="problemBodyAssetsFixture.bodyMarkdown"
          :assets="[previewAsset]"
        />
      </div>
    </Variant>

    <Variant title="大なぞ・Markdownのみ">
      <div class="h-96 max-w-3xl">
        <QuestionArea
          :problem-number="4"
          title="大なぞ"
          problem-type="final"
          :body-markdown="finalMarkdown"
          :assets="[]"
        />
      </div>
    </Variant>
  </Story>
</template>
