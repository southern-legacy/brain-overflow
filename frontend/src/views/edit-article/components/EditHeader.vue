<script setup>
import UserAvatar from '@/components/UserAvatar.vue'
import { ref } from 'vue'
const title = ref('')

// 定义事件
const emits = defineEmits(['update:modelValue', 'publish'])

// 使用计算属性处理 v-model，保持单向数据流

const handlePublish = () => {
  if (!title.value.trim()) return // 简单校验
  emits('publish')
}
</script>

<template>
  <header class="editor-header">
    <!-- 左侧：标题输入区域 -->
    <div class="input-wrapper">
      <el-input
        v-model="title"
        class="ghost-input"
        placeholder="请输入文章标题..."
        maxlength="100"
      />
    </div>

    <!-- 右侧：功能区域 -->
    <div class="actions-wrapper">
      <!-- 添加 loading 状态，防止重复提交 -->
      <el-button
        type="primary"
        class="publish-btn"
        :loading="loading"
        :disabled="!title.trim()"
        @click="handlePublish"
      >
        发布文章
      </el-button>

      <div class="avatar-box">
        <UserAvatar />
      </div>
    </div>
  </header>
</template>

<style scoped lang="scss">
.editor-header {
  height: 64px; /* 稍微增加高度，更大气 */
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: #ffffff;
  border-bottom: 1px solid #e1e4e8; /* 增加底部边框，区分内容区 */
  box-sizing: border-box;
  transition: all 0.3s ease;
}

.input-wrapper {
  flex: 1;
  margin-right: 20px;
  max-width: 800px; /* 限制最大宽度，防止在大屏上太长 */

  /* 深度选择器修改 Element Plus 样式 */
  :deep(.ghost-input) {
    --el-input-border-color: transparent;
    --el-input-hover-border-color: transparent;
    --el-input-focus-border-color: transparent;
    --el-input-bg-color: transparent;

    .el-input__wrapper {
      box-shadow: none !important; /* 移除默认阴影 */
      padding-left: 0;
      background-color: transparent;
    }

    .el-input__inner {
      font-size: 24px;
      font-weight: 700;
      color: #24292e;
      height: 40px;
      line-height: 40px;

      &::placeholder {
        color: #a0aec0;
        font-weight: 500;
      }
    }
  }
}

.actions-wrapper {
  display: flex;
  align-items: center;
  gap: 16px; /* 使用 gap 代替 margin，更现代 */

  .publish-btn {
    font-weight: 500;
    padding: 8px 20px;
    border-radius: 6px;
    /* 这里的颜色可以根据你的主题色调整 */
  }

  .avatar-box {
    display: flex;
    align-items: center;
    cursor: pointer;
    transition: opacity 0.2s;

    &:hover {
      opacity: 0.8;
    }
  }
}

@media (max-width: 768px) {
  .editor-header {
    padding: 0 12px;
    height: 56px;
  }

  .input-wrapper :deep(.ghost-input .el-input__inner) {
    font-size: 18px;
  }

  .publish-btn {
    padding: 8px 12px;
  }
}
</style>
