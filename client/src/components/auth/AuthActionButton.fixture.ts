import type AuthActionButton from './AuthActionButton.vue'

type Props = InstanceType<typeof AuthActionButton>['$props']

export const authActionButtonFixtures = {
  login: { action: 'login' },
  loginLink: { action: 'login', href: '/_oauth/login?redirect=/' },
  logout: { action: 'logout' },
  busy: { action: 'login', label: '処理中…', disabled: true },
} satisfies Record<string, Props>
