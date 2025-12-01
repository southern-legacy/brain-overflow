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
          component: () => import('@/views/mainpage/index.vue'),
        },
      ],
    },
    {
      path: '/userProfile',
      component: () => import('@/views/userProfile/layout/index.vue'),
    },
  ],
})

export default router
