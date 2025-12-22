<script setup>
import { ref } from 'vue'
import { ArrowRight, Trophy, UserFilled } from '@element-plus/icons-vue'
import PanelCard from '@/components/PanelCard.vue'

// 模拟热门标签
const hotTags = ref([
  { id: 1, name: 'JavaScript', count: 230 },
  { id: 2, name: 'Vue.js', count: 180 },
  { id: 3, name: 'CSS3', count: 150 },
  { id: 4, name: 'Node.js', count: 120 },
  { id: 5, name: 'TypeScript', count: 98 },
  { id: 6, name: 'React', count: 85 },
])

// 模拟作者榜
const topUsers = ref([
  { id: 1, name: '前端小黑', level: 6, desc: '写代码的诗人' },
  { id: 2, name: 'CodeMaster', level: 5, desc: '全栈练习生' },
  { id: 3, name: 'BugHunter', level: 4, desc: '专注修 Bug 三十年' },
])
</script>

<template>
  <aside class="right-panel">
    <!-- 欢迎卡片 -->
    <PanelCard>
      <div class="welcome-title">下午好！</div>
      <div class="welcome-desc">点亮在社区的每一天</div>
      <el-button type="primary" plain class="w-full mt-2"> 立即签到 </el-button>
    </PanelCard>

    <!-- 热门标签 -->
    <PanelCard :marginTop="12">
      <template #header>
        <span class="title">热门标签</span>
        <el-button text class="more-btn">
          更多
          <el-icon><ArrowRight /></el-icon>
        </el-button>
      </template>

      <div class="tags-container">
        <el-tag v-for="tag in hotTags" :key="tag.id" class="custom-tag" effect="plain" round>
          {{ tag.name }}
        </el-tag>
      </div>
    </PanelCard>

    <!-- 作者榜 -->
    <PanelCard :marginTop="12">
      <template #header>
        <span class="title">
          <el-icon class="mr-1 text-yellow-500">
            <Trophy />
          </el-icon>
          作者榜
        </span>
      </template>

      <ul class="user-list">
        <li v-for="user in topUsers" :key="user.id" class="user-item">
          <el-avatar :size="36" :icon="UserFilled" class="user-avatar" />

          <div class="user-info">
            <div class="user-name">
              {{ user.name }}
              <img src="https://api.iconify.design/emojione:star.svg" class="level-icon" alt="lv" />
            </div>
            <div class="user-desc">
              {{ user.desc }}
            </div>
          </div>

          <el-button type="primary" link size="small"> + 关注 </el-button>
        </li>
      </ul>
    </PanelCard>

    <!-- 底部 -->
    <div class="footer-links mt-3">
      <a href="#">用户协议</a>
      <span class="divider">·</span>
      <a href="#">隐私政策</a>
      <span class="divider">·</span>
      <a href="#">关于我们</a>
      <div class="copyright">© 2025 Southern Legacy</div>
    </div>
  </aside>
</template>

<style scoped>
.right-panel {
  width: 260px;
  position: sticky;
  top: 10px;
  height: fit-content;
  flex-shrink: 0;
}

.mt-3 {
  margin-top: 12px;
}

.mt-2 {
  margin-top: 8px;
}

.w-full {
  width: 100%;
}

.title {
  font-size: 14px;
  font-weight: 600;
  color: #334155;
  display: flex;
  align-items: center;
}

.more-btn {
  font-size: 12px;
  color: #94a3b8;
  padding: 0;
}

.more-btn:hover {
  color: #409eff;
}

/* tags */
.tags-container {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.custom-tag {
  cursor: pointer;
  border-color: transparent;
  background-color: #f1f5f9;
  color: #475569;
  transition: all 0.2s;
}

.custom-tag:hover {
  background-color: #e2e8f0;
  color: #0f172a;
}

/* user list */
.user-list {
  list-style: none;
  padding: 0;
  margin: 0;
}

.user-item {
  display: flex;
  align-items: center;
  margin-bottom: 16px;
}

.user-item:last-child {
  margin-bottom: 0;
}

.user-avatar {
  background-color: #f1f5f9;
  flex-shrink: 0;
}

.user-info {
  flex: 1;
  margin-left: 10px;
  overflow: hidden;
}

.user-name {
  font-size: 13px;
  font-weight: 500;
  color: #333;
  display: flex;
  align-items: center;
}

.level-icon {
  width: 12px;
  height: 12px;
  margin-left: 4px;
}

.user-desc {
  font-size: 12px;
  color: #94a3b8;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-top: 2px;
}

/* footer */
.footer-links {
  padding: 0 8px;
  font-size: 12px;
  color: #94a3b8;
}

.footer-links a {
  color: #94a3b8;
  text-decoration: none;
}

.footer-links a:hover {
  color: #409eff;
}

.divider {
  margin: 0 6px;
}

.copyright {
  margin-top: 6px;
  color: #cbd5e1;
}

/* welcome */
.welcome-title {
  font-size: 16px;
  font-weight: 600;
  color: #1e293b;
}

.welcome-desc {
  font-size: 12px;
  color: #64748b;
  margin: 4px 0 12px 0;
}
</style>
