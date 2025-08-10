import request from '@/utils/request'

export const regisAccountWithEmail = (name, email, passwd) => {
  return request({
    url: '/usr/signup',
    method: 'post',
    data: {
      name,
      email,
      passwd
    }
  })
}

export const regisAccountWithPhone = (name, phone, passwd) => {
  return request({
    url: '/usr/signup',
    method: 'post',
    data: {
      name,
      phone,
      passwd
    }
  })
}

export const loginWithPhone = (phone, passwd) => {
  return request({
    url: '/usr/login',
    method: 'post',
    data: {
      phone,
      passwd
    }
  })
}

export const loginWithId = (id, passwd) => {
  return request({
    url: `/usr/login/${id}`,
    method: 'post',
    headers: {
      'Content-Type': 'text/plain'
    },
    data: passwd
  })
}

export const loginWithEmail = (email, passwd) => {
  return request({
    url: '/usr/login',
    method: 'post',
    data: {
      email,
      passwd
    }
  })
}

export const deleteAccount = (passwd) => {
  return request({
    url: '/usr/delete_account',
    method: 'post',
    headers: {
      'Authorization': `Bearer ${localStorage.getItem('token')}`, // 从本地获取 token
      'Content-Type': 'text/plain'
    },
    data: passwd // 直接传纯文本密码
  })
}