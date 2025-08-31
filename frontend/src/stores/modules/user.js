import { ref } from 'vue'
import { defineStore } from 'pinia'

export const useUserStore = defineStore('user', () => {
  
  const token = ref('')

  const setToken = (payload) => {
    token.value = payload
  }
  return { token, setToken }
 
},{
  persist: true
})
