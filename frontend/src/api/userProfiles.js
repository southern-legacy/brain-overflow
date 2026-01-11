import request from '@/utils/request'

export const getUserProfile = (id) => {
  return request({
    url: `/user/${id}`,
    method: 'get',
  })
}

export const startUploadUserProfileAssets = (type) => {
  return request({
    url: `/user/bio/${type}`,
    method: 'put',
    raw: true,
  })
}
