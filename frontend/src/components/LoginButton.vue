<template>
  <el-button class="custom-btn" v-bind="$attrs" v-on="$listeners">
    <slot />
  </el-button>
</template>

<style scoped>
.custom-btn {
  height: 46px;
  border-radius: 24px;
  background: linear-gradient(135deg, #f7c8d1, #d797a3);
  border: none;
  color: #520d27;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: 0.5px;
  box-shadow: 0 6px 14px rgba(215, 151, 163, 0.4);
  transition: 0.3s ease;
}

.custom-btn:hover {
  background: linear-gradient(135deg, #ffdde1, #ee9ca7);
  transform: translateY(-1px);
}

.custom-btn:active {
  transform: translateY(0);
}
</style>