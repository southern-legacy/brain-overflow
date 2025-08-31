import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

export default pinia

// import { useUserStore } from './modules/user'
// export { useUserStore }

// 简单语法
export * from '@/stores/modules/user'
