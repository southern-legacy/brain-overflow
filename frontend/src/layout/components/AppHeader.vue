<script setup>
import { ref } from 'vue'
import { useRouter } from 'vue-router'
import { Search, Edit, Monitor } from '@element-plus/icons-vue' // 引入图标
import UserAvatar from '@/components/UserAvatar.vue'

const router = useRouter()

// 状态管理
const keyword = ref('')
const isSearchFocused = ref(false)
// TODO: 这里应该从 Pinia 获取真实的登录状态
const isLoggedIn = ref(true)

// 路由跳转
const handleCommand = (cmd) => {
  switch (cmd) {
    case 'login':
      router.push('/login')
      break
    case 'register':
      router.push('/register')
      break
    case 'edit':
      router.push('/edit-article')
      break
    case 'home':
      router.push('/')
      break
  }
}

// 模拟切换登录状态
const toggleLoginState = () => (isLoggedIn.value = !isLoggedIn.value)
</script>

<template>
  <header class="app-header">
    <div class="header-inner">
      <!-- 1. 左侧区域：Logo + 导航 -->
      <div class="left-section">
        <div class="logo" @click="handleCommand('home')">
          <span class="logo-icon">🧠</span>
          <span class="logo-text">Brain Overflow</span>
        </div>

        <!-- 预留的导航区域 -->
        <nav class="nav-links" :class="{ 'fade-out': isSearchFocused }">
          <a href="#" class="nav-item active">首页</a>
          <a href="#" class="nav-item">问答</a>
          <a href="#" class="nav-item">社区</a>
        </nav>
      </div>

      <!-- 2. 中间区域：搜索框 -->
      <!-- 当聚焦时，通过 flex-grow 占据更多空间 -->
      <div class="center-section" :class="{ 'is-focused': isSearchFocused }">
        <el-input
          v-model="keyword"
          placeholder="搜索问题、话题或人..."
          :prefix-icon="Search"
          class="search-input"
          @focus="isSearchFocused = true"
          @blur="isSearchFocused = false"
        />
      </div>

      <!-- 3. 右侧区域：操作按钮 + 用户信息 -->
      <div class="right-section">
        <!-- 创作按钮组：搜索时会收起 -->
        <div class="action-group" :class="{ collapsed: isSearchFocused }">
          <el-button text bg class="action-btn" @click="handleCommand('edit')">
            <el-icon class="el-icon--left"><Edit /></el-icon>
            提问
          </el-button>

          <el-button type="primary" class="action-btn" @click="handleCommand('edit')">
            <el-icon class="el-icon--left"><Monitor /></el-icon>
            创作者中心
          </el-button>
        </div>

        <!-- 用户状态区域 -->
        <div class="user-area">
          <template v-if="isLoggedIn">
            <!-- 已登录显示头像 -->
            <div class="avatar-wrapper">
              <UserAvatar />
            </div>
            <!-- 临时测试按钮 -->
            <el-button link size="small" type="info" @click="toggleLoginState">退出</el-button>
          </template>

          <template v-else>
            <!-- 未登录显示登录注册 -->
            <div class="auth-buttons">
              <el-button class="login-btn" text @click="handleCommand('login')">登录</el-button>
              <el-button class="register-btn" type="primary" @click="handleCommand('register')"
                >注册</el-button
              >
            </div>
          </template>
        </div>
      </div>
    </div>
  </header>
</template>

<style scoped lang="scss">
$header-height: 64px;
$primary-color: #409eff;
$text-main: #2c3e50;
$border-color: #e4e7ed;

.app-header {
  position: sticky; /* 固定在顶部 */
  top: 0;
  z-index: 1000;
  width: 100%;
  height: $header-height;
  background-color: #ffffff;
  border-top: 3px solid $primary-color;
  box-shadow: 0 1px 4px rgba(0, 0, 0, 0.05);
  display: flex;
  justify-content: center;
}

.header-inner {
  width: 100%;
  max-width: 1800px; /* 限制最大宽度，大屏更美观 */
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
}

/* --- 左侧区域 --- */
.left-section {
  display: flex;
  align-items: center;
  gap: 24px;
  flex-shrink: 0;

  .logo {
    display: flex;
    align-items: center;
    gap: 8px;
    cursor: pointer;
    transition: opacity 0.2s;

    &:hover {
      opacity: 0.8;
    }

    .logo-icon {
      font-size: 24px;
    }
    .logo-text {
      font-size: 20px;
      font-weight: 700;
      color: $text-main;
      font-family: 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
      letter-spacing: -0.5px;
    }
  }

  .nav-links {
    display: flex;
    gap: 20px;
    transition:
      opacity 0.3s ease,
      transform 0.3s ease;

    &.fade-out {
      opacity: 0;
      transform: translateX(-10px);
      pointer-events: none; /* 隐藏时不可点击 */
    }

    .nav-item {
      text-decoration: none;
      color: #606266;
      font-size: 14px;
      font-weight: 500;
      padding: 4px 0;
      position: relative;

      &:hover,
      &.active {
        color: $text-main;
        font-weight: 600;
      }
    }
  }
}

/* --- 中间区域 (搜索) --- */
.center-section {
  flex: 1; /* 占据剩余空间 */
  max-width: 600px;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.5, 1);

  /* 聚焦时，搜索框容器稍微变宽（如果布局允许） */
  &.is-focused {
    flex: 1.2;
  }

  :deep(.search-input) {
    .el-input__wrapper {
      border-radius: 4px;
      box-shadow: 0 0 0 1px #dcdfe6 inset;
      padding: 4px 11px;
      transition: all 0.2s;

      &.is-focus {
        box-shadow:
          0 0 0 1px $primary-color inset,
          0 0 0 4px rgba($primary-color, 0.1);
      }
    }
  }
}

/* --- 右侧区域 --- */
.right-section {
  display: flex;
  align-items: center;
  gap: 16px;
  flex-shrink: 0;

  .action-group {
    display: flex;
    gap: 12px;
    overflow: hidden;
    max-width: 300px;
    opacity: 1;
    transition: all 0.4s cubic-bezier(0.25, 0.8, 0.5, 1);

    &.collapsed {
      max-width: 0;
      opacity: 0;
      margin: 0; /* 去除间距 */
    }

    .action-btn {
      margin: 0 !important; /* 覆盖 Element Plus 可能的默认 margin */
    }
  }

  .user-area {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-left: 12px;
    border-left: 1px solid $border-color; /* 分割线 */

    .avatar-wrapper {
      cursor: pointer;
      transition: transform 0.2s;
      &:hover {
        transform: scale(1.05);
      }
    }

    .auth-buttons {
      display: flex;
      gap: 8px;

      .login-btn {
        padding: 8px 16px;
        &:hover {
          background-color: #f5f7fa;
          color: $primary-color;
        }
      }

      .register-btn {
        padding: 8px 20px;
      }
    }
  }
}

/* 移动端适配 */
@media (max-width: 768px) {
  .nav-links,
  .action-group {
    display: none !important; /* 移动端隐藏导航和操作按钮 */
  }

  .center-section {
    margin: 0 10px;
  }

  .logo-text {
    display: none; /* 移动端只显示 Logo 图标 */
  }
}
</style>
