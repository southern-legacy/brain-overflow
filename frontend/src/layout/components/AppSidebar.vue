<!--
  * Component: AppSideBar
  * Description:  left part of the main layout. Used to display common navigation links and promotional information
-->

<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ChatLineSquare, PriceTag, User, Document, Trophy, Star } from '@element-plus/icons-vue'
import PanelCard from '@/components/PanelCard.vue'

const route = useRoute()

// 当前激活菜单
const activeMenu = computed(() => route.path)
</script>

<template>
  <aside class="sidebar-container">
    <!-- part 1：common  navs -->
    <PanelCard padding="8px">
      <el-menu :default-active="activeMenu" router class="custom-menu">
        <el-menu-item index="/questions">
          <el-icon><ChatLineSquare /></el-icon>
          <span>问题广场</span>
        </el-menu-item>

        <el-menu-item index="/articles">
          <el-icon><Document /></el-icon>
          <span>精选文章</span>
        </el-menu-item>

        <el-menu-item index="/tags">
          <el-icon><PriceTag /></el-icon>
          <span>热门标签</span>
        </el-menu-item>

        <el-menu-item index="/users">
          <el-icon><User /></el-icon>
          <span>活跃用户</span>
        </el-menu-item>
      </el-menu>
    </PanelCard>

    <!-- part 2：findings(navs) -->
    <PanelCard padding="8px" :marginTop="12">
      <template #header>
        <div class="card-header">发现</div>
      </template>

      <el-menu :default-active="activeMenu" router class="custom-menu">
        <el-menu-item index="/trending">
          <el-icon><Trophy /></el-icon>
          <span>全站热榜</span>
        </el-menu-item>

        <el-menu-item index="/following">
          <el-icon><Star /></el-icon>
          <span>我的关注</span>
        </el-menu-item>
      </el-menu>
    </PanelCard>

    <!-- part3：promotion-->
    <div class="promo-card mt-3">
      <div class="promo-content">
        <h4>炉管挑战赛</h4>
        <p>参与赢取限量周边！</p>
      </div>
    </div>
  </aside>
</template>

<style scoped>
.sidebar-container {
  width: 240px;
  position: sticky;
  top: 10px;
  height: fit-content;
  flex-shrink: 0;
}

.mt-3 {
  margin-top: 12px;
}

/* header */
.card-header {
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* --- Element Plus Menu styles--- */
.custom-menu {
  border-right: none !important;
  background: transparent !important;
}

:deep(.el-menu-item) {
  height: 48px;
  line-height: 48px;
  margin-bottom: 4px;
  border-radius: 6px;
  color: #64748b;
  font-weight: 500;
}

:deep(.el-menu-item:hover) {
  background-color: #f8fafc !important;
  color: #0f172a;
}

:deep(.el-menu-item.is-active) {
  background-color: #eff6ff !important;
  color: #2563eb !important;
  font-weight: 600;
}

:deep(.el-menu-item .el-icon) {
  font-size: 18px;
  margin-right: 10px;
  color: inherit;
}

/* promo card */
.promo-card {
  background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
  border-radius: 8px;
  padding: 16px;
  color: white;
  cursor: pointer;
  transition: transform 0.2s;
}

.promo-card:hover {
  transform: translateY(-2px);
}

.promo-content h4 {
  margin: 0 0 4px 0;
  font-size: 14px;
  font-weight: bold;
}

.promo-content p {
  margin: 0;
  font-size: 12px;
  opacity: 0.9;
}
</style>
