import { createRouter, createWebHashHistory } from 'vue-router'

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes: [
    {
      path:'/login',
      component: () => import('@/views/login/index.vue')
    },
    {
      path:'/',
      component: () => import('@/views/layout/LayoutContainer.vue')
    }
  ],
})

export default router
