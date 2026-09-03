<script setup lang="ts">
import { computed, ref, useId } from 'vue'

import {
  keyboardInputSource,
  screenButtonInputSource,
  type AlternativeInputSource,
} from '@/input/InputAdapter.types'

const props = defineProps<{
  maxLength: number
  pending: boolean
  disabled: boolean
}>()

const emit = defineEmits<{
  submit: [answer: string, source: AlternativeInputSource]
}>()

const answer = ref('')
const answerInputId = useId()
const helpId = `${answerInputId}-help`
const countId = `${answerInputId}-count`

const interactionDisabled = computed(() => props.pending || props.disabled)
const answerTooLong = computed(() => answer.value.length > props.maxLength)
const submitDisabled = computed(() => interactionDisabled.value || answerTooLong.value)

function submitAnswer(source: AlternativeInputSource) {
  if (submitDisabled.value) return
  emit('submit', answer.value, source)
}

function handleAnswerKeydown(event: KeyboardEvent) {
  if (
    event.key !== 'Enter' ||
    event.altKey ||
    event.ctrlKey ||
    event.metaKey ||
    event.shiftKey ||
    event.isComposing
  ) {
    return
  }

  event.preventDefault()
  if (event.repeat) return

  submitAnswer(keyboardInputSource)
}
</script>

<template>
  <section
    id="answer-panel"
    class="rounded-xl border border-[#c8d5e8] bg-[#eef4ff] p-4 text-[#121a2a] sm:p-5"
    aria-labelledby="answer-panel-title"
    :aria-busy="pending"
  >
    <header class="mb-4">
      <h2 id="answer-panel-title" class="text-base font-extrabold">回答パネル</h2>
    </header>

    <form
      class="rounded-xl border border-[#c8d5e8] bg-white p-4 sm:p-5"
      @submit.prevent="submitAnswer(screenButtonInputSource)"
    >
      <label :for="answerInputId" class="mb-3 block text-sm font-bold">回答</label>
      <textarea
        :id="answerInputId"
        v-model="answer"
        name="answer"
        rows="6"
        :maxlength="maxLength"
        :disabled="interactionDisabled"
        :aria-describedby="`${helpId} ${countId}`"
        class="min-h-36 w-full resize-y rounded-lg border border-[#7aa7ff] bg-[#fbfcff] px-4 py-3 text-sm text-[#121a2a] outline-none transition-shadow placeholder:text-[#8da0bd] focus:border-[#2e6bea] focus:ring-2 focus:ring-[#2e6bea]/20 disabled:cursor-not-allowed disabled:border-[#d5dce7] disabled:bg-[#f1f4f8] disabled:text-[#78869a]"
        placeholder="答えを入力してください"
        @keydown="handleAnswerKeydown"
      />

      <div class="mt-2 flex flex-wrap items-start justify-between gap-2 text-xs text-[#65758d]">
        <p :id="helpId">最大{{ maxLength }}文字。Enterで送信、Shift+Enterで改行します。</p>
        <p :id="countId" :class="answerTooLong ? 'font-bold text-red-700' : ''">
          <span class="sr-only">入力文字数 </span>
          {{ answer.length }}/{{ maxLength }}
        </p>
      </div>

      <div class="mt-4 flex items-center justify-between gap-4">
        <p class="text-xs font-bold text-[#2764d8]" aria-live="polite">
          {{ pending ? '送信中…' : disabled ? '入力できません' : '入力待ち' }}
        </p>
        <button
          type="submit"
          :disabled="submitDisabled"
          class="inline-flex min-h-10 min-w-20 items-center justify-center rounded-lg bg-[#2864e8] px-5 text-sm font-bold text-white transition-colors hover:bg-[#1f56cc] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[#3997ea] disabled:cursor-not-allowed disabled:bg-[#a7b8d8]"
        >
          {{ pending ? '送信中…' : '送信' }}
        </button>
      </div>
    </form>
  </section>
</template>
