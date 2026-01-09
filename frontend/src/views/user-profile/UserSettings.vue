<script setup>
import { Tickets, User, Setting, Message, Back } from '@element-plus/icons-vue'
import { useRouter, useRoute } from 'vue-router'

let router = useRouter()
let route = useRoute()
</script>

<template>
  <div class="setting-page">
    <!-- 左侧导航 -->
    <el-card class="setting-nav" shadow="never">
      <el-menu default-active="2" class="nav-menu">
        <!-- 特殊处理：返回按钮 -->
        <el-menu-item index="1" class="nav-item back-btn" @click="router.push('/user/profile')">
          <el-icon><Back /></el-icon>
          <span>返回资料</span>
        </el-menu-item>

        <!-- 分割线，视觉上区分操作和导航 -->
        <div class="nav-divider"></div>

        <el-menu-item index="2" class="nav-item" @click="router.push('/user/settings/profile')">
          <el-icon><Tickets /></el-icon>
          <span>资料设置</span>
        </el-menu-item>

        <el-menu-item index="3" class="nav-item" @click="router.push('/user/settings/account')">
          <el-icon><User /></el-icon>
          <span>账号设置</span>
        </el-menu-item>

        <el-menu-item index="4" class="nav-item" @click="router.push('/user/settings/general')">
          <el-icon><Setting /></el-icon>
          <span>通用设置</span>
        </el-menu-item>

        <el-menu-item
          index="5"
          class="nav-item"
          @click="router.push('/user/settings/notification')"
        >
          <el-icon><Message /></el-icon>
          <span>消息设置</span>
        </el-menu-item>
      </el-menu>
    </el-card>

    <!-- 右侧内容 -->
    <el-card class="setting-content" shadow="never">
      <template #header>
        <div class="content-header">
          <span class="title">{{ route.meta.title ?? '设置' }}</span>
          <span class="subtitle">{{ route.meta.subTitle ?? '欢迎来到设置界面' }}</span>
        </div>
      </template>
      <div class="content-body">
        <router-view></router-view>
      </div>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
$primary-color: var(--el-color-primary);
$primary-light: var(--el-color-primary-light-9);
$hover-bg: var(--el-fill-color-light);
$border-radius: 8px;

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
      }
      .subtitle {
        font-size: 13px;
        color: var(--el-text-color-secondary);
      }
    }

    .content-body {
      min-height: 500px;
    }
  }
}
</style>
