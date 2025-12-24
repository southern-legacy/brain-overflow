export default [
  {
    path: '/user',
    component: () => import('@/views/user-profile/layout/index.vue'),
    children: [
      {
        path: 'profile',
        component: () => import('@/views/user-profile/index.vue'),
        meta: { title: '用户主页' },
      },
      {
        path: 'settings',
        component: () => import('@/views/user-profile/UserSettings.vue'),
        meta: { title: '用户设置' },
      },
    ],
  },
]
