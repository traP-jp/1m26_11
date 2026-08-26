<script setup lang="ts">
import { Menu, MenuButton, MenuItem, MenuItems } from '@headlessui/vue'

const stack = ['Vue 3', 'Vite', 'TypeScript', 'pnpm']

const actions = ['Save changes', 'Duplicate', 'Archive']
</script>

<template>
  <main class="app-shell">
    <section class="hero" aria-labelledby="app-name">
      <p class="eyebrow">1Monthon 2026 · Team 11</p>
      <h1 id="app-name">Frontend is ready.</h1>
      <p class="description">
        Vue と Rust server をつなぐための、シンプルな開発環境をセットアップしました。
      </p>

      <Menu as="div" class="relative inline-block py-1 text-left">
        <MenuButton
          class="rounded bg-sky-600 px-4 py-2 text-sm font-medium text-white shadow-sm transition hover:bg-sky-500 active:bg-sky-700">
          Open actions
        </MenuButton>

        <MenuItems
          class="absolute right-0 z-10 mt-3 w-56 origin-top-right overflow-hidden rounded-xl border border-white/10 bg-slate-900/95 p-1 shadow-2xl ring-1 ring-black/20 backdrop-blur">
          <MenuItem v-for="action in actions" :key="action" v-slot="{ active }">
            <button :class="[
              'block w-full rounded-lg px-4 py-3 text-left text-sm transition',
              active ? 'bg-sky-500 text-white' : 'text-slate-200 hover:bg-white/5',
            ]">
              {{ action }}
            </button>
          </MenuItem>
        </MenuItems>
      </Menu>

      <ul class="stack" aria-label="Frontend stack">
        <li v-for="item in stack" :key="item">{{ item }}</li>
      </ul>

      <a class="contract-link" href="/openapi.yaml">OpenAPI contract を確認</a>
    </section>
  </main>
</template>
