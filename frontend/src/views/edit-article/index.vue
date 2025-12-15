<script setup>
import { ref } from 'vue'
let text = ref('')
</script>

<template>
  <!-- todo： edit-header -->
  <!-- todo： 一个头部， -->
  <div class="app-container">
    <v-md-editor v-model="text" height="1000px"></v-md-editor>
  </div>
</template>

<style scoped lang="scss">
.app-container {
}
</style>
