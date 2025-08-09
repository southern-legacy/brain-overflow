import Vue from 'vue'
import App from './App.vue'
import router from './router'
import store from './store'
import '@/styles/common.css'
import { Divider} from 'element-ui'

Vue.config.productionTip = false
Vue.component(Divider.name,Divider)


new Vue({
  router,
  store,
  render: h => h(App)
}).$mount('#app')
