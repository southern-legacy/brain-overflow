import axios from 'axios'
import { useUserStore } from '@/stores'
import router from '@/router'

const instance = axios.create({
  baseURL: import.meta.env.VITE_API_BASE_URL,
  timeout: 10000,
})

// 请求拦截器
instance.interceptors.request.use(
  (config) => {
    const userStore = useUserStore()
    const token = userStore.token

    // if we don't config Authorization, add with user Token
    if (token && !config.headers.Authorization) {
      config.headers.Authorization = `Bearer ${token}`
    }

    // if content type is text/plain, make sure that axios will not turn it into JSON
    if (config.headers['Content-Type'] === 'text/plain' && typeof config.data !== 'string') {
      config.data = String(config.data ?? '')
    }

    return config
  },
  (error) => Promise.reject(error),
)

instance.interceptors.response.use(
  (response) => {
    // if raw: true，return full response. Unless return response.data
    if (response.config.raw) {
      return response
    }
    return response.data
  },
  (error) => {
    // 1. Internet Error
    if (error.code === 'ECONNABORTED' || error.message.includes('timeout')) {
      ElMessage.error('请求超时，请检查网络')
      return Promise.reject(error)
    }

    // if there is no response, meaning that the server / internet is dead
    if (!error.response) {
      ElMessage.error('网络连接异常，无法连接到服务器')
      return Promise.reject(error)
    }

    const { status, data } = error.response

    // handle status
    switch (status) {
      case 401:
        // get reason from response data
        // backend returns: { reason: 'tokenExpired' }
        if (data?.reason === 'tokenExpired') {
          // if Token expired -> force logout
          ElMessage.error('登录状态已过期，请重新登录')

          const userStore = useUserStore()
          userStore.logout() // clear pinia
          router.push(`/login?redirect=${router.currentRoute.value.fullPath}`)
        } else {
          //  wrong password / cannot validate
          const msg = data?.message || '身份验证失败，请检查'
          ElMessage.error(msg)
        }
        break

      case 403:
        ElMessage.warning('您没有权限执行此操作')
        break

      case 500:
        ElMessage.error('服务器内部错误，请稍后再试')
        break

      default:
        // other error
        ElMessage.error(data?.message || `请求失败 (${status})`)
    }

    return Promise.reject({
      status,
      message: data?.message || error.message,
      data: data,
      reason: data?.reason,
    })
  },
)

export default instance
