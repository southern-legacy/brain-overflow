import request from '@/utils/request'

/**
 * starting to upload an asset
 *
 * @param {*} id userId, type: uuid
 * @returns {*}  a url, used for the upload
 */
export const startUploadAssetWithId = (id) => {
  return request({
    url: `/asset/${id}`,
    method: 'put',
    raw: true,
  })
}

/**
 * starting to upload an asset, but with full url request
 *
 * @param {*} url full url
 * @returns {*}
 */
export const startUploadAssetWithFullURL = (url) => {
  return request({
    url: url,
    method: 'put',
    raw: true,
  })
}

export const uploadAsset = (file, token, url) => {
  return request({
    url: url,
    method: 'put',
    data: file,
    headers: {
      'Content-Type': file.type,
      Authorization: `Bearer ${token}`,
    },
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
