<script setup lang="ts">
import { computed, inject, reactive, ref, watch } from 'vue'

import { apiClient } from './api/client'
import {
  AUTHORING_CONTROLS,
  buildProblemSubmission,
  createDefaultProblemAuthoringForm,
  createEditableCandidate,
  createEditableOperation,
  type EditableOperation,
} from './authoring/problemForm'
import { ProblemAuthoringController } from './controllers/ProblemAuthoringController'
import { problemAuthoringApiClientKey } from './authoring/problemAuthoringApi'

const props = defineProps<{ roomId: string }>()

const controller = new ProblemAuthoringController(inject(problemAuthoringApiClientKey, apiClient))
const form = reactive(createDefaultProblemAuthoringForm())
const validationErrors = ref<string[]>([])

const busy = computed(
  () => controller.state.phase === 'creating' || controller.state.phase === 'uploading',
)
const locked = computed(() => controller.state.phase !== 'idle')
const canRetry = computed(
  () => controller.state.phase === 'create-error' || controller.state.phase === 'upload-error',
)

const phaseMessage = computed(() => {
  switch (controller.state.phase) {
    case 'creating':
      return '問題を作成しています…'
    case 'uploading':
      return '問題を作成しました。画像をアップロードしています…'
    case 'succeeded':
      return controller.state.imageSelected ? '問題と画像を作成しました。' : '問題を作成しました。'
    case 'create-error':
      return '問題の作成に失敗しました。'
    case 'upload-error':
      return '問題は作成されましたが、画像のアップロードに失敗しました。'
    case 'idle':
      return null
  }

  return null
})

watch(
  () => form.problemType,
  (problemType) => {
    if (problemType === 'final') {
      form.dependsOnProblemId = ''
    }
  },
)

function addOperation(operations: EditableOperation[]): void {
  operations.push(createEditableOperation())
}

function removeOperation(operations: EditableOperation[], index: number): void {
  operations.splice(index, 1)
}

function addCandidate(): void {
  form.candidates.push(createEditableCandidate(form.candidates.length))
}

function removeCandidate(index: number): void {
  form.candidates.splice(index, 1)
}

function handleImageChange(event: Event): void {
  const target = event.target
  if (!(target instanceof HTMLInputElement)) return

  form.imageFile = target.files?.item(0) ?? null
}

function submit(): void {
  validationErrors.value = []

  const result = buildProblemSubmission(form)
  if (!result.ok) {
    validationErrors.value = result.errors
    return
  }

  void controller.submit(props.roomId, result.request, result.image).catch(() => undefined)
}

function retry(): void {
  void controller.retry().catch(() => undefined)
}
</script>

<template>
  <main class="min-h-screen bg-slate-100 px-4 py-8 text-slate-900 sm:px-6">
    <div class="mx-auto max-w-5xl">
      <a href="/" class="text-sm font-bold text-sky-700 underline-offset-4 hover:underline">
        Portalへ戻る
      </a>

      <header class="mt-4">
        <p class="text-sm font-bold text-slate-500">Room ID: {{ roomId }}</p>
        <h1 class="mt-1 text-3xl font-black tracking-tight">問題を新規作成</h1>
        <p class="mt-2 text-sm text-slate-600">
          問題を作成した後、選択した画像を問題資料としてアップロードします。
        </p>
      </header>

      <div
        v-if="validationErrors.length > 0"
        class="mt-6 rounded-xl border border-red-300 bg-red-50 p-4 text-red-900"
        data-testid="authoring-validation-errors"
        role="alert"
      >
        <p class="font-bold">入力内容を確認してください。</p>
        <ul class="mt-2 list-disc space-y-1 pl-5">
          <li v-for="error in validationErrors" :key="error">{{ error }}</li>
        </ul>
      </div>

      <div
        v-if="phaseMessage"
        class="mt-6 rounded-xl border p-4"
        :class="
          controller.state.phase === 'succeeded'
            ? 'border-emerald-300 bg-emerald-50 text-emerald-900'
            : controller.state.error
              ? 'border-red-300 bg-red-50 text-red-900'
              : 'border-sky-300 bg-sky-50 text-sky-900'
        "
        data-testid="authoring-status"
        :role="controller.state.error ? 'alert' : 'status'"
      >
        <p class="font-bold">{{ phaseMessage }}</p>
        <p v-if="controller.state.problemId" class="mt-1 break-all text-sm">
          Problem ID: {{ controller.state.problemId }}
        </p>
        <p v-if="controller.state.error" class="mt-1 text-sm">
          {{ controller.state.error.message }}
        </p>
        <button
          v-if="canRetry"
          type="button"
          class="mt-3 rounded-lg bg-red-700 px-4 py-2 text-sm font-bold text-white hover:bg-red-800"
          data-testid="authoring-retry"
          @click="retry"
        >
          失敗した処理を再試行
        </button>
      </div>

      <form class="mt-6 space-y-6" @submit.prevent="submit">
        <fieldset :disabled="locked" class="space-y-6 disabled:opacity-70">
          <section class="rounded-2xl bg-white p-5 shadow-sm sm:p-6">
            <h2 class="text-xl font-black">基本情報</h2>

            <div class="mt-5 grid gap-5 sm:grid-cols-2">
              <label class="block">
                <span class="text-sm font-bold">問題番号</span>
                <input
                  v-model.number="form.number"
                  type="number"
                  min="1"
                  step="1"
                  class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
                  data-testid="authoring-number"
                />
              </label>

              <label class="block">
                <span class="text-sm font-bold">問題種別</span>
                <select
                  v-model="form.problemType"
                  class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
                  data-testid="authoring-problem-type"
                >
                  <option value="small">小問</option>
                  <option value="final">最終問題</option>
                </select>
              </label>
            </div>

            <label class="mt-5 block">
              <span class="text-sm font-bold">問題タイトル</span>
              <input
                v-model="form.title"
                type="text"
                maxlength="255"
                class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
                data-testid="authoring-title"
              />
            </label>

            <label class="mt-5 block">
              <span class="text-sm font-bold">問題文（Markdown）</span>
              <textarea
                v-model="form.bodyMarkdown"
                rows="8"
                class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 font-mono text-sm"
                data-testid="authoring-body"
              />
            </label>

            <div class="mt-5 grid gap-5 sm:grid-cols-2">
              <label v-if="form.problemType === 'small'" class="block">
                <span class="text-sm font-bold">依存先問題ID（任意）</span>
                <input
                  v-model="form.dependsOnProblemId"
                  type="text"
                  class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 font-mono text-sm"
                  placeholder="UUID"
                />
              </label>

              <label class="flex items-center gap-3 self-end py-2">
                <input v-model="form.isRequired" type="checkbox" class="size-4" />
                <span class="text-sm font-bold">クリアに必須の問題にする</span>
              </label>
            </div>
          </section>

          <section class="rounded-2xl bg-white p-5 shadow-sm sm:p-6">
            <h2 class="text-xl font-black">解答と判定</h2>

            <label class="mt-5 block">
              <span class="text-sm font-bold">解答形式</span>
              <select
                v-model="form.submissionType"
                class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
                data-testid="authoring-submission-type"
              >
                <option value="string">文字列</option>
                <option value="operation_sequence">操作列</option>
              </select>
            </label>

            <div v-if="form.submissionType === 'string'" class="mt-5">
              <label class="block">
                <span class="text-sm font-bold">正解文字列</span>
                <input
                  v-model="form.acceptedAnswer"
                  type="text"
                  maxlength="50"
                  class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
                  data-testid="authoring-answer"
                />
              </label>

              <div class="mt-4 flex flex-wrap gap-x-6 gap-y-3">
                <label class="flex items-center gap-2 text-sm">
                  <input v-model="form.trimOuterWhitespace" type="checkbox" class="size-4" />
                  前後の空白を無視
                </label>
                <label class="flex items-center gap-2 text-sm">
                  <input v-model="form.collapseInternalWhitespace" type="checkbox" class="size-4" />
                  連続する空白を1つにする
                </label>
                <label class="flex items-center gap-2 text-sm">
                  <input v-model="form.caseSensitive" type="checkbox" class="size-4" />
                  大文字と小文字を区別
                </label>
              </div>
            </div>

            <div v-else class="mt-6 space-y-6">
              <section>
                <div class="flex items-center justify-between gap-3">
                  <h3 class="font-black">正解操作列</h3>
                  <button
                    type="button"
                    class="rounded-lg border border-sky-700 px-3 py-1.5 text-sm font-bold text-sky-700"
                    @click="addOperation(form.correctOperations)"
                  >
                    操作を追加
                  </button>
                </div>

                <div class="mt-3 space-y-3">
                  <div
                    v-for="(operation, index) in form.correctOperations"
                    :key="index"
                    class="grid grid-cols-[1fr_7rem_auto] gap-2"
                  >
                    <select
                      v-model="operation.control"
                      :aria-label="`正解操作${index + 1}のボタン`"
                      class="rounded-lg border border-slate-300 px-3 py-2"
                    >
                      <option v-for="control in AUTHORING_CONTROLS" :key="control" :value="control">
                        {{ control }}
                      </option>
                    </select>
                    <input
                      v-model.number="operation.count"
                      type="number"
                      min="1"
                      step="1"
                      :aria-label="`正解操作${index + 1}の回数`"
                      class="rounded-lg border border-slate-300 px-3 py-2"
                    />
                    <button
                      type="button"
                      class="rounded-lg border border-red-300 px-3 py-2 text-sm font-bold text-red-700"
                      :aria-label="`正解操作${index + 1}を削除`"
                      @click="removeOperation(form.correctOperations, index)"
                    >
                      削除
                    </button>
                  </div>
                </div>
              </section>

              <section>
                <div class="flex items-center justify-between gap-3">
                  <h3 class="font-black">候補操作列</h3>
                  <button
                    type="button"
                    class="rounded-lg border border-sky-700 px-3 py-1.5 text-sm font-bold text-sky-700"
                    data-testid="authoring-add-candidate"
                    @click="addCandidate"
                  >
                    候補を追加
                  </button>
                </div>

                <div class="mt-3 space-y-4">
                  <article
                    v-for="(candidate, candidateIndex) in form.candidates"
                    :key="candidateIndex"
                    class="rounded-xl border border-slate-200 p-4"
                  >
                    <div class="flex items-end gap-3">
                      <label class="grow">
                        <span class="text-sm font-bold">候補ID</span>
                        <input
                          v-model="candidate.candidateId"
                          type="text"
                          :aria-label="`候補${candidateIndex + 1}のID`"
                          class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
                        />
                      </label>
                      <button
                        type="button"
                        class="rounded-lg border border-red-300 px-3 py-2 text-sm font-bold text-red-700"
                        :aria-label="`候補${candidateIndex + 1}を削除`"
                        @click="removeCandidate(candidateIndex)"
                      >
                        候補を削除
                      </button>
                    </div>

                    <div class="mt-3 space-y-2">
                      <div
                        v-for="(operation, operationIndex) in candidate.operations"
                        :key="operationIndex"
                        class="grid grid-cols-[1fr_7rem_auto] gap-2"
                      >
                        <select
                          v-model="operation.control"
                          :aria-label="`候補${candidateIndex + 1}の操作${operationIndex + 1}のボタン`"
                          class="rounded-lg border border-slate-300 px-3 py-2"
                        >
                          <option
                            v-for="control in AUTHORING_CONTROLS"
                            :key="control"
                            :value="control"
                          >
                            {{ control }}
                          </option>
                        </select>
                        <input
                          v-model.number="operation.count"
                          type="number"
                          min="1"
                          step="1"
                          :aria-label="`候補${candidateIndex + 1}の操作${operationIndex + 1}の回数`"
                          class="rounded-lg border border-slate-300 px-3 py-2"
                        />
                        <button
                          type="button"
                          class="rounded-lg border border-red-300 px-3 py-2 text-sm font-bold text-red-700"
                          :aria-label="`候補${candidateIndex + 1}の操作${operationIndex + 1}を削除`"
                          @click="removeOperation(candidate.operations, operationIndex)"
                        >
                          削除
                        </button>
                      </div>
                    </div>

                    <button
                      type="button"
                      class="mt-3 rounded-lg border border-sky-700 px-3 py-1.5 text-sm font-bold text-sky-700"
                      @click="addOperation(candidate.operations)"
                    >
                      この候補に操作を追加
                    </button>
                  </article>
                </div>
              </section>
            </div>
          </section>

          <section class="rounded-2xl bg-white p-5 shadow-sm sm:p-6">
            <h2 class="text-xl font-black">ヒントと画像</h2>

            <label class="mt-5 block">
              <span class="text-sm font-bold">ヒント（任意・Markdown）</span>
              <textarea
                v-model="form.hintMarkdown"
                rows="4"
                class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 font-mono text-sm"
              />
            </label>

            <label class="mt-5 block">
              <span class="text-sm font-bold">問題画像（任意）</span>
              <input
                type="file"
                accept="image/png,image/jpeg,image/webp"
                class="mt-1 block w-full text-sm"
                data-testid="authoring-image"
                @change="handleImageChange"
              />
              <span class="mt-1 block text-xs text-slate-500">PNG、JPEG、WebP／5 MiB以下</span>
            </label>

            <label v-if="form.imageFile" class="mt-5 block">
              <span class="text-sm font-bold">画像の代替テキスト</span>
              <input
                v-model="form.imageAlt"
                type="text"
                maxlength="200"
                class="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2"
              />
            </label>
          </section>
        </fieldset>

        <button
          type="submit"
          class="w-full rounded-xl bg-sky-700 px-5 py-3 font-black text-white hover:bg-sky-800 disabled:cursor-not-allowed disabled:bg-slate-400"
          data-testid="authoring-submit"
          :disabled="locked || busy"
        >
          {{ busy ? '処理中…' : '問題を作成' }}
        </button>
      </form>
    </div>
  </main>
</template>
