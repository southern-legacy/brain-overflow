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
export const userRegisService = (name, email, phone, password) => {
  const data = {
    name,
    password,
  }

  if (email) {
    data.email = email
  }
  if (phone) {
    data.phone = phone
  }

  return request({
    url: '/user',
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
export const userLoginService = (id, email, phone, password) => {
  const data = {
    password,
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
    url: '/user/login',
    method: 'post',
    data,
  })
}

/**
 * change user verification
 *
 * @param {*} password password before
 * @param {*} newPassword new password, change one at a time
 * @param {*} newEmail new email, change one at a time
 * @param {*} newPhone new phone, change one at a time
 * @returns {*}
 */
export const changeUserVerification = (password, newPassword, newEmail, newPhone) => {
  const data = {
    password,
  }
  if (newPassword) data.newPassword = newPassword
  if (newEmail) data.newEmail = newEmail
  if (newPhone) data.newPhone = newPhone
  return request({
    url: '/user',
    method: 'put',
    data,
  })
}

export const deleteUserAccount = (password) => {
  return request({
    url: '/user',
    method: 'delete',
    data: password,
  })
}
