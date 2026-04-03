import { createRouter, createWebHashHistory, RouteRecordRaw } from 'vue-router'

const routes: Array<RouteRecordRaw> = [
  {
    path: '/',
    redirect: '/dashboard'
  },
  {
    path: '/dashboard',
    name: 'Dashboard',
    component: () => import('../views/dashboard/index.vue')
  },
  {
    path: '/novels',
    name: 'Novels',
    component: () => import('../views/novels/index.vue')
  },
  {
    path: '/novels/create',
    name: 'NovelCreate',
    component: () => import('../views/novels/create.vue')
  },
  {
    path: '/novels/:id',
    name: 'NovelDetail',
    component: () => import('../views/novels/detail.vue')
  },
  {
    path: '/novels/:id/chapters',
    name: 'ChapterList',
    component: () => import('../views/novels/chapters.vue')
  },
  {
    path: '/novels/:novelId/chapters/create',
    name: 'ChapterCreate',
    component: () => import('../views/novels/chapter-create.vue')
  },
  {
    path: '/novels/:novelId/chapters/:id/edit',
    name: 'ChapterEdit',
    component: () => import('../views/novels/chapter-edit.vue')
  },
  {
    path: '/audio',
    name: 'Audio',
    component: () => import('../views/audio/index.vue')
  },
  {
    path: '/mixer',
    name: 'Mixer',
    component: () => import('../views/mixer/index.vue')
  },
  {
    path: '/ambient',
    name: 'Ambient',
    component: () => import('../views/ambient/index.vue')
  },
  {
    path: '/characters',
    name: 'Characters',
    component: () => import('../views/characters/index.vue')
  },
  {
    path: '/video',
    name: 'Video',
    component: () => import('../views/video/index.vue')
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('../views/settings/index.vue')
  }
]

const router = createRouter({
  history: createWebHashHistory(),
  routes
})

export default router
