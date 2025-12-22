<!-- PanelCard: reusable layout container for sidebar and right-panel sections -->
<script setup>
import { computed } from 'vue'

const props = defineProps({
  padding: {
    type: String,
    default: '16px',
  },
  marginTop: {
    type: Number,
    default: 0,
  },
})

const style = computed(() => ({
  padding: props.padding,
  marginTop: props.marginTop ? `${props.marginTop}px` : undefined,
}))
</script>

<template>
  <div class="panel-card" :style="style">
    <!-- Header 插槽 -->
    <div v-if="$slots.header" class="card-header">
      <slot name="header" />
    </div>

    <!-- 默认内容 -->
    <slot />
  </div>
</template>

<style scoped>
.panel-card {
  background: #ffffff;
  border-radius: 8px;
  box-shadow: 0 1px 2px 0 rgba(0, 0, 0, 0.05);
  border: 1px solid #f1f5f9;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
  padding-bottom: 8px;
  border-bottom: 1px solid #f8fafc;
}
</style>
