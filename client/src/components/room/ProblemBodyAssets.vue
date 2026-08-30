<script setup lang="ts">
import { computed, reactive, ref, watch } from 'vue'
import { Dialog, DialogPanel, DialogTitle } from '@headlessui/vue'
import DOMPurify from 'dompurify'
import MarkdownIt from 'markdown-it'

import type { ProblemBodyAssetsProps } from './ProblemBodyAssets.types'

defineOptions({ inheritAttrs: false })

const props = defineProps<ProblemBodyAssetsProps>()
const urlPolicyBase = new URL('https://problem-assets.invalid/')

interface DisplayAsset {
  alt: string
  label: string
  key: string
  url: string
  renderable: boolean
}

const markdown = new MarkdownIt({
  breaks: false,
  html: false,
  linkify: false,
  typographer: false,
})

markdown.validateLink = isSafeUrl
markdown.renderer.rules.image = (tokens, index) =>
  markdown.utils.escapeHtml(tokens[index]?.content ?? '')
markdown.renderer.rules.link_open = (tokens, index, options, _env, renderer) => {
  const token = tokens[index]
  token?.attrSet('referrerpolicy', 'no-referrer')
  token?.attrSet('rel', 'noreferrer')
  return renderer.renderToken(tokens, index, options)
}

for (const rule of ['heading_open', 'heading_close'] as const) {
  markdown.renderer.rules[rule] = (tokens, index, options, _env, renderer) => {
    const token = tokens[index]
    if (token?.tag === 'h1') token.tag = 'h2'
    return renderer.renderToken(tokens, index, options)
  }
}

const renderedBody = computed(() =>
  DOMPurify.sanitize(markdown.render(props.bodyMarkdown), {
    ALLOWED_ATTR: ['href', 'referrerpolicy', 'rel', 'title'],
    ALLOWED_TAGS: [
      'a',
      'blockquote',
      'br',
      'code',
      'em',
      'h2',
      'h3',
      'h4',
      'h5',
      'h6',
      'hr',
      'li',
      'ol',
      'p',
      'pre',
      's',
      'strong',
      'table',
      'tbody',
      'td',
      'th',
      'thead',
      'tr',
      'ul',
    ],
    ALLOW_ARIA_ATTR: false,
    ALLOW_DATA_ATTR: false,
    FORBID_ATTR: ['id', 'name', 'src', 'srcset', 'style', 'target'],
    FORBID_TAGS: ['img', 'math', 'style', 'svg'],
  }),
)

const displayAssets = computed<DisplayAsset[]>(() =>
  props.assets.map((asset, index) => {
    const url = asset.url.trim()
    return {
      alt: asset.alt,
      label: asset.alt.trim() || '問題資料',
      key: JSON.stringify([index, asset.type, url]),
      url,
      renderable: asset.type === 'image' && isSafeUrl(url),
    }
  }),
)

const failedAssetKeys = reactive(new Set<string>())
const expandedAssetKey = ref<string | null>(null)
const expandedAsset = computed(
  () =>
    displayAssets.value.find(
      (asset) =>
        asset.key === expandedAssetKey.value && asset.renderable && !failedAssetKeys.has(asset.key),
    ) ?? null,
)

watch(displayAssets, (assets) => {
  if (expandedAssetKey.value && !assets.some((asset) => asset.key === expandedAssetKey.value)) {
    expandedAssetKey.value = null
  }
})

function isSafeUrl(value: string): boolean {
  const url = value.trim()
  if (!url || url.startsWith('//')) return false

  try {
    const parsed = new URL(url, urlPolicyBase)
    if (parsed.protocol !== 'https:' || parsed.username || parsed.password) return false

    const hasExplicitScheme = /^[a-z][a-z\d+.-]*:/i.test(url)
    return hasExplicitScheme || parsed.origin === urlPolicyBase.origin
  } catch {
    return false
  }
}

function markAssetFailed(asset: DisplayAsset) {
  failedAssetKeys.add(asset.key)
  if (expandedAssetKey.value === asset.key) expandedAssetKey.value = null
}

function openAsset(asset: DisplayAsset) {
  if (!asset.renderable || failedAssetKeys.has(asset.key)) return
  expandedAssetKey.value = asset.key
}

function closeAsset() {
  expandedAssetKey.value = null
}
</script>

<template>
  <div
    class="problem-body-assets grid min-w-0 gap-6 text-[#121a2a]"
    :class="displayAssets.length > 0 ? 'lg:grid-cols-[minmax(0,2fr)_minmax(0,3fr)]' : ''"
  >
    <!-- eslint-disable-next-line vue/no-v-html -- output is sanitized with a strict allowlist -->
    <div
      class="problem-markdown min-w-0 leading-7"
      data-testid="problem-markdown"
      v-html="renderedBody"
    />

    <section v-if="displayAssets.length > 0" class="min-w-0" aria-label="問題資料">
      <ul class="grid gap-3" :class="displayAssets.length === 1 ? 'grid-cols-1' : 'grid-cols-2'">
        <li v-for="displayAsset in displayAssets" :key="displayAsset.key" class="min-w-0">
          <button
            v-if="displayAsset.renderable && !failedAssetKeys.has(displayAsset.key)"
            type="button"
            class="group relative flex aspect-[16/10] w-full items-center justify-center overflow-hidden rounded-xl border border-[#c8d5e8] bg-[#eef4ff] p-2 text-left shadow-sm outline-none transition hover:border-[#8da9d2] hover:shadow-md focus-visible:ring-2 focus-visible:ring-[#2864e8] focus-visible:ring-offset-2"
            :aria-label="`画像を拡大: ${displayAsset.label}`"
            @click="openAsset(displayAsset)"
          >
            <img
              class="h-full w-full object-contain"
              :src="displayAsset.url"
              :alt="displayAsset.alt"
              referrerpolicy="no-referrer"
              @error="markAssetFailed(displayAsset)"
            />
            <span
              class="absolute right-2 bottom-2 rounded-full bg-[#121a2a]/85 px-2.5 py-1 text-xs font-bold text-white opacity-0 shadow-sm transition group-hover:opacity-100 group-focus-visible:opacity-100"
              aria-hidden="true"
            >
              画像を拡大
            </span>
          </button>

          <div
            v-else
            class="flex aspect-[16/10] w-full flex-col items-center justify-center rounded-xl border border-dashed border-[#b7c6dc] bg-[#f7f9fd] px-4 text-center"
            data-testid="asset-fallback"
            :role="displayAsset.renderable ? 'status' : undefined"
          >
            <p class="text-sm font-bold text-[#52627a]">
              {{
                displayAsset.renderable ? '画像を読み込めませんでした' : '表示できない問題資料です'
              }}
            </p>
            <p class="mt-1 line-clamp-2 text-xs text-[#52627a]">
              {{ displayAsset.label }}
            </p>
          </div>
        </li>
      </ul>
    </section>

    <Dialog :open="expandedAsset !== null" class="relative z-50" @close="closeAsset">
      <div class="fixed inset-0 bg-[#121a2a]/70 backdrop-blur-sm" aria-hidden="true" />

      <div class="fixed inset-0 overflow-y-auto p-4 sm:p-8">
        <div class="flex min-h-full items-center justify-center">
          <DialogPanel
            v-if="expandedAsset"
            class="w-full max-w-5xl overflow-hidden rounded-2xl bg-white shadow-2xl"
          >
            <div
              class="flex items-center justify-between gap-4 border-b border-[#d7e1ef] px-4 py-3 sm:px-6"
            >
              <DialogTitle class="min-w-0 truncate text-base font-bold text-[#121a2a]">
                {{ expandedAsset.label }}
              </DialogTitle>
              <button
                type="button"
                class="shrink-0 rounded-lg border border-[#c8d5e8] bg-white px-3 py-1.5 text-sm font-bold text-[#36465f] outline-none transition hover:bg-[#eef4ff] focus-visible:ring-2 focus-visible:ring-[#2864e8] focus-visible:ring-offset-2"
                @click="closeAsset"
              >
                閉じる
              </button>
            </div>
            <div
              class="flex max-h-[calc(100vh-9rem)] min-h-60 items-center justify-center bg-[#eef4ff] p-4 sm:p-6"
            >
              <img
                class="max-h-[calc(100vh-12rem)] max-w-full object-contain"
                :src="expandedAsset.url"
                alt=""
                referrerpolicy="no-referrer"
                @error="markAssetFailed(expandedAsset)"
              />
            </div>
          </DialogPanel>
        </div>
      </div>
    </Dialog>
  </div>
</template>

<style scoped>
.problem-markdown :deep(> :first-child) {
  margin-top: 0;
}

.problem-markdown :deep(> :last-child) {
  margin-bottom: 0;
}

.problem-markdown :deep(p),
.problem-markdown :deep(ul),
.problem-markdown :deep(ol),
.problem-markdown :deep(blockquote),
.problem-markdown :deep(pre),
.problem-markdown :deep(table) {
  margin: 0.85rem 0;
}

.problem-markdown :deep(h2),
.problem-markdown :deep(h3),
.problem-markdown :deep(h4),
.problem-markdown :deep(h5),
.problem-markdown :deep(h6) {
  margin: 1.4rem 0 0.65rem;
  color: #121a2a;
  font-weight: 700;
  line-height: 1.4;
}

.problem-markdown :deep(h2) {
  font-size: 1.35rem;
}

.problem-markdown :deep(h3) {
  font-size: 1.15rem;
}

.problem-markdown :deep(ul),
.problem-markdown :deep(ol) {
  padding-left: 1.5rem;
}

.problem-markdown :deep(ul) {
  list-style: disc;
}

.problem-markdown :deep(ol) {
  list-style: decimal;
}

.problem-markdown :deep(li + li) {
  margin-top: 0.3rem;
}

.problem-markdown :deep(blockquote) {
  border-left: 3px solid #8da9d2;
  padding: 0.15rem 0 0.15rem 1rem;
  color: #52627a;
}

.problem-markdown :deep(code) {
  border-radius: 0.3rem;
  background: #eef4ff;
  padding: 0.1rem 0.35rem;
  font-size: 0.9em;
}

.problem-markdown :deep(pre) {
  overflow-x: auto;
  border-radius: 0.75rem;
  background: #121a2a;
  padding: 1rem;
  color: #f7f9fd;
}

.problem-markdown :deep(pre code) {
  background: transparent;
  padding: 0;
  color: inherit;
}

.problem-markdown :deep(a) {
  color: #2864e8;
  font-weight: 600;
  text-decoration: underline;
  text-decoration-thickness: 0.08em;
  text-underline-offset: 0.16em;
}

.problem-markdown :deep(a:hover) {
  color: #174ab4;
}

.problem-markdown :deep(table) {
  display: block;
  max-width: 100%;
  overflow-x: auto;
  border-collapse: collapse;
}

.problem-markdown :deep(th),
.problem-markdown :deep(td) {
  border: 1px solid #c8d5e8;
  padding: 0.45rem 0.65rem;
  text-align: left;
}

.problem-markdown :deep(th) {
  background: #eef4ff;
  font-weight: 700;
}
</style>
