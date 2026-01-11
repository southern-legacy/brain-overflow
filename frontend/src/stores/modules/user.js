import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useUserStore = defineStore(
  'user',
  () => {
    const token = ref('')
    const userInfo = ref({})

    const setToken = (payload) => {
      token.value = payload
    }
    const setUserInfo = (payload) => {
      userInfo.value = payload
    }
    const logout = () => {
      setToken('')
      setUserInfo({})
    }
    return { token, setToken, userInfo, setUserInfo, logout }
  },
  {
    persist: true,
  },
)
