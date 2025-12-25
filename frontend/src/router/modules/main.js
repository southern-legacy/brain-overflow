export default [
  {
    path: '/',
    component: () => import('@/layout/index.vue'),
    children: [
      {
        path: '',
        component: () => import('@/views/main-page/index.vue'),
        meta: { title: 'BrainOverflow' },
      },
    ],
  },
]
