import axios from 'axios'
import { Message } from 'element-ui'
// 创建统一实例
const instance = axios.create({
  baseURL: 'http://localhost:10086',
  timeout: 5000
})

// 请求拦截器
instance.interceptors.request.use(config => {
  // 自动附加 Bearer token（如果存在）
  const token = localStorage.getItem('token')
  if (token) {
    config.headers.Authorization = `Bearer ${token}`
  }

  // 如果是 text/plain 请求，确保 axios 不会把它转换成 JSON
  if (config.headers['Content-Type'] === 'text/plain' && typeof config.data !== 'string') {
    config.data = String(config.data ?? '')
  }

  return config
}, error => Promise.reject(error))

// 响应拦截器
instance.interceptors.response.use(response => {
  // 直接返回后端数据，剥离 axios 包装
  return response.data
}, error => {
  // 统一错误处理
  const errData = error.response?.data || { message: error.message }
  console.error('请求出错:', errData)
    Message({
      message: errData.message || '请求出错',
      type: 'error',
      duration: 3000
    })
  return Promise.reject(errData)
})

export default instance