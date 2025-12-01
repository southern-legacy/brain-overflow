<template>
  <nav class="sidebar">
    <el-menu default-active="questions" router class="menu">
      <el-menu-item index="/questions">Questions</el-menu-item>
      <el-menu-item index="/tags">Tags</el-menu-item>
      <el-menu-item index="/users">Users</el-menu-item>
      <el-menu-item index="/articles">Articles</el-menu-item>
    </el-menu>
  </nav>
</template>

<style scoped>
.sidebar {
  padding: 16px 8px;
}
.menu {
  border-right: none;
}
</style>
