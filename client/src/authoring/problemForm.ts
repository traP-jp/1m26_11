import type { CreateProblemRequest } from '@/api/client'
import type { ProblemAssetDraft } from '@/controllers/ProblemAuthoringController'
import type { components } from '@/generated/api'

type Operation = components['schemas']['Operation']

export const AUTHORING_CONTROLS = ['up', 'down', 'left', 'right', 'red', 'yellow', 'green'] as const

const MAX_IMAGE_BYTES = 5_242_880
const SUPPORTED_IMAGE_TYPES = new Set(['image/png', 'image/jpeg', 'image/webp'])
const UUID_PATTERN = /^[0-9a-f]{8}-[0-9a-f]{4}-[1-5][0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i

export interface EditableOperation {
  control: string
  count: number
}

export interface EditableCandidate {
  candidateId: string
  operations: EditableOperation[]
}

export interface ProblemAuthoringForm {
  number: number
  problemType: CreateProblemRequest['problem_type']
  title: string
  bodyMarkdown: string
  submissionType: CreateProblemRequest['submission_type']
  acceptedAnswer: string
  trimOuterWhitespace: boolean
  collapseInternalWhitespace: boolean
  caseSensitive: boolean
  correctOperations: EditableOperation[]
  candidates: EditableCandidate[]
  dependsOnProblemId: string
  isRequired: boolean
  hintMarkdown: string
  imageFile: File | null
  imageAlt: string
}

export type BuildProblemSubmissionResult =
  | {
      ok: true
      request: CreateProblemRequest
      image: ProblemAssetDraft | null
    }
  | {
      ok: false
      errors: string[]
    }

export function createEditableOperation(control = 'up'): EditableOperation {
  return { control, count: 1 }
}

export function createEditableCandidate(index: number): EditableCandidate {
  return {
    candidateId: `pattern-${index + 1}`,
    operations: [createEditableOperation()],
  }
}

export function createDefaultProblemAuthoringForm(): ProblemAuthoringForm {
  return {
    number: 1,
    problemType: 'small',
    title: '',
    bodyMarkdown: '',
    submissionType: 'string',
    acceptedAnswer: '',
    trimOuterWhitespace: true,
    collapseInternalWhitespace: false,
    caseSensitive: false,
    correctOperations: [createEditableOperation()],
    candidates: [createEditableCandidate(0)],
    dependsOnProblemId: '',
    isRequired: true,
    hintMarkdown: '',
    imageFile: null,
    imageAlt: '',
  }
}

function characterCount(value: string): number {
  return Array.from(value).length
}

function validateOperations(
  operations: EditableOperation[],
  label: string,
  errors: string[],
): void {
  if (operations.length === 0) {
    errors.push(`${label}を1件以上入力してください。`)
    return
  }

  let operationCount = 0

  operations.forEach((operation, index) => {
    if (!AUTHORING_CONTROLS.includes(operation.control as (typeof AUTHORING_CONTROLS)[number])) {
      errors.push(`${label}${index + 1}件目の操作が不正です。`)
    }
    if (!Number.isInteger(operation.count) || operation.count <= 0) {
      errors.push(`${label}${index + 1}件目の回数は1以上の整数にしてください。`)
    } else {
      operationCount += operation.count
    }
  })

  if (operationCount > 100) {
    errors.push(`${label}合計回数は100以下にしてください。`)
  }
}

function normalizeOperations(operations: EditableOperation[]): Operation[] {
  const normalized: Operation[] = []

  for (const operation of operations) {
    const previous = normalized[normalized.length - 1]
    if (previous?.control === operation.control) {
      previous.count += operation.count
    } else {
      normalized.push({
        control: operation.control,
        count: operation.count,
      })
    }
  }

  return normalized
}

function operationsMatch(left: EditableOperation[], right: EditableOperation[]): boolean {
  return JSON.stringify(normalizeOperations(left)) === JSON.stringify(normalizeOperations(right))
}

function validateImage(form: ProblemAuthoringForm, errors: string[]): void {
  if (form.imageFile === null) return

  if (form.imageFile.size === 0) {
    errors.push('画像ファイルが空です。')
  }
  if (form.imageFile.size > MAX_IMAGE_BYTES) {
    errors.push('画像ファイルは5 MiB以下にしてください。')
  }
  if (!SUPPORTED_IMAGE_TYPES.has(form.imageFile.type)) {
    errors.push('画像はPNG、JPEG、WebPのいずれかにしてください。')
  }

  const alt = form.imageAlt.trim()
  if (alt.length === 0) {
    errors.push('画像を選択した場合は代替テキストを入力してください。')
  } else if (characterCount(alt) > 200) {
    errors.push('画像の代替テキストは200文字以下にしてください。')
  }
}

function validateForm(form: ProblemAuthoringForm): string[] {
  const errors: string[] = []

  if (!Number.isInteger(form.number) || form.number <= 0) {
    errors.push('問題番号は1以上の整数にしてください。')
  }

  if (form.title.trim().length === 0) {
    errors.push('問題タイトルを入力してください。')
  } else if (characterCount(form.title) > 255) {
    errors.push('問題タイトルは255文字以下にしてください。')
  }

  if (form.bodyMarkdown.trim().length === 0) {
    errors.push('問題文を入力してください。')
  }

  const dependency = form.dependsOnProblemId.trim()
  if (form.problemType === 'final' && dependency.length > 0) {
    errors.push('最終問題には依存先問題を指定できません。')
  } else if (dependency.length > 0 && !UUID_PATTERN.test(dependency)) {
    errors.push('依存先問題IDをUUID形式で入力してください。')
  }

  if (form.submissionType === 'string') {
    if (form.acceptedAnswer.trim().length === 0) {
      errors.push('正解文字列を入力してください。')
    } else if (characterCount(form.acceptedAnswer) > 50) {
      errors.push('正解文字列は50文字以下にしてください。')
    }
  } else {
    validateOperations(form.correctOperations, '正解操作列の', errors)

    if (form.candidates.length === 0) {
      errors.push('候補を1件以上入力してください。')
    }

    const candidateIds = new Set<string>()
    for (const [index, candidate] of form.candidates.entries()) {
      const candidateId = candidate.candidateId.trim()
      if (candidateId.length === 0) {
        errors.push(`候補${index + 1}のIDを入力してください。`)
      } else if (candidateIds.has(candidateId)) {
        errors.push(`候補ID「${candidateId}」が重複しています。`)
      } else {
        candidateIds.add(candidateId)
      }

      validateOperations(candidate.operations, `候補${index + 1}の操作列の`, errors)
    }

    if (
      form.correctOperations.length > 0 &&
      !form.candidates.some((candidate) =>
        operationsMatch(form.correctOperations, candidate.operations),
      )
    ) {
      errors.push('正解操作列と一致する候補を1件以上作成してください。')
    }
  }

  validateImage(form, errors)
  return errors
}

export function buildProblemSubmission(form: ProblemAuthoringForm): BuildProblemSubmissionResult {
  const errors = validateForm(form)
  if (errors.length > 0) return { ok: false, errors }

  const hints = form.hintMarkdown.trim().length === 0 ? [] : [{ body_markdown: form.hintMarkdown }]

  const judgeConfig: CreateProblemRequest['judge_config'] =
    form.submissionType === 'string'
      ? {
          type: 'string',
          accepted_answer: form.acceptedAnswer,
          normalization: {
            unicode: 'nfkc',
            trim_outer_whitespace: form.trimOuterWhitespace,
            collapse_internal_whitespace: form.collapseInternalWhitespace,
            case_sensitive: form.caseSensitive,
          },
        }
      : {
          type: 'operation_sequence',
          correct_operations: normalizeOperations(form.correctOperations),
          candidates: form.candidates.map((candidate) => ({
            candidate_id: candidate.candidateId.trim(),
            operations: normalizeOperations(candidate.operations),
          })),
        }

  const request: CreateProblemRequest = {
    number: form.number,
    problem_type: form.problemType,
    title: form.title,
    body_markdown: form.bodyMarkdown,
    submission_type: form.submissionType,
    input_schema: {
      query: {
        type: 'operation_sequence',
        allowed_controls: [...AUTHORING_CONTROLS],
        max_operations: 100,
      },
      answer: {
        type: 'string',
        max_length: 50,
      },
    },
    hints,
    judge_config: judgeConfig,
    depends_on_problem_id:
      form.problemType === 'small' && form.dependsOnProblemId.trim().length > 0
        ? form.dependsOnProblemId.trim()
        : null,
    is_required: form.isRequired,
  }

  return {
    ok: true,
    request,
    image:
      form.imageFile === null
        ? null
        : {
            file: form.imageFile,
            alt: form.imageAlt.trim(),
          },
  }
}
