export default [
  {
    path: '/login',
    component: () => import('@/views/login/index.vue'),
    meta: { title: '登录' },
  },
]
