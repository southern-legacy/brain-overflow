import request from '@/utils/request'

/**
 * send user registration request
 *
 * @param {*} name  username
 * @param {*} email user email (mutually exclusive with user phone)
 * @param {*} phone user phone (mutually exclusive with user email)
 * @param {*} passwd user password
 * @returns {*}
 */
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

/**
 * send user login request
 *
 * @param {*} id user id (mutually exclusive with email and phone)
 * @param {*} email user email (mutually exclusive with id and phone)
 * @param {*} phone user phone (mutually exclusive with id and email)
 * @param {*} passwd user password
 * @returns {*}
 */
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
