import { describe, expect, it } from 'vitest'

import {
  buildProblemSubmission,
  createDefaultProblemAuthoringForm,
  createEditableCandidate,
  createEditableOperation,
} from '../problemForm'

describe('problem authoring form', () => {
  it('requires the minimum string problem fields', () => {
    const result = buildProblemSubmission(createDefaultProblemAuthoringForm())

    expect(result).toEqual({
      ok: false,
      errors: [
        '問題タイトルを入力してください。',
        '問題文を入力してください。',
        '正解文字列を入力してください。',
      ],
    })
  })

  it('builds a string problem request', () => {
    const form = createDefaultProblemAuthoringForm()
    form.number = 4
    form.problemType = 'final'
    form.title = '最後の問題'
    form.bodyMarkdown = '問題文です。'
    form.acceptedAnswer = 'ワンマンソン'
    form.hintMarkdown = 'ヒントです。'
    form.caseSensitive = false

    const result = buildProblemSubmission(form)

    expect(result).toMatchObject({
      ok: true,
      request: {
        number: 4,
        problem_type: 'final',
        title: '最後の問題',
        body_markdown: '問題文です。',
        submission_type: 'string',
        depends_on_problem_id: null,
        is_required: true,
        hints: [{ body_markdown: 'ヒントです。' }],
        judge_config: {
          type: 'string',
          accepted_answer: 'ワンマンソン',
          normalization: {
            unicode: 'nfkc',
            trim_outer_whitespace: true,
            collapse_internal_whitespace: false,
            case_sensitive: false,
          },
        },
      },
      image: null,
    })
  })

  it('normalizes adjacent operations and builds candidates', () => {
    const form = createDefaultProblemAuthoringForm()
    form.title = '操作列問題'
    form.bodyMarkdown = 'ボタンを操作してください。'
    form.submissionType = 'operation_sequence'
    form.correctOperations = [
      createEditableOperation('down'),
      createEditableOperation('down'),
      { control: 'right', count: 2 },
    ]
    form.candidates = [
      {
        candidateId: ' pattern-a ',
        operations: [
          { control: 'down', count: 2 },
          { control: 'right', count: 2 },
        ],
      },
      createEditableCandidate(1),
    ]

    const result = buildProblemSubmission(form)

    expect(result).toMatchObject({
      ok: true,
      request: {
        submission_type: 'operation_sequence',
        judge_config: {
          type: 'operation_sequence',
          correct_operations: [
            { control: 'down', count: 2 },
            { control: 'right', count: 2 },
          ],
          candidates: [
            {
              candidate_id: 'pattern-a',
              operations: [
                { control: 'down', count: 2 },
                { control: 'right', count: 2 },
              ],
            },
            {
              candidate_id: 'pattern-2',
              operations: [{ control: 'up', count: 1 }],
            },
          ],
        },
      },
    })
  })

  it('rejects invalid operation candidates and final-problem dependencies', () => {
    const form = createDefaultProblemAuthoringForm()
    form.problemType = 'final'
    form.dependsOnProblemId = '22222222-2222-4222-8222-222222222221'
    form.title = '操作列問題'
    form.bodyMarkdown = '問題文です。'
    form.submissionType = 'operation_sequence'
    form.correctOperations = [{ control: 'up', count: 101 }]
    form.candidates = [
      {
        candidateId: 'same',
        operations: [{ control: 'left', count: 1 }],
      },
      {
        candidateId: ' same ',
        operations: [{ control: 'right', count: 0 }],
      },
    ]

    const result = buildProblemSubmission(form)

    expect(result.ok).toBe(false)
    if (result.ok) throw new Error('validationが成功してしまいました')

    expect(result.errors).toContain('最終問題には依存先問題を指定できません。')
    expect(result.errors).toContain('正解操作列の合計回数は100以下にしてください。')
    expect(result.errors).toContain('候補ID「same」が重複しています。')
    expect(result.errors).toContain('候補2の操作列の1件目の回数は1以上の整数にしてください。')
    expect(result.errors).toContain('正解操作列と一致する候補を1件以上作成してください。')
  })

  it('returns the selected image after validating its alt text', () => {
    const form = createDefaultProblemAuthoringForm()
    form.title = '画像問題'
    form.bodyMarkdown = '画像を見てください。'
    form.acceptedAnswer = '答え'
    form.imageFile = new File(['image'], 'question.png', {
      type: 'image/png',
    })
    form.imageAlt = '  問題画像  '

    const result = buildProblemSubmission(form)

    expect(result.ok).toBe(true)
    if (!result.ok) throw new Error('validationに失敗しました')

    expect(result.image).toEqual({
      file: form.imageFile,
      alt: '問題画像',
    })
  })
})
