<script setup>
/**
 * todo:  动态数据： 头像， 用户关注，个人主页跳转，个人设置跳转，收藏页面，历史记录
 */
</script>

<template>
  <el-popover trigger="click" placement="bottom" :width="250">
    <!-- 触发器 -->
    <template #reference>
      <el-avatar class="avatar-trigger" />
    </template>

    <!-- 弹出内容 -->
    <div class="user-popover">
      <!-- 顶部用户信息 -->
      <el-row class="user-basic">
        <el-avatar size="large" />
        <div class="username">kenzin</div>
      </el-row>

      <!-- 数据区 -->
      <el-row class="user-stats">
        <div class="stat">
          <p class="num">0</p>
          <p class="label">关注</p>
        </div>

        <div class="stat">
          <p class="num">0</p>
          <p class="label">赞过</p>
        </div>

        <div class="stat">
          <p class="num">0</p>
          <p class="label">收藏</p>
        </div>
      </el-row>

      <!-- 功能区 -->
      <el-row class="user-actions" justify="middle">
        <el-link class="link" underline="never">个人主页</el-link>
        <el-link class="link" underline="never">用户设置</el-link>
        <el-link class="link" underline="never">收藏文章</el-link>
        <el-link class="link" underline="never">我的足迹</el-link>
      </el-row>

      <!-- 底部 -->
      <el-row class="user-footer">
        <el-link
          class="link"
          underline="never"
          href="https://github.com/southern-legacy/brain-overflow"
          >GitHub</el-link
        >
        <el-link class="link" underline="never">退出登录</el-link>
      </el-row>
    </div>
  </el-popover>
</template>
<style scoped lang="scss">
.user-popover {
  padding: 10px;
}

.user-basic {
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
  .username {
    font-weight: 600;
  }
}

.user-stats {
  justify-content: space-around;
  margin-bottom: 20px;
  .stat {
    text-align: center;

    .num {
      font-weight: 600;
    }

    .label {
      font-size: 12px;
      color: #86909c;
    }
  }
}

.user-actions {
  flex-wrap: wrap;
  justify-content: space-between;
  gap: 8px;
  margin-bottom: 20px;
  .link {
    width: 70px;
  }
}

.user-footer {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
}
</style>
