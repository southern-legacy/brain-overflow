import request from '@/utils/request'

export const userRegisService = (name, email, phone, passwd) => {
  const data = {
    name,
    passwd,
  }

  if (email) {
    data.email = email
  }
  if (phone) {
    data.phone = phone
  }

  return request({
    url: '/usr',
    method: 'post',
    data,
  })
}

export const userLoginService = (id, email, phone, passwd) => {
  const data = {
    passwd,
  }
  if (id) {
    data.id = id
  }
  if (phone) {
    data.phone = phone
  }
  if (email) {
    data.email = email
  }

  return request({
    url: '/usr/login',
    method: 'post',
    data,
  })
}
