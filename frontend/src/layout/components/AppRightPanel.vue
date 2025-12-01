<script setup>
const hotTags = ['javascript', 'vue', 'css', 'node.js']
</script>

<template>
  <div class="right-panel">
    <el-card shadow="never" class="item">
      <h3>Hot Tags</h3>
      <div class="tags">
        <el-tag v-for="tag in hotTags" :key="tag">{{ tag }}</el-tag>
      </div>
    </el-card>

    <el-card shadow="never" class="item">
      <h3>Top Users</h3>
      ...
    </el-card>
  </div>
</template>
