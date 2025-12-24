<!--
  * Component: EditHeader
  * Description:  the custom header for edit page
-->

<script setup>
import UserAvatar from '@/components/UserAvatar.vue'
import { ref } from 'vue'
const title = ref('')

// define emits
const emits = defineEmits(['update:modelValue', 'publish'])

const handlePublish = () => {
  if (!title.value.trim()) return // simple validates
  emits('publish')
}
</script>

<template>
  <header class="editor-header">
    <!-- left part：the input of the title -->
    <div class="input-wrapper">
      <el-input
        v-model="title"
        class="ghost-input"
        placeholder="请输入文章标题..."
        maxlength="100"
      />
    </div>

    <!-- left part：functional area -->
    <div class="actions-wrapper">
      <!-- add loading state to prevent duplicate submission -->
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
  height: 64px;
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background-color: #ffffff;
  border-bottom: 1px solid #e1e4e8;
  box-sizing: border-box;
  transition: all 0.3s ease;
}

.input-wrapper {
  flex: 1;
  margin-right: 20px;
  max-width: 800px;

  :deep(.ghost-input) {
    --el-input-border-color: transparent;
    --el-input-hover-border-color: transparent;
    --el-input-focus-border-color: transparent;
    --el-input-bg-color: transparent;

    .el-input__wrapper {
      box-shadow: none !important;
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
  gap: 16px;

  .publish-btn {
    font-weight: 500;
    padding: 8px 20px;
    border-radius: 6px;
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
