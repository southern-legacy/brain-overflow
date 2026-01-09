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
        children: [
          {
            path: 'profile',
            name: 'SettingProfile',
            component: () => import('@/views/user-profile/settings/ProfileSetting.vue'),
            meta: { title: '资料设置', subTitle: '更改您的个人资料信息' },
          },
          {
            path: 'account',
            name: 'SettingAccount',
            component: () => import('@/views/user-profile/settings/AccountSetting.vue'),
            meta: { title: '账号设置', subTitle: '更改账号相关设置，进行密码修改，账户删除等' },
          },
          {
            path: 'general',
            name: 'SettingGeneral',
            component: () => import('@/views/user-profile/settings/GeneralSetting.vue'),
            meta: { title: '通用设置', subTitle: '更改网站的通用设置' },
          },
          {
            path: 'notification',
            name: 'SettingNotification',
            component: () => import('@/views/user-profile/settings/NotificationSetting.vue'),
            meta: { title: '消息设置', subTitle: '更改网站的消息设置' },
          },
        ],
      },
    ],
  },
]
