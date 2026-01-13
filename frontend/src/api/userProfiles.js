import request from '@/utils/request'

/**
 * get user profile infomation
 *
 * @param {*} id
 * @returns {*} obj
 */
export const getUserProfile = (id) => {
  return request({
    url: `/user/${id}`,
    method: 'get',
  })
}

/**
 * * start to upload user profile assets
 * * try to get the loction from response header to request futher interfaces
 * @param {*} type enum(avatar, banner, biography)
 * @returns {*}
 */
export const startUploadUserProfileAssets = (type) => {
  return request({
    url: `/user/bio/${type}`,
    method: 'put',
    raw: true,
  })
}

/**
 * * upload no-asset user profile datas(username, contactMe...)
 *
 * @param {*} data
 * @returns {*}
 */
export const uploadUserProfileOther = (data) => {
  return request({
    url: `/user/bio/other`,
    method: 'put',
    data,
  })
}

/**
 * * get the asset blob using the url and token given by crab vault
 *
 * @param {*} url the url
 * @param {*} token  the token
 * @returns {*} blob
 */
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
