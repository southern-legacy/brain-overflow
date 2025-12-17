<script setup>
import UserAvatar from '@/components/UserAvatar.vue'
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Search } from '@element-plus/icons-vue'
const keyword = ref('')
const isFocusOnSearch = ref(false)
const router = useRouter()

function jumpToEdit() {
  router.push('/edit-article')
}

function jumpToLogin() {
  router.push('/login')
}
</script>

<template>
  <header class="header">
    <div class="logo">Brain Overflow</div>

    <el-input
      v-model="keyword"
      placeholder="Search questions..."
      :prefix-icon="Search"
      class="search"
      @focus="isFocusOnSearch = true"
      @blur="isFocusOnSearch = false"
    />

    <div class="actions">
      <el-button type="primary" class="action-btn" :class="{ hide: isFocusOnSearch }"
        >Ask Question</el-button
      >
      <!-- todo: 目前创作者中心用于跳转至编辑文章页面，后续功能暂未实现 -->
      <!-- todo: the Button is now used to jump to edit-article page -->
      <el-button
        type="primary"
        class="action-btn"
        :class="{ hide: isFocusOnSearch }"
        @click="jumpToEdit"
        >创作者中心</el-button
      >
      <UserAvatar></UserAvatar>
      <el-button circle size="large" type="primary" @click="jumpToLogin">登录</el-button>
      <el-button circle size="large" @click="jumpToLogin">注册</el-button>
    </div>
  </header>
</template>

<style scoped>
.header {
  height: 75px;
  padding: 0 20px;
  border-bottom: 1px solid #ddd;
  display: flex;
  align-items: center;
  gap: 16px;
}

.logo {
  font-size: 24px;
  font-weight: bold;
  cursor: pointer;
}

.search {
  flex: 1;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;

  .action-btn {
    width: 120px; /* 初始宽度 */
    transition:
      width 0.3s ease,
      opacity 0.3s ease;
  }

  .action-btn.hide {
    width: 0; /* 宽度过渡到 0 */
    opacity: 0;
    padding: 0; /* 防止文字占位 */
    overflow: hidden; /* 隐藏溢出文字 */
  }
}
</style>
