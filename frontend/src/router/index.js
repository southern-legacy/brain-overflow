import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path: '/login',
      component: () => import('@/views/login/index.vue'),
    },

    {
      path: '/',
      component: () => import('@/layout/index.vue'),
      children: [
        {
          path: '',
          component: () => import('@/views/main-page/index.vue'),
        },
      ],
    },

    {
      path: '/user-profile',
      component: () => import('@/views/user-profile/layout/index.vue'),
      children: [
        {
          path: '',
          component: () => import('@/views/user-profile/index.vue'),
        },
      ],
    },

    {
      path: '/user-settings',
      component: () => import('@/views/user-profile/layout/index.vue'),
      children: [
        {
          path: '',
          component: () => import('@/views/user-profile/UserSettings.vue'),
        },
      ],
    },

    {
      path: '/edit-article',
      component: () => import('@/views/edit-article/index.vue'),
    },

    {
      path: '/:pathMatch(.*)*', // 匹配所有未匹配的路径
      name: 'NotFound',
      component: () => import('@/views/404/index.vue'),
    },
  ],
})

export default router
