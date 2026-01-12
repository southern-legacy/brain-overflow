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

export const getUserProfileAsset = (url, token) => {
  return request({
    url: url,
    method: 'get',
    headers: {
      Authorization: `Bearer ${token}`,
    },
    responseType: 'blob',
  })
}
