import request from '@/utils/request'

export const getUserProfile = (id) => {
  return request({
    url: `/user/${id}`,
    method: 'get',
  })
}
