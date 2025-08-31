import axios from 'axios'
import { useUserStore } from '@/stores'
// 创建统一实例

const instance = axios.create({
  baseURL: 'http://localhost:10086',
  timeout: 10000
})

// 请求拦截器
instance.interceptors.request.use(config => {
  // 自动附加 Bearer token（如果存在）
  const userStore = useUserStore()
  const token = userStore.token
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
instance.interceptors.response.use(
  response => response.data, //如果成功，直接剥离一层后再返回
  error => {
    let errMessage = "Unknown Error"; // 错误信息
    let errCode = null; // 服务器返回的code
    let status = null; //

    // 如果存在响应体 则在其中尝试获取code
    if (error.response) {
      const serverData = error.response.data;
      status = error.response.status;
      errCode = serverData?.code || null;

      // 优先用后端返回的 message 或 error
      errMessage =
        typeof serverData === "string" ? serverData :
        serverData?.message ||
        serverData?.error ||
        error.response.statusText;
    }

    // 网络超时
    if (error.code === "ECONNABORTED") {
      errMessage = "Request Timeout";
    }

    // 按 HTTP 状态码分类处理
    switch (status) {
      case 400:
        console.warn("400 Bad Request", {
          status,
          code: errCode,
          rawMessage: errMessage,
          response: error.response
        });

        // 给用户的简短提示
        ElMessage.error("请求参数错误");
        break;

      case 401:
        if (errCode === "token_expired" || errCode === "token_invalid") {
          ElMessage.error("登录状态已过期，请重新登录");
          // window.location.href = "/login";
        } else {
          ElMessage.error("登录验证失败，请检查账户和密码并重新登陆");
        }
        break;

      case 405:
        console.warn("405 Method Not Allowed", {
          status,
          code: errCode,
          rawMessage: errMessage,
          response: error.response
        });
        ElMessage.error("请求方法错误，请联系开发者");
        break;

      case 422:
        // 记录详细错误
        console.warn("422 Unprocessable Entity", {
          status,
          code: errCode,
          rawMessage: errMessage,
          response: error.response
        });
        break;

      case 500:
        ElMessage.error("服务器错误，请稍后再试");
        break;

      default:
        ElMessage.error(errMessage || "请求失败");
    }

    return Promise.reject({
      status,
      code: errCode,
      message: errMessage
    });
  }
);

export default instance