import { createRouter, createWebHashHistory } from 'vue-router'

// 按模块导入
import mainRoutes from './modules/main'
import userRoutes from './modules/user'
import editArticleRoutes from './modules/editArticle'
import loginRoutes from './modules/login'
import notFoundRoutes from './modules/notFound'

const routes = [
  ...loginRoutes,
  ...mainRoutes,
  ...userRoutes,
  ...editArticleRoutes,
  ...notFoundRoutes,
]

const router = createRouter({
  history: createWebHashHistory(import.meta.env.BASE_URL),
  routes,
})

// Unified management of page titles
router.afterEach((to) => {
  document.title = to.meta.title || 'BrainOverflow'
})

export default router
