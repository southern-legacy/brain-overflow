import axios from 'axios'

// 创建统一实例
const instance = axios.create({
  baseURL: 'http://localhost:10086',
  timeout: 5000
})

// 请求拦截器
instance.interceptors.request.use(config => {
  // 可在此添加token等
  return config
}, error => Promise.reject(error))

// 响应拦截器
instance.interceptors.response.use(response => {
  // 直接返回后端数据，剥离axios包装
  return response.data
}, error => {
  // 统一错误处理
  return Promise.reject(error.response?.data || error.message)
})

export default instance