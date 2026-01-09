<!--
  * Component: AppHeader
  * Description:  Header Component for the main layout
-->

<script setup>
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'
import { useUserStore } from '@/stores'
import { Search, Edit, Monitor } from '@element-plus/icons-vue' // icons for buttons
import UserAvatar from '@/components/UserAvatar.vue'

const router = useRouter()
const userStore = useUserStore()

// state managemnet
const keyword = ref('')
const isSearchFocused = ref(false)

// login state
const isLoggedIn = computed(() => !!userStore.token)

// routes
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
</script>

<template>
  <header class="app-header">
    <div class="header-inner">
      <!-- 1. left part：Logo + navs -->
      <div class="left-section">
        <div class="logo" @click="handleCommand('home')">
          <span class="logo-icon">🧠</span>
          <span class="logo-text">Brain Overflow</span>
        </div>

        <!-- nav part -->
        <nav class="nav-links" :class="{ 'fade-out': isSearchFocused }">
          <a href="#" class="nav-item active">首页</a>
          <a href="#" class="nav-item">问答</a>
          <a href="#" class="nav-item">社区</a>
        </nav>
      </div>

      <!-- 2. middle：search box -->
      <!-- it will expand its width when focused -->
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

      <!-- 3. right part：buttons + user info -->
      <div class="right-section">
        <!-- buttons: display none when the search box is focused -->
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

        <!-- user info-->
        <div class="user-area">
          <template v-if="isLoggedIn">
            <!-- display avatar when login -->
            <div class="avatar-wrapper">
              <UserAvatar />
            </div>
          </template>

          <template v-else>
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
  position: sticky; /* stick at top */
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
  max-width: 1800px;
  padding: 0 20px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 20px;
}

/* --- left part --- */
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
      pointer-events: none; /* can not click when it is hidden */
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

/* --- middle (search box) --- */
.center-section {
  flex: 1;
  max-width: 600px;
  transition: all 0.3s cubic-bezier(0.25, 0.8, 0.5, 1);

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

/* --- right part(user info) --- */
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
      margin: 0; /* no margin */
    }

    .action-btn {
      margin: 0 !important;
    }
  }

  .user-area {
    display: flex;
    align-items: center;
    gap: 12px;
    padding-left: 12px;
    border-left: 1px solid $border-color; /* divider*/

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

@media (max-width: 768px) {
  .nav-links,
  .action-group {
    display: none !important;
  }

  .center-section {
    margin: 0 10px;
  }

  .logo-text {
    display: none;
  }
}
</style>
