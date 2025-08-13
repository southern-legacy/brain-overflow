import Vue from 'vue'
import App from './App.vue'
import router from './router'
import store from './store'
import '@/styles/common.css'
import { Divider, Button} from 'element-ui'

Vue.config.productionTip = false
Vue.component(Divider.name,Divider)
Vue.component(Button.name, Button);

new Vue({
  router,
  store,
  render: h => h(App)
}).$mount('#app')
