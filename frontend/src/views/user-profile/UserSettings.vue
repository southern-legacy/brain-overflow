<script setup>
import { Tickets, User, Setting, Message, Back } from '@element-plus/icons-vue'
import { useRouter } from 'vue-router'

let router = useRouter()
</script>

<template>
  <div class="setting-page">
    <el-card class="setting-nav">
      <el-menu default-active="2" class="nav-menu">
        <el-menu-item index="1" @click="router.push('/user-profile')">
          <div>
            <el-icon><Back /></el-icon>
            <span>返回资料</span>
          </div>
        </el-menu-item>

        <el-menu-item index="2">
          <div>
            <el-icon><Tickets></Tickets></el-icon>
            <span>资料设置</span>
          </div>
        </el-menu-item>

        <el-menu-item index="3">
          <el-icon><User></User></el-icon>
          <span>账号设置</span>
        </el-menu-item>

        <el-menu-item index="4">
          <el-icon><setting></setting></el-icon>
          <span>通用设置</span>
        </el-menu-item>

        <el-menu-item index="5">
          <el-icon><Message></Message></el-icon>
          <span>消息设置</span>
        </el-menu-item>
      </el-menu>
    </el-card>

    <el-card class="setting">
      <template #header> 个人资料 </template>
      <!-- 后续在这里进行新的路由 -->
      <router-view></router-view>
    </el-card>
  </div>
</template>

<style scoped lang="scss">
.setting-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
  display: flex;

  .setting-nav {
    width: 230px;
    margin-right: 20px;
    :deep(.el-card__body) {
      padding: 8px;
      padding-left: 0;
      padding-right: 0;
    }
    .el-menu-item {
      justify-content: start;
      font-size: 17px;
    }
  }
  .setting {
    flex: 1;
  }
}
</style>
