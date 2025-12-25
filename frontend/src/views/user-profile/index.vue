<script setup>
// 这里先不接真实数据，只做 layout 骨架
import { useRouter } from 'vue-router'
let router = useRouter()
</script>

<template>
  <div class="profile-page">
    <div class="profile-layout">
      <!-- 左侧：Header + 内容区 -->
      <main class="profile-main">
        <!-- Header：头像 + 基本信息 + 设置 -->
        <el-card class="profile-header">
          <div class="user-basic">
            <el-avatar :size="80" />

            <div class="info">
              <h2 class="username">用户名</h2>
              <p class="desc">这里是个人简介，可以写一句话</p>
            </div>
          </div>

          <el-button
            type="primary"
            class="edit-profile-button"
            @click="router.push('/user/settings')"
            >编辑资料</el-button
          >
        </el-card>

        <!-- 内容区：文章 / 关注 / 收藏 -->
        <el-card class="profile-content">
          <el-tabs>
            <el-tab-pane label="文章">
              <div class="placeholder">文章列表区域</div>
            </el-tab-pane>

            <el-tab-pane label="关注">
              <div class="placeholder">关注列表区域</div>
            </el-tab-pane>

            <el-tab-pane label="收藏">
              <div class="placeholder">收藏列表区域</div>
            </el-tab-pane>
          </el-tabs>
        </el-card>
      </main>

      <!-- 右侧：用户数据栏 -->
      <aside class="profile-sidebar">
        <el-card class="user-stats-card">
          <div class="follower">
            <div>关注了</div>
            <div>1</div>
          </div>

          <el-divider direction="vertical" class="user-stats-card-divider" />

          <div class="fans">
            <div>关注者</div>
            <div>1</div>
          </div>
        </el-card>

        <div class="stat-item">
          <span class="label">关注</span>
          <span class="num">12</span>
        </div>

        <div class="stat-item">
          <span class="label">收藏</span>
          <span class="num">8</span>
        </div>

        <div class="stat-item">
          <span class="label">点赞</span>
          <span class="num">56</span>
        </div>

        <div class="stat-item">
          <span class="label">加入时间</span>
          <span class="value">2024-05</span>
        </div>
      </aside>
    </div>
  </div>
</template>

<style scoped lang="scss">
.profile-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
}

/* 外层两列布局 */
.profile-layout {
  display: grid;
  grid-template-columns: 1fr 280px;
  gap: 24px;
  align-items: start;
}

.profile-main {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.profile-header {
  :deep(.el-card__body) {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .user-basic {
    display: flex;
    align-items: center;
    gap: 16px;

    .username {
      margin: 0;
    }

    .desc {
      font-size: 14px;
      color: #86909c;
    }
  }

  .edit-profile-button {
    align-self: flex-end;
  }
}

.profile-content {
  background: #fff;
  padding: 16px;
  border-radius: 8px;
}

.placeholder {
  padding: 24px;
  color: #86909c;
}

/* 右侧用户数据栏 */
.profile-sidebar {
  background: #fff;

  border-radius: 8px;

  display: flex;
  flex-direction: column;
  gap: 16px;

  .user-stats-card {
    margin-bottom: 20px;
    :deep(.el-card__body) {
      display: flex;
      justify-content: space-around;
      align-items: center;
    }

    .follower {
      display: flex;
      flex-direction: column;
      text-align: center;
    }
    .fans {
      display: flex;
      flex-direction: column;
      text-align: center;
    }
    .user-stats-card-divider {
      height: 3em;
      border-left: 2px var(--el-border-color) var(--el-border-style);
    }
  }

  .stat-item {
    display: flex;
    justify-content: space-between;
    font-size: 14px;

    .num {
      font-weight: 600;
    }

    .label {
      color: #86909c;
    }
  }
}
</style>
