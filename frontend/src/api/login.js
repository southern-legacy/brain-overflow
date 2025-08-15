import request from '@/utils/request'

export const regisAccount = (obj) => {
  return request({
    url: '/usr',
    method: 'post',
    data: {
      ...obj
    }
 })
}

export const loginAccount = (obj) => {
  return request({
    url: '/usr/login',
    method: 'post',
    data: {
      ...obj
    }
  })
}

export const deleteAccount = (passwd) => {
  return request({
    url: '/usr',
    method: 'delete',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('token')}`, // 从本地获取 token
      'Content-Type': 'text/plain'
    },
    data: passwd // 直接传纯文本密码
  })
}