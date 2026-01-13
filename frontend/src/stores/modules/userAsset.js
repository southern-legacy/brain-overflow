import { defineStore } from 'pinia'
import { getAsset } from '@/api/crab-vault'
import { getUserProfileAsset } from '@/api/userProfiles'

export const useUserAssetStore = defineStore('userAsset', () => {
  // assetId -> objectUrl
  const assetMap = new Map()

  // assetId -> in-flight promise
  const loadingMap = new Map()

  async function getAssetBlob(assetId) {
    if (!assetId) return null

    if (assetMap.has(assetId)) {
      return assetMap.get(assetId)
    }

    if (loadingMap.has(assetId)) {
      return loadingMap.get(assetId)
    }

    const task = (async () => {
      const { url, token } = await getAsset(assetId)
      const blob = await getUserProfileAsset(url, token)

      assetMap.set(assetId, blob)
      loadingMap.delete(assetId)
      return blob
    })()

    loadingMap.set(assetId, task)
    return task
  }

  function invalidate(assetId) {
    if (!assetId) return
    if (assetMap.has(assetId)) assetMap.delete(assetId)
    if (loadingMap.has(assetId)) loadingMap.delete(assetId)
  }

  function clearAll() {
    assetMap.clear()
    loadingMap.clear()
  }

  return {
    getAssetBlob,
    invalidate,
    clearAll,
  }
})
