<script setup>
import UserAvatar from '@/components/UserAvatar.vue'
import { ref } from 'vue'

let articleTitle = ref('')
</script>

<template>
  <el-row class="header-container">
    <el-col class="input-container" :span="16">
      <el-input
        class="title-input"
        v-model="articleTitle"
        placeholder="请输入文章标题..."
      ></el-input>
    </el-col>
    <el-col :span="6" :offset="2" class="function">
      <el-button type="primary" class="submit">发布文章</el-button>
      <UserAvatar></UserAvatar>
    </el-col>
  </el-row>
</template>

<style scoped lang="scss">
.header-container {
  height: 60px;
  padding: 10px;
  display: flex;
  align-items: center;

  .input-container {
    flex: 1;
    margin-left: 20px;
    :deep(.title-input .el-input__wrapper) {
      box-shadow: none !important;
      border: none !important;
      padding-left: 0;
    }

    :deep(.title-input .el-input__wrapper.is-focus) {
      box-shadow: none !important;
    }

    :deep(.title-input .el-input__inner) {
      font-size: 22px;
      font-weight: 600;
      padding: 0;
      color: #1d2129;
    }

    :deep(.title-input .el-input__inner::placeholder) {
      color: #86909c;
    }
  }

  .function {
    display: flex;
    align-items: center;
    .submit {
      margin-right: 10px;
    }
  }
}
</style>
