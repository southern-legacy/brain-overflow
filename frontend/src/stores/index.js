import { createPinia } from 'pinia'
import piniaPluginPersistedstate from 'pinia-plugin-persistedstate'
const pinia = createPinia()
pinia.use(piniaPluginPersistedstate)

export default pinia

// import { useUserStore } from './modules/user'
// export { useUserStore }

// simple exports
export * from '@/stores/modules/user'
