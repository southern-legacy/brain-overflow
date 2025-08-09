import Vue from 'vue'
import VueRouter from 'vue-router'
import main from '@/views/main.vue'
import login from '@/views/login/login.vue'

Vue.use(VueRouter)

const routes = [
  {
    path:'/',
    component: main
  },
  {
    path:'/login',
    component: login
  }
]

const router = new VueRouter({
  routes
})

export default router
