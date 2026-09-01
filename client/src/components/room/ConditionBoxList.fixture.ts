import type { ConditionBoxItem } from './ConditionBoxList.types'

export const firstConditionBoxItem = { id: 'condition-up', label: '上を2回' }

export const conditionBoxListFixture = {
  items: [firstConditionBoxItem, { id: 'condition-right', label: '右を1回' }],
} satisfies { items: ConditionBoxItem[] }
