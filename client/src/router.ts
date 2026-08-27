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
  {
    path: '/:pathMatch(.*)*',
    redirect: { name: 'portal' },
  },
]

export function createAppRouter(history: RouterHistory = createWebHistory()): Router {
  return createRouter({ history, routes })
}

export const router = createAppRouter()
