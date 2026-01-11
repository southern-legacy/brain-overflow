import request from '@/utils/request'

/**
 * starting to upload an asset
 *
 * @param {*} id userId, type: uuid
 * @returns {*}  a url, used for the upload
 */
export const startUploadAsset = (id) => {
  return request({
    url: `/asset/${id}`,
    method: 'put',
  })
}

/**
 * inform the server that the upload is over
 *
 * @param {*} assetId the asset id you get when end the upload
 * @returns {*}
 */
export const abortUploadAsset = (assetId) => {
  return request({
    url: `/asset/${assetId}/end`,
    method: 'put',
  })
}

/**
 * get an asset
 *
 * @param {*} assetId  the asset id
 * @returns {*}
 */
export const getAsset = (assetId) => {
  return request({
    url: `/asset/${assetId}`,
    method: 'get',
  })
}
