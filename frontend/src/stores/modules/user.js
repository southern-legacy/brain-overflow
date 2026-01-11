import { ref } from 'vue'
import { defineStore } from 'pinia'
import { getUserProfile } from '@/api/userProfiles'
export const useUserStore = defineStore(
  'user',
  () => {
    const token = ref('')
    const userInfo = ref({})
    const userProfile = ref({})

    const setToken = (payload) => {
      token.value = payload
    }

    const setUserInfo = (payload) => {
      userInfo.value = payload
    }

    const getUserProfilePinia = async (id) => {
      const profile = await getUserProfile(id)
      setUserProfile(profile)
    }

    const setUserProfile = (profile) => {
      userProfile.value = profile
    }

    const logout = () => {
      setToken('')
      setUserInfo({})
    }
    return {
      token,
      setToken,
      userInfo,
      setUserInfo,
      logout,
      setUserProfile,
      getUserProfilePinia,
      userProfile,
    }
  },
  {
    persist: true,
  },
)
