<!--
  * Component: PanelCard
  * Description: Reusable layout container for sidebar and right-panel sections.
  * Props:
      - padding (String, default '16px'): custom padding for the container
      - marginTop (Number, default 0): top margin in px
  * Slots:
      - header: optional header content
      - default: main content
-->

<script setup>
import { computed } from 'vue'

/**
 * * Padding inside the panel card (e.g., '16px', '1rem')
 */
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

/**
 * * Generates style object for the panel card
 * * based on padding and marginTop props
 */
const style = computed(() => ({
  padding: props.padding,
  marginTop: props.marginTop ? `${props.marginTop}px` : undefined,
}))
</script>

<template>
  <div class="panel-card" :style="style">
    <!-- Header slot -->
    <div v-if="$slots.header" class="card-header">
      <slot name="header" />
    </div>

    <!-- Default Slot -->
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
