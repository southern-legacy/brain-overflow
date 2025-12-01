<script setup></script>

<template>
  <div class="layout">
    <AppHeader />

    <div class="layout-body">
      <main class="content">
        <router-view />
      </main>
    </div>
  </div>
</template>

<script setup>
import AppHeader from '@/layout/components/AppHeader.vue'
</script>

<style scoped>
.layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.layout-body {
  display: flex;
  justify-content: center;
}

.content {
  overflow-y: auto;
  width: 1400px;
  padding: 24px 32px;
}
</style>
