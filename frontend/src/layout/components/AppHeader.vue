<script setup>
import { ref } from 'vue'
const keyword = ref('')
</script>

<template>
  <header class="header">
    <div class="logo">Brain Overflow</div>

    <el-input
      v-model="keyword"
      placeholder="Search questions..."
      prefix-icon="Search"
      class="search"
    />

    <div class="actions">
      <el-button type="primary">Ask Question</el-button>
      <el-avatar src="xxx" />
      <el-button circle size="large" type="primary">登录</el-button>
      <el-button circle size="large">注册</el-button>
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
}

.search {
  flex: 1;
}

.actions {
  display: flex;
  align-items: center;
  gap: 12px;
}
</style>
