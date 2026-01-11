<script setup>
import { onActivated, ref } from 'vue'
import { useUserStore } from '@/stores'
import { Plus } from '@element-plus/icons-vue'
import { startUploadUserProfileAssets } from '@/api/userProfiles'

defineOptions({ name: 'ProfileSetting' })
const userStore = useUserStore()

/**
 * todo : remove the getProfile function to pinia
 */
onActivated(() => {
  userStore.getUserProfilePinia(userStore.userInfo.id)
})

const userProfileForm = ref({
  name: userStore.userInfo.name ?? '',
  userBiographyMarkdown: userStore.userProfile.biography ?? '',
  userBanner: userStore.userProfile.banner ?? '',
  contactMe: userStore.userProfile.contactMe ?? '',
  userAvatar: userStore.userProfile.avatar ?? '',
})
const avatarUrl = ref('')

async function submitBannerUpload() {
  const res = await startUploadUserProfileAssets('banner')

  const location = res.headers
  console.log(location)
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
          <el-input
            v-model="userProfileForm.userBiographyMarkdown"
            type="textarea"
            :rows="5"
          ></el-input>
        </el-form-item>

        <el-form-item class="user-biography-preview" label="效果预览">
          <v-md-editor
            mode="preview"
            :model-value="userProfileForm.userBiographyMarkdown"
          ></v-md-editor>
        </el-form-item>

        <el-form-item class="user-banner-upload" label="个人主页背景">
          <el-upload class="banner-upload" :limit="1" :auto-upload="false">
            <template #trigger>
              <el-button type="primary">选择文件</el-button>
            </template>

            <div>
              <el-button class="ml-3" type="success" @click="submitBannerUpload">
                上传文件
              </el-button>
            </div>

            <template #tip>
              <div class="el-upload__tip text-red">仅支持上传一个文件，多余的会被覆盖</div>
            </template>
          </el-upload>
        </el-form-item>

        <el-form-item class="user-contact-me" label="联系方式">
          <el-input> </el-input>
        </el-form-item>

        <el-form-item>
          <el-button type="primary">点击修改</el-button>
        </el-form-item>
      </el-form>
    </div>
    <div class="container-right">
      <el-upload class="user-avatar-upload" :limit="1" :auto-upload="false">
        <img v-if="avatarUrl" :src="avatarUrl" class="avatar" />
        <el-icon v-else class="avatar-uploader-icon"><Plus /></el-icon>

        <template #tip>
          <div class="el-upload__tip">点击上传头像</div>
        </template>
      </el-upload>
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
    justify-content: center;

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
