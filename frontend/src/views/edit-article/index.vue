<!--
  * Component: EditArticle
  * Description:  the page used to edit articles or questions
-->

<script setup>
import EditHeader from './components/EditHeader.vue'
import { ref } from 'vue'

const text = ref('')

const handleUploadImage = async (event, insertImage, files) => {
  console.log('myImageUpload')
  const file = files[0]
  if (!file) return

  // todo：upload image
  const url = await uploadImage(file)

  insertImage({
    url,
    desc: file.name,
  })
}

const uploadImage = async (file) => {
  // mock
  return URL.createObjectURL(file)
}
</script>

<template>
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
