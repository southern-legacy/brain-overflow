import { defineStore } from 'pinia'

import { getAsset } from '@/api/crab-vault'
import { getUserProfileAsset } from '@/api/userProfiles'

export const useUserAssetStore = defineStore('userAsset', () => {
  const assetCache = new Map()
  const loadingMap = new Map()

  /**
   * get Asset Blob from crab vault
   *
   * @async
   * @param {string} assetId asset id
   * @returns {blob}
   */
  async function getAssetBlob(assetId) {
    if (!assetId) return null

    // if cache has
    if (assetCache.has(assetId)) return assetCache.get(assetId)

    // if it is requesting
    if (loadingMap.has(assetId)) return loadingMap.get(assetId)

    const task = (async () => {
      try {
        const { url, token } = await getAsset(assetId)
        const blob = await getUserProfileAsset(url, token)

        // save in  cache
        assetCache.set(assetId, blob)

        return blob
      } finally {
        // delete from loading map
        loadingMap.delete(assetId)
      }
    })()

    loadingMap.set(assetId, task)
    return task
  }

  /**
   * invalidate an asset
   */
  function invalidate(assetId) {
    if (!assetId) return
    assetCache.delete(assetId)
    loadingMap.delete(assetId)
  }

  function clearAll() {
    assetCache.clear()
    loadingMap.clear()
  }

  return {
    getAssetBlob,
    invalidate,
    clearAll,
  }
})
