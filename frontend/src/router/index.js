import Vue from 'vue'
import VueRouter from 'vue-router'
import main from '@/views/main.vue'

Vue.use(VueRouter)

const routes = [
  {
    path:'/',
    component: main
  }
]

const router = new VueRouter({
  routes
})

export default router
