import {
  createRouter,
  createWebHistory,
  type Router,
  type RouterHistory,
  type RouteRecordRaw,
} from 'vue-router'

import ClearPage from './ClearPage.vue'
import PortalPage from './PortalPage.vue'
import RoomPage from './RoomPage.vue'

const developmentRoutes: RouteRecordRaw[] = import.meta.env.DEV
  ? [
      {
        path: '/device-poc',
        name: 'device-poc',
        component: () => import('./DevicePocPage.vue'),
      },
    ]
  : []

export const routes: RouteRecordRaw[] = [
  {
    path: '/',
    name: 'portal',
    component: PortalPage,
  },
  {
    path: '/rooms/:roomId',
    name: 'room',
    component: RoomPage,
  },
  {
    path: '/rooms/:roomId/clear',
    name: 'clear',
    component: ClearPage,
  },
  ...developmentRoutes,
  {
    path: '/:pathMatch(.*)*',
    redirect: { name: 'portal' },
  },
]

export function createAppRouter(history: RouterHistory = createWebHistory()): Router {
  return createRouter({ history, routes })
}

export const router = createAppRouter()
