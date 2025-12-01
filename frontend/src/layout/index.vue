<script setup></script>

<template>
  <div class="layout">
    <AppHeader />

    <div class="layout-body">
      <AppSidebar class="sidebar" />

      <main class="content">
        <router-view />
      </main>

      <AppRightPanel class="right-panel" />
    </div>
  </div>
</template>

<script setup>
import AppHeader from './components/AppHeader.vue'
import AppSidebar from './components/AppSidebar.vue'
import AppRightPanel from './components/AppRightPanel.vue'
</script>

<style scoped>
.layout {
  display: flex;
  flex-direction: column;
  height: 100vh;
}

.layout-body {
  display: grid;
  grid-template-columns: 240px 1fr 300px;
  flex: 1;
  overflow: hidden;
}

.content {
  overflow-y: auto;
  padding: 24px 32px;
}

.sidebar {
  border-right: 1px solid #ddd;
  overflow-y: auto;
}

.right-panel {
  border-left: 1px solid #ddd;
  overflow-y: auto;
}
</style>
