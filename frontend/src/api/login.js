import request from '@/utils/request'

export const regisAccountWithEmail = function (name,email,passwd) {
    return request.post('/usr/signup',{
    name,
	  email,
	  passwd
    })
}

export const regisAccountWithPhone = function (name,phone,passwd){
  return request.post('/usr/signup',{
    name,
    phone,
    passwd
  })
}

export const loginWithPhone = function (phone,passwd) {
  return request.post('usr/login',{
    phone,
    passwd
  })
}

export const loginWithId = function (id,passwd) {
  return request.post(`usr/login/${id}`,{
    passwd
  })
}

export const loginWithEmail = function (email,passwd) {
  return request.post('usr/login',{
    email,
    passwd
  })
}