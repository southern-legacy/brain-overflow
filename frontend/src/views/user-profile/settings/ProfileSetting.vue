<script setup>
import { onActivated, ref, watch } from 'vue'
import { useUserStore } from '@/stores'
import { Plus } from '@element-plus/icons-vue'
import { startUploadUserProfileAssets, getUserProfileAsset } from '@/api/userProfiles'
import {
  abortUploadAsset,
  startUploadAssetWithFullURL,
  uploadAsset,
  getAsset,
} from '@/api/crab-vault'

defineOptions({ name: 'ProfileSetting' })
const userStore = useUserStore()

onActivated(async () => {
  // when activated:  get user profile(update)
  // and then get the asset we needed to show
  // bug fixed: must await the user profile data, otherwise it may use the old data
  await userStore.getUserNewProfilePinia(userStore.userInfo.id)
  await getProfileSettingAssets()
})

const userProfileForm = ref({
  name: userStore.userInfo.name ?? '',
  userBiography: userStore.userProfile.biography ?? '',
  userBanner: userStore.userProfile.banner ?? '',
  contactMe: userStore.userProfile.contactMe ?? '',
  userAvatar: userStore.userProfile.avatar ?? '',
})

watch(
  () => userStore.userProfile,
  (newProfile) => {
    userProfileForm.value = {
      name: userStore.userInfo.name ?? '',
      userBiography: newProfile.biography ?? '',
      userBanner: newProfile.banner ?? '',
      contactMe: newProfile.contactMe ?? [],
      userAvatar: newProfile.avatar ?? '',
    }
  },
  { immediate: true, deep: true },
)

// the avatar url to show
const avatarUrl = ref('')

// banner upload file
const bannerFileList = ref([])

// new contact me input
const newContactMe = ref('')

// avatar upload file
const avatarFileList = ref([])

// the markdown string
const userBiographyMarkdown = ref('')

/**
 * * the submit of aseets(banner) with crab-vault
 * * workflow:
 * - 1. request banner upload (brain-overflow) to get a Location(needed to add)
 * - 2. using the location to start uploading asset(brain-overflow), it will return a token and the crab-vault request url(full)
 * - 3. with the token and url, upload the asset you want(crab-vault)
 * - 4. send end info to the brainoverflow(using assetId which you get in the step 2 url)
 * - 5. remember to save the asset id
 *
 */
async function submitBannerUpload() {
  if (!bannerFileList.value || bannerFileList.value.length === 0) {
    return ElMessage({
      type: 'warning',
      message: '请选择文件',
    })
  }

  try {
    // starting to upload asset, get a location
    const res = await startUploadUserProfileAssets('banner')

    const location = res.headers.location
    // create the URL
    const url = new URL(location, import.meta.env.VITE_API_BASE_URL).toString()

    // start the upload, get a token and a new URL
    const crabVault = await startUploadAssetWithFullURL(url)
    const crabVaultToken = crabVault.data.token

    const crabVaultUrl = crabVault.headers.location
    // save the asset id
    const assetId = new URL(crabVaultUrl).pathname.split('/').pop()

    // using the token and URL to upload to crab vault
    await uploadAsset(bannerFileList.value[0].raw, crabVaultToken, crabVaultUrl)

    // inform brain-overflow that the upload is over
    await abortUploadAsset(assetId)

    // save the asset id
    userProfileForm.value.userBanner = assetId
    ElMessage({
      type: 'success',
      message: '成功上传',
    })
  } catch (error) {
    console.warn('[submitBannerUpload] upload failed:', error)
    ElMessage({
      type: 'error',
      message: '上传banner失败',
    })
  }
}

/**
 * * add contact way
 *
 * * it only supports ten contact ways
 * * and its length is [1, 100]
 *
 *
 */
const addContact = () => {
  if (userProfileForm.value.contactMe.length > 10) {
    ElMessage({
      type: 'warning',
      message: '仅支持十个联系方式，请先清除多余联系方式',
    })
    return
  }

  if (newContactMe.value.length > 100 || newContactMe.value.length <= 0) {
    ElMessage({
      type: 'warning',
      message: '联系方式必须是 1 - 100个字符',
    })
    return
  }

  userProfileForm.value.contactMe.push(newContactMe.value.trim())
  newContactMe.value = ''
}

// remove contact
const removeContact = (index) => {
  userProfileForm.value.contactMe.splice(index, 1)
}

/**
 * * the submit of aseets(avatar) with crab-vault
 * * workflow: same as "submitBannerUpload" function
 *
 */
const submitAvatarUpload = async () => {
  if (!avatarFileList.value || avatarFileList.value.length === 0) {
    return ElMessage({
      type: 'warning',
      message: '请选择文件',
    })
  }

  try {
    // starting to upload asset, get a location
    const res = await startUploadUserProfileAssets('avatar')

    const location = res.headers.location
    // create the URL
    const url = new URL(location, import.meta.env.VITE_API_BASE_URL).toString()

    // start the upload, get a token and a new URL
    const crabVault = await startUploadAssetWithFullURL(url)
    const crabVaultToken = crabVault.data.token

    const crabVaultUrl = crabVault.headers.location
    // save the asset id
    const assetId = new URL(crabVaultUrl).pathname.split('/').pop()

    // using the token and URL to upload to crab vault
    await uploadAsset(avatarFileList.value[0].raw, crabVaultToken, crabVaultUrl)

    // inform brain-overflow that the upload is over
    await abortUploadAsset(assetId)

    // save the asset id
    userProfileForm.value.userAvatar = assetId
    ElMessage({
      type: 'success',
      message: '成功上传',
    })
  } catch (error) {
    console.warn('[submitAvatarUpload] upload failed:', error)
    ElMessage({
      type: 'error',
      message: '上传头像失败',
    })
  }
}

/**
 * * get the avatar with asset id
 * * workflow
 * 1. get token and URL(from Brain-Overflow) with asset id
 * 2. get blob value(from crab vault) with token and URL
 */
const getProfileAvatar = async () => {
  const avatar = await getAsset(userStore.userProfile.avatar)
  const res = await getUserProfileAsset(avatar.url, avatar.token)
  if (avatarUrl.value) URL.revokeObjectURL(avatarUrl.value)
  avatarUrl.value = URL.createObjectURL(res)
}

/**
 * * get the biography with asset id
 * * workflow: same as "getProfileAvatar" function
 */
const getProfileBiography = async () => {
  const biography = await getAsset(userStore.userProfile.biography)
  const res = await getUserProfileAsset(biography.url, biography.token)
  const text = await res.text()
  userBiographyMarkdown.value = text
}

/**
 * * generate the two steps(get biography and avatar) together
 * * easier to send request when activated
 * * [notice]: here we do not request the banner, cause it's too big to show
 */
const getProfileSettingAssets = async () => {
  const promises = []

  if (userStore.userProfile.biography) {
    promises.push(getProfileBiography())
  }

  if (userStore.userProfile.avatar) {
    promises.push(getProfileAvatar())
  }

  await Promise.all(promises)
}

const submitBiographyUpload = async () => {
  if (!userBiographyMarkdown.value || userBiographyMarkdown.value.length === 0) {
    return ElMessage({
      type: 'warning',
      message: '请选择输入用户简介',
    })
  }

  try {
    // starting to upload asset, get a location
    const res = await startUploadUserProfileAssets('biography')

    const location = res.headers.location
    // create the URL
    const url = new URL(location, import.meta.env.VITE_API_BASE_URL).toString()

    // start the upload, get a token and a new URL
    const crabVault = await startUploadAssetWithFullURL(url)
    const crabVaultToken = crabVault.data.token

    const crabVaultUrl = crabVault.headers.location
    // save the asset id
    const assetId = new URL(crabVaultUrl).pathname.split('/').pop()

    // using the token and URL to upload to crab vault
    await uploadAsset(userBiographyMarkdown.value, crabVaultToken, crabVaultUrl)

    // inform brain-overflow that the upload is over
    await abortUploadAsset(assetId)

    // save the asset id
    userProfileForm.value.userBiography = assetId
    ElMessage({
      type: 'success',
      message: '成功上传',
    })
  } catch (error) {
    console.warn('[submitBiographyUpload] upload failed:', error)
    ElMessage({
      type: 'error',
      message: '上传头像失败',
    })
  }
}

/**
 * ! now it is only used to upload biography, cause the server doesn't support to change usernames
 * todo: add change usernames functions and other more basic info
 */
const submitProfileSettings = async () => {
  submitBiographyUpload()
}
</script>

<template>
  <div class="profile-setting-container">
    <div class="container-left">
      <el-form label-width="130px" size="large" :model="userProfileForm">
        <el-form-item label="用户名">
          <el-input v-model="userProfileForm.name"></el-input>
        </el-form-item>

        <el-form-item label="个人简介">
          <el-input v-model="userBiographyMarkdown" type="textarea" :rows="5"></el-input>
        </el-form-item>

        <el-form-item class="user-biography-preview" label="效果预览">
          <v-md-editor mode="preview" :model-value="userBiographyMarkdown"></v-md-editor>
        </el-form-item>

        <el-form-item class="user-banner-upload" label="个人主页背景">
          <el-upload
            class="banner-upload"
            :limit="1"
            :auto-upload="false"
            v-model:file-list="bannerFileList"
          >
            <template #trigger>
              <el-button type="primary">选择文件</el-button>
            </template>

            <div>
              <el-button class="ml-3" type="success" @click="submitBannerUpload">
                上传文件
              </el-button>
            </div>

            <template #tip>
              <div class="el-upload__tip text-red">仅支持上传一个文件，多余的会被忽略</div>
            </template>
          </el-upload>
        </el-form-item>

        <el-form-item label="联系方式">
          <div class="contact-input">
            <el-tag
              v-for="(item, index) in userProfileForm.contactMe"
              :key="item + index"
              closable
              @close="removeContact(index)"
              class="contact-tag"
            >
              {{ item }}
            </el-tag>

            <el-input
              v-model="newContactMe"
              placeholder="回车添加联系方式"
              size="small"
              @keyup.enter="addContact"
              style="width: 220px"
            />
          </div>
        </el-form-item>

        <el-form-item>
          <el-button type="primary" @click="submitProfileSettings">点击修改</el-button>
        </el-form-item>
      </el-form>
    </div>
    <div class="container-right">
      <el-upload
        class="user-avatar-upload"
        :limit="1"
        :auto-upload="false"
        v-model:file-list="avatarFileList"
      >
        <img v-if="avatarUrl" :src="avatarUrl" class="avatar" />
        <el-icon v-else class="avatar-uploader-icon"><Plus /></el-icon>

        <template #tip>
          <div class="el-upload__tip">点击上传头像</div>
        </template>
      </el-upload>

      <el-button type="primary" @click="submitAvatarUpload">开始上传</el-button>
    </div>
  </div>
</template>

<style scoped lang="scss">
.profile-setting-container {
  display: flex;
  justify-content: space-around;
  .container-left {
    flex: 2;
    ::v-deep(.user-biography-preview .el-form-item__content) {
      border: 1px solid #666;
      border-radius: 5px;
    }
    .banner-upload {
      button {
        width: 100px;
      }
    }
  }
  .container-right {
    flex: 1;
    display: flex;
    align-items: center;
    flex-direction: column;

    .user-avatar-upload {
      width: 250px;
      display: flex;
      flex-direction: column;
      align-items: center;
    }

    .user-avatar-upload .avatar {
      width: 178px;
      height: 178px;
      display: block;
    }

    ::v-deep(.user-avatar-upload .el-upload) {
      border: 3px dashed var(--el-border-color);
      border-radius: 89px;
      cursor: pointer;
      position: relative;
      overflow: hidden;
      transition: var(--el-transition-duration-fast);
    }

    ::v-deep(.user-avatar-upload .el-upload:hover) {
      border-color: var(--el-color-primary);
    }

    ::v-deep(.el-icon.avatar-uploader-icon) {
      font-size: 28px;
      color: #8c939d;
      width: 178px;
      height: 178px;
      text-align: center;
    }
  }
}
</style>
