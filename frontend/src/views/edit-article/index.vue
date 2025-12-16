<script setup>
import EditHeader from './components/EditHeader.vue'
import { ref } from 'vue'

const text = ref('')

const handleUploadImage = async (event, insertImage, files) => {
  console.log('myImageUpload')
  const file = files[0]
  if (!file) return

  //todo：换成你自己的上传逻辑
  const url = await uploadImage(file)

  insertImage({
    url,
    desc: file.name,
  })
}

const uploadImage = async (file) => {
  // 临时 mock
  return URL.createObjectURL(file)
}
</script>

<template>
  <!-- todo： edit-header -->
  <!-- todo： 一个头部， -->
  <div class="app-container">
    <EditHeader></EditHeader>
    <v-md-editor
      v-model="text"
      height="1000px"
      @upload-image="handleUploadImage"
      :disabled-menus="[]"
    ></v-md-editor>
  </div>
</template>

<style scoped lang="scss"></style>
