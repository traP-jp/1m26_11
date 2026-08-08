<script setup lang="ts">
const stack = ['Vue 3', 'Vite', 'TypeScript', 'pnpm']
</script>

<template>
  <main class="app-shell">
    <section class="hero" aria-labelledby="app-title">
      <p class="eyebrow">1Monthon 2026 · Team 11</p>
      <h1 id="app-title">Frontend is ready.</h1>
      <p class="description">
        Vue と Rust server をつなぐための、シンプルな開発環境をセットアップしました。
      </p>

      <button
        class="rounded bg-sky-600 px-4 py-2 text-sm text-white hover:bg-sky-500 active:bg-sky-700 disabled:bg-gray-500">
        Save changes
      </button>

      <ul class="stack" aria-label="Frontend stack">
        <li v-for="item in stack" :key="item">{{ item }}</li>
      </ul>

      <a class="contract-link" href="/openapi.yaml">OpenAPI contract を確認</a>
    </section>
  </main>
</template>
