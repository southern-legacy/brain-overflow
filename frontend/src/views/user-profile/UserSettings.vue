<script setup>
import { computed } from 'vue'
import { Tickets, User, Setting, Message, Back } from '@element-plus/icons-vue'
import { useRouter, useRoute } from 'vue-router'

defineOptions({ name: 'UserSettings' })
const router = useRouter()
const route = useRoute()

const activeMenu = computed(() => {
  if (route.path.includes('/profile')) return '2'
  if (route.path.includes('/account')) return '3'
  if (route.path.includes('/general')) return '4'
  if (route.path.includes('/notification')) return '5'
  return '2'
})

/**
 * use View Transitions API
 * @param path route path
 *
 */
const handleNavigation = async (path) => {
  if (route.path === path) return

  // whether the browser is able to use View Transitions API
  if (document.startViewTransition) {
    const transition = document.startViewTransition(async () => {
      await router.push(path)
    })

    await transition.finished
  } else {
    // if not use normal router jump
    router.push(path)
  }
}
</script>

<template>
  <div class="setting-page">
    <el-card class="setting-nav" shadow="never">
      <el-menu :default-active="activeMenu" class="nav-menu">
        <el-menu-item
          index="1"
          class="nav-item back-btn"
          @click="handleNavigation('/user/profile')"
        >
          <el-icon><Back /></el-icon>
          <span>返回资料</span>
        </el-menu-item>

        <div class="nav-divider"></div>

        <el-menu-item
          index="2"
          class="nav-item"
          @click="handleNavigation('/user/settings/profile')"
        >
          <el-icon><Tickets /></el-icon>
          <span>资料设置</span>
        </el-menu-item>

        <el-menu-item
          index="3"
          class="nav-item"
          @click="handleNavigation('/user/settings/account')"
        >
          <el-icon><User /></el-icon>
          <span>账号设置</span>
        </el-menu-item>

        <el-menu-item
          index="4"
          class="nav-item"
          @click="handleNavigation('/user/settings/general')"
        >
          <el-icon><Setting /></el-icon>
          <span>通用设置</span>
        </el-menu-item>

        <el-menu-item
          index="5"
          class="nav-item"
          @click="handleNavigation('/user/settings/notification')"
        >
          <el-icon><Message /></el-icon>
          <span>消息设置</span>
        </el-menu-item>
      </el-menu>
    </el-card>

    <el-card class="setting-content" shadow="never">
      <template #header>
        <div class="content-header">
          <transition name="fade" mode="out-in">
            <div :key="route.path">
              <span class="title">{{ route.meta.title ?? '设置' }}</span>
              <div class="subtitle">{{ route.meta.subTitle ?? '管理您的个人信息与偏好' }}</div>
            </div>
          </transition>
        </div>
      </template>

      <div class="content-body">
        <router-view v-slot="{ Component, route }">
          <transition name="fade" mode="out-in">
            <keep-alive
              :include="[
                'GeneralSetting',
                'ProfileSetting',
                'NotificationSetting',
                'AccountSettings',
              ]"
            >
              <component :is="Component" :key="route.fullPath" />
            </keep-alive>
          </transition>
        </router-view>
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
$primary-color: var(--el-color-primary);
$primary-light: var(--el-color-primary-light-9);
$hover-bg: var(--el-fill-color-light);
$border-radius: 8px;

/* 
  route transition animation
  used for Vue Transition Component
*/
.slide-fade-enter-active,
.slide-fade-leave-active {
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
}

.slide-fade-enter-from {
  opacity: 0;
  transform: translateX(20px);
}

.slide-fade-leave-to {
  opacity: 0;
  transform: translateX(-20px);
}

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

.setting-page {
  max-width: 1600px;
  margin: 40px auto;
  padding: 0 24px;
  display: flex;
  gap: 24px;
  align-items: flex-start;

  .setting-nav {
    width: 260px;
    border: none;
    background: transparent;
    position: sticky;
    top: 20px;

    :deep(.el-card__body) {
      padding: 0;
    }

    .nav-menu {
      border-right: none;
      background: transparent;
    }

    .nav-divider {
      height: 1px;
      background-color: var(--el-border-color-lighter);
      margin: 12px 16px;
    }

    .nav-item {
      margin: 4px 0;
      margin-right: 10px;
      border-radius: 0 24px 24px 0;
      height: 50px;
      line-height: 50px;
      color: var(--el-text-color-regular);
      font-weight: 500;
      transition: all 0.3s;
      user-select: none; /* 防止快速点击时选中文本 */

      .el-icon {
        font-size: 18px;
        margin-right: 12px;
        vertical-align: middle;
      }

      &:hover {
        background-color: $hover-bg;
        color: var(--el-text-color-primary);
        transform: translateX(4px);
      }

      &.is-active {
        background-color: $primary-light;
        color: $primary-color;
        font-weight: 600;

        &::before {
          content: '';
          position: absolute;
          left: 0;
          top: 10px;
          bottom: 10px;
          width: 4px;
          background-color: $primary-color;
          border-radius: 0 4px 4px 0;
        }
      }
    }

    .back-btn {
      color: var(--el-text-color-secondary);
      &:hover {
        background-color: transparent;
        color: var(--el-text-color-primary);
      }
      &.is-active {
        background: transparent;
        color: var(--el-text-color-primary);
        &::before {
          display: none;
        }
      }
    }
  }

  .setting-content {
    flex: 1;
    border-radius: $border-radius;
    border: 1px solid var(--el-border-color-lighter);
    overflow: hidden;

    :deep(.el-card__header) {
      padding: 20px 24px;
      border-bottom: 1px solid var(--el-border-color-lighter);
    }

    .content-header {
      display: flex;
      flex-direction: column;

      .title {
        font-size: 18px;
        font-weight: 600;
        color: var(--el-text-color-primary);
        margin-bottom: 4px;
        display: block;
      }
      .subtitle {
        font-size: 13px;
        color: var(--el-text-color-secondary);
        margin-top: 4px;
      }
    }

    .content-body {
      min-height: 500px;
      padding: 24px;
      position: relative;
    }
  }
}
</style>
