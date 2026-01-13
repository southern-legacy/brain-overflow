<script setup>
import { watch, ref, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useUserAssetStore, useUserStore } from '@/stores'

const router = useRouter()
const userStore = useUserStore()
const userAssetStore = useUserAssetStore()

// avatar show src
const avatarSrc = ref('')
const userBiographyMarkdown = ref('')
// if avatar assetid changes， try to get a new URL
watch(
  () => userStore.userProfile.avatar,
  async (newVal) => {
    if (!newVal) return
    const blob = await userAssetStore.getAssetBlob(newVal)
    const url = URL.createObjectURL(blob)
    avatarSrc.value = url
  },
  { immediate: true },
)

watch(
  () => userStore.userProfile.biography,
  async (newVal) => {
    if (!newVal) return
    const blob = await userAssetStore.getAssetBlob(newVal)
    const text = await blob.text()
    userBiographyMarkdown.value = text
  },
  { immediate: true },
)

// revoke the blob url
onUnmounted(() => {
  if (avatarSrc.value) {
    URL.revokeObjectURL(avatarSrc.value)
  }
})

// --- 处理 ContactMe 显示逻辑 ---
// 映射类型到显示的 Label 或 图标名
const formatContactType = (type) => {
  const map = {
    github: 'GitHub',
    email: '邮箱',
    wechat: '微信',
    qq: 'QQ',
    blog: '博客',
    twitter: 'Twitter',
  }
  return map[type.toLowerCase()] || type.toUpperCase()
}

// 根据类型返回不同的 Tag 颜色
const getContactTagType = (type) => {
  const t = type.toLowerCase()
  if (['github', 'blog'].includes(t)) return 'info'
  if (['email'].includes(t)) return 'warning'
  if (['wechat'].includes(t)) return 'success'
  return '' // default primary
}

// 处理点击复制或跳转 (可选功能)
const handleContactClick = (type, val) => {
  if (type === 'github' || type === 'blog' || val.startsWith('http')) {
    window.open(val.startsWith('http') ? val : `https://${val}`, '_blank')
  } else {
    // simple copy logic
    navigator.clipboard.writeText(val).then(() => {
      ElMessage({
        type: 'success',
        message: '成功复制',
      })
    })
  }
}
</script>

<template>
  <div class="profile-page">
    <div class="profile-layout">
      <!-- 左侧主区域 -->
      <main class="profile-main">
        <!-- 1. 头部卡片：纯粹的身份展示 -->
        <el-card class="profile-header">
          <div class="user-identity">
            <el-avatar :size="80" :src="avatarSrc" class="avatar" />
            <div class="names">
              <h1 class="nickname">{{ userStore.userInfo.name }}</h1>
              <span class="uid">UID: {{ userStore.userInfo.id || 'Unknown' }}</span>
            </div>
          </div>
          <el-button plain round @click="router.push('/user/settings')"> 编辑资料 </el-button>
        </el-card>

        <!-- 2. 新增：个人简介 + 联系方式模块 -->
        <el-card class="bio-card">
          <template #header>
            <div class="card-header">
              <span>个人简介</span>
            </div>
          </template>

          <!-- 这里放置你的 Markdown 组件 -->
          <div class="md-content">
            <p v-if="!userStore.userProfile.biography" class="empty-text">
              这个人很懒，什么都没写...
            </p>
            <!-- 示例占位符 -->
            <div v-else class="markdown-placeholder">
              <v-md-editor mode="preview" :model-value="userBiographyMarkdown"></v-md-editor>
            </div>
          </div>

          <el-divider border-style="dashed" />

          <!-- 联系方式列表 -->
          <div class="contact-section">
            <div class="section-title">联系我</div>

            <div v-if="userStore.userProfile.contactMe.length > 0" class="contact-list">
              <!-- 循环渲染 contactMe: [type, str] -->
              <div
                v-for="([type, val], index) in userStore.userProfile.contactMe"
                :key="index"
                class="contact-item"
                @click="handleContactClick(type, val)"
              >
                <el-tag :type="getContactTagType(type)" effect="light" round>
                  {{ formatContactType(type) }}
                </el-tag>
                <span class="contact-value">{{ val }}</span>
              </div>
            </div>

            <div v-else class="empty-text">暂未设置联系方式</div>
          </div>
        </el-card>

        <!-- 3. 内容区：文章/Tabs -->
        <el-card class="profile-content">
          <el-tabs>
            <el-tab-pane label="文章" name="articles">
              <div class="placeholder">文章列表区域</div>
            </el-tab-pane>
            <el-tab-pane label="关注" name="following">
              <div class="placeholder">关注列表区域</div>
            </el-tab-pane>
            <el-tab-pane label="收藏" name="favorites">
              <div class="placeholder">收藏列表区域</div>
            </el-tab-pane>
          </el-tabs>
        </el-card>
      </main>

      <!-- 右侧：数据栏 (保持原样或微调) -->
      <aside class="profile-sidebar">
        <el-card class="user-stats-card" shadow="hover">
          <div class="stat-box">
            <div class="label">关注了</div>
            <div class="value">12</div>
          </div>
          <el-divider direction="vertical" class="stats-divider" />
          <div class="stat-box">
            <div class="label">关注者</div>
            <div class="value">345</div>
          </div>
        </el-card>

        <!-- 详细数据面板 -->
        <el-card class="info-card" shadow="hover">
          <div class="info-row">
            <span class="label">获赞数</span>
            <span class="num">1,024</span>
          </div>
          <div class="info-row">
            <span class="label">文章数</span>
            <span class="num">8</span>
          </div>
          <div class="info-row">
            <span class="label">加入时间</span>
            <span class="time">2024-05-20</span>
          </div>
        </el-card>
      </aside>
    </div>
  </div>
</template>

<style scoped lang="scss">
.profile-page {
  max-width: 1250px;
  margin: 0 auto;
  padding: 20px;
}

.profile-layout {
  display: grid;
  grid-template-columns: 1fr 300px;
  gap: 20px;
  align-items: start;
}

.profile-main {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.profile-header {
  :deep(.el-card__body) {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 30px;
  }

  .user-identity {
    display: flex;
    align-items: center;
    gap: 20px;

    .names {
      display: flex;
      flex-direction: column;
      gap: 4px;

      .nickname {
        margin: 0;
        font-size: 24px;
        font-weight: 700;
        color: #1d2129;
      }
      .uid {
        font-size: 12px;
        color: #86909c;
        background: #f2f3f5;
        padding: 2px 6px;
        border-radius: 4px;
        width: fit-content;
      }
    }
  }
}

.bio-card {
  .card-header {
    font-weight: 600;
  }

  .md-content {
    min-height: 60px;
    color: #4e5969;
    line-height: 1.6;
    font-size: 15px;

    .empty-text {
      color: #c9cdd4;
      font-style: italic;
    }
  }

  .contact-section {
    .section-title {
      font-size: 14px;
      color: #86909c;
      margin-bottom: 12px;
    }

    .contact-list {
      display: flex;
      flex-wrap: wrap;
      gap: 12px;
    }

    .contact-item {
      display: flex;
      align-items: center;
      gap: 8px;
      background: #f7f8fa;
      padding: 6px 12px;
      border-radius: 6px;
      cursor: pointer;
      transition: all 0.2s;
      border: 1px solid transparent;

      &:hover {
        background: #fff;
        border-color: var(--el-color-primary-light-5);
        box-shadow: 0 2px 8px rgba(0, 0, 0, 0.05);

        .contact-value {
          color: var(--el-color-primary);
        }
      }

      .contact-value {
        font-size: 14px;
        color: #4e5969;
        font-family: monospace;
      }
    }
  }
}

.profile-content {
  min-height: 400px;
  .placeholder {
    padding: 40px;
    text-align: center;
    color: #c9cdd4;
  }
}

.profile-sidebar {
  display: flex;
  flex-direction: column;
  gap: 16px;

  .user-stats-card {
    :deep(.el-card__body) {
      display: flex;
      justify-content: space-evenly;
      align-items: center;
      padding: 20px 0;
    }

    .stat-box {
      text-align: center;
      cursor: pointer;
      &:hover .value {
        color: var(--el-color-primary);
      }

      .label {
        font-size: 13px;
        color: #86909c;
        margin-bottom: 4px;
      }
      .value {
        font-size: 18px;
        font-weight: 600;
        color: #1d2129;
        transition: color 0.2s;
      }
    }

    .stats-divider {
      height: 24px;
    }
  }

  .info-card {
    .info-row {
      display: flex;
      justify-content: space-between;
      align-items: center;
      padding: 12px 0;
      border-bottom: 1px solid #f2f3f5;
      font-size: 14px;

      &:last-child {
        border-bottom: none;
      }

      .label {
        color: #86909c;
      }
      .num {
        font-weight: 600;
        color: #1d2129;
      }
      .time {
        color: #4e5969;
      }
    }
  }
}

/* 响应式适配 */
@media (max-width: 768px) {
  .profile-layout {
    grid-template-columns: 1fr;
  }
  .profile-header :deep(.el-card__body) {
    flex-direction: column;
    gap: 16px;
    align-items: flex-start;

    .el-button {
      width: 100%;
    }
  }
}
</style>
