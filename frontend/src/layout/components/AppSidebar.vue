<script setup>
import { computed } from 'vue'
import { useRoute } from 'vue-router'
import { ChatLineSquare, PriceTag, User, Document, Trophy, Star } from '@element-plus/icons-vue'

const route = useRoute()

// 获取当前激活的菜单项，默认高亮当前路由路径
const activeMenu = computed(() => route.path)
</script>

<template>
  <aside class="sidebar-container">
    <!-- 模块一：核心导航 -->
    <div class="sidebar-card">
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
    </div>

    <!-- 模块二：发现/个人 (模拟掘金的分组) -->
    <div class="sidebar-card mt-3">
      <div class="card-header">发现</div>
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
    </div>

    <!-- 模块三-->
    <div class="promo-card mt-3">
      <div class="promo-content">
        <h4>炉管挑战赛</h4>
        <p>参与赢取限量周边！</p>
      </div>
    </div>
  </aside>
</template>

<style scoped>
/* 
  容器布局：
  1. 宽度固定
  2. sticky 定位实现"粘性"效果
*/
.sidebar-container {
  width: 240px;
  position: sticky;
  top: 10px;
  height: fit-content;
  flex-shrink: 0;
}

.sidebar-card {
  background: #ffffff;
  border-radius: 8px;
  padding: 8px;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  border: 1px solid #f1f5f9; /* 非常淡的边框 */
}

.mt-3 {
  margin-top: 12px;
}

.card-header {
  padding: 8px 16px;
  font-size: 12px;
  font-weight: 600;
  color: #94a3b8;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

/* --- Element Plus Menu 深度覆盖 --- */

/* 移除默认的右边框 */
.custom-menu {
  border-right: none !important;
  background: transparent !important;
}

/* 菜单项基础样式 */
:deep(.el-menu-item) {
  height: 48px;
  line-height: 48px;
  margin-bottom: 4px;
  border-radius: 6px;
  color: #64748b; /* text-slate-500 */
  font-weight: 500;
  border: none;
}

/* 悬停状态 */
:deep(.el-menu-item:hover) {
  background-color: #f8fafc !important; /* slate-50 */
  color: #0f172a; /* slate-900 */
}

/* 激活/选中状态 */
:deep(.el-menu-item.is-active) {
  background-color: #eff6ff !important; /* blue-50 */
  color: #2563eb !important; /* blue-600 */
  font-weight: 600;
}

:deep(.el-menu-item .el-icon) {
  font-size: 18px;
  margin-right: 10px;
  /* 图标颜色跟随文字颜色变化 */
  color: inherit;
}

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
