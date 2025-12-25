<!--
  * Component: Layout index
  * Description:  Main layout for the app, contains header, sidebar, content, and right panel.
-->

<script setup>
import AppHeader from './components/AppHeader.vue'
import AppSidebar from './components/AppSidebar.vue'
import AppRightPanel from './components/AppRightPanel.vue'
</script>

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
  /* 隐藏所有滚动条 */
  scrollbar-width: none; /* Firefox */
  -ms-overflow-style: none; /* IE 和 Edge */

  /* Chrome, Safari 和 Opera */
  &::-webkit-scrollbar {
    display: none;
  }
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
