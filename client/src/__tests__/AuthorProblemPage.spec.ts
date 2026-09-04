import { flushPromises, mount } from '@vue/test-utils'
import { describe, expect, it, vi } from 'vitest'
import { createMemoryHistory } from 'vue-router'

import AuthorProblemPage from '../AuthorProblemPage.vue'
import { ApiClientError, type ProblemAuthoringApiClient } from '../api/client'
import { createAppRouter } from '../router'
import { problemAuthoringApiClientKey } from '../authoring/problemAuthoringApi'

const roomId = '1411824c-d357-4941-af76-c76cb827dda6'
const problemId = '22222222-2222-4222-8222-222222222222'

function createClient(): ProblemAuthoringApiClient {
  return {
    createProblem: vi
      .fn<ProblemAuthoringApiClient['createProblem']>()
      .mockResolvedValue({ problem_id: problemId }),
    uploadProblemAsset: vi.fn<ProblemAuthoringApiClient['uploadProblemAsset']>(),
  }
}

async function mountPage(client: ProblemAuthoringApiClient = createClient()) {
  const router = createAppRouter(createMemoryHistory())
  await router.push(`/author/rooms/${roomId}/problems/new`)

  const wrapper = mount(AuthorProblemPage, {
    props: { roomId },
    global: {
      plugins: [router],
      provide: {
        [problemAuthoringApiClientKey as symbol]: client,
      },
    },
  })
  await router.isReady()

  return { client, wrapper }
}

async function enterRequiredStringFields(
  wrapper: Awaited<ReturnType<typeof mountPage>>['wrapper'],
): Promise<void> {
  await wrapper.get('[data-testid="authoring-title"]').setValue('新しい問題')
  await wrapper.get('[data-testid="authoring-body"]').setValue('問題文です。')
  await wrapper.get('[data-testid="authoring-answer"]').setValue('正解')
}

describe('AuthorProblemPage', () => {
  it('shows validation errors without calling the API', async () => {
    const { client, wrapper } = await mountPage()

    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(wrapper.get('[data-testid="authoring-validation-errors"]').text()).toContain(
      '問題タイトルを入力してください。',
    )
    expect(wrapper.get('[data-testid="authoring-validation-errors"]').text()).toContain(
      '問題文を入力してください。',
    )
    expect(wrapper.get('[data-testid="authoring-validation-errors"]').text()).toContain(
      '正解文字列を入力してください。',
    )
    expect(client.createProblem).not.toHaveBeenCalled()
  })

  it('creates a string problem and shows the created problem ID', async () => {
    const { client, wrapper } = await mountPage()
    await enterRequiredStringFields(wrapper)

    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(client.createProblem).toHaveBeenCalledExactlyOnceWith(
      { room_id: roomId },
      { 'Idempotency-Key': expect.any(String) },
      expect.objectContaining({
        number: 1,
        problem_type: 'small',
        title: '新しい問題',
        body_markdown: '問題文です。',
        submission_type: 'string',
        judge_config: expect.objectContaining({
          type: 'string',
          accepted_answer: '正解',
        }),
      }),
    )
    expect(wrapper.get('[data-testid="authoring-status"]').text()).toContain('問題を作成しました。')
    expect(wrapper.get('[data-testid="authoring-status"]').text()).toContain(problemId)
  })

  it('builds an operation-sequence request from the visible operation fields', async () => {
    const { client, wrapper } = await mountPage()

    await wrapper.get('[data-testid="authoring-title"]').setValue('操作列問題')
    await wrapper.get('[data-testid="authoring-body"]').setValue('上ボタンを押してください。')
    await wrapper.get('[data-testid="authoring-submission-type"]').setValue('operation_sequence')
    await wrapper.get('input[aria-label="正解操作1の回数"]').setValue('2')
    await wrapper.get('input[aria-label="候補1の操作1の回数"]').setValue('2')

    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(client.createProblem).toHaveBeenCalledWith(
      { room_id: roomId },
      { 'Idempotency-Key': expect.any(String) },
      expect.objectContaining({
        submission_type: 'operation_sequence',
        judge_config: {
          type: 'operation_sequence',
          correct_operations: [{ control: 'up', count: 2 }],
          candidates: [
            {
              candidate_id: 'pattern-1',
              operations: [{ control: 'up', count: 2 }],
            },
          ],
        },
      }),
    )
  })

  it('retries a failed creation with the controller state kept intact', async () => {
    const client = createClient()
    vi.mocked(client.createProblem)
      .mockRejectedValueOnce(
        new ApiClientError('一時的に作成できませんでした', {
          kind: 'http',
          status: 500,
          code: 'internal_error',
          details: {},
        }),
      )
      .mockResolvedValueOnce({ problem_id: problemId })

    const { wrapper } = await mountPage(client)
    await enterRequiredStringFields(wrapper)

    await wrapper.get('form').trigger('submit')
    await flushPromises()

    expect(wrapper.get('[data-testid="authoring-status"]').text()).toContain(
      '問題の作成に失敗しました。',
    )

    await wrapper.get('[data-testid="authoring-retry"]').trigger('click')
    await flushPromises()

    expect(client.createProblem).toHaveBeenCalledTimes(2)
    expect(wrapper.get('[data-testid="authoring-status"]').text()).toContain('問題を作成しました。')
  })
})
