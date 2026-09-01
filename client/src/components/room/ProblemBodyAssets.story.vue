<script setup lang="ts">
import ProblemBodyAssets from './ProblemBodyAssets.vue'
import { problemBodyAssetsFixture } from './ProblemBodyAssets.fixture'

const previewAsset = {
  type: 'image',
  url: '/favicon.ico',
  alt: problemBodyAssetsFixture.assets[0]?.alt ?? '問題資料',
}

const richMarkdown = `## 操作のルール

次の条件をすべて満たすように、**矢印キー**を操作してください。

1. 赤いマスを通る
2. 青いマスには入らない

> 操作回数には上限があります。

| キー | 動作 |
| --- | --- |
| \`↑\` | 上へ移動 |
| \`→\` | 右へ移動 |`
</script>

<template>
  <Story title="Room/ProblemBodyAssets">
    <Variant title="問題文と画像">
      <div class="max-w-5xl rounded-2xl bg-white p-6 shadow-sm">
        <ProblemBodyAssets
          :body-markdown="problemBodyAssetsFixture.bodyMarkdown"
          :assets="[previewAsset]"
        />
      </div>
    </Variant>

    <Variant title="Markdownのみ">
      <div class="max-w-3xl rounded-2xl bg-white p-6 shadow-sm">
        <ProblemBodyAssets :body-markdown="richMarkdown" :assets="[]" />
      </div>
    </Variant>

    <Variant title="複数画像">
      <div class="max-w-5xl rounded-2xl bg-white p-6 shadow-sm">
        <ProblemBodyAssets
          :body-markdown="problemBodyAssetsFixture.bodyMarkdown"
          :assets="[
            { ...previewAsset, alt: '問題資料 1' },
            { ...previewAsset, alt: '問題資料 2' },
            { ...previewAsset, alt: '問題資料 3' },
          ]"
        />
      </div>
    </Variant>

    <Variant title="取得失敗・未対応">
      <div class="max-w-5xl rounded-2xl bg-white p-6 shadow-sm">
        <ProblemBodyAssets
          :body-markdown="problemBodyAssetsFixture.bodyMarkdown"
          :assets="[
            { type: 'image', url: '/missing-problem-asset.png', alt: '取得できない画像' },
            { type: 'video', url: 'https://example.com/problem.mp4', alt: '未対応の資料' },
          ]"
        />
      </div>
    </Variant>
  </Story>
</template>
