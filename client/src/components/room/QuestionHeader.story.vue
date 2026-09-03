<script setup lang="ts">
import QuestionHeader from './QuestionHeader.vue'
</script>

<template>
  <Story title="QuestionHeader">
    <Variant title="小なぞ (Small Problem)">
      <QuestionHeader :problemNumber="1" title="最初の封筒" problemType="small" :smallIndex="1" />
    </Variant>

    <Variant title="大なぞ (Final Problem)">
      <QuestionHeader :problemNumber="4" title="大なぞ" problemType="final" />
    </Variant>

    <Variant title="長いタイトル (Long Title)">
      <QuestionHeader
        :problemNumber="2"
        title="この問題のタイトルは非常に長く設定されており、画面幅が狭い場合や文字数が多い場合にどのように折り返されて表示されるかを確認するためのテスト用ダミーテキストです"
        problemType="small"
        :smallIndex="2"
        :smallTotal="4"
      />
    </Variant>
  </Story>
</template>
