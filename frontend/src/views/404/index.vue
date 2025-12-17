<script setup>
import { ref, computed } from 'vue'
import { useRouter } from 'vue-router'

const router = useRouter()

// 视差效果状态
const mouseX = ref(0)
const mouseY = ref(0)

// 处理鼠标移动，计算视差偏移量
const handleMouseMove = (e) => {
  // 归一化鼠标位置 (-1 到 1)
  const x = (e.clientX / window.innerWidth) * 2 - 1
  const y = (e.clientY / window.innerHeight) * 2 - 1

  mouseX.value = x
  mouseY.value = y
}

// 计算样式
const parallaxStyle = computed(() => {
  return {
    transform: `translate(${mouseX.value * 15}px, ${mouseY.value * 15}px)`,
  }
})

// 路由跳转
const goHome = () => {
  router.push('/')
}

const goBack = () => {
  router.go(-1)
}
</script>
<template>
  <div class="not-found-container" @mousemove="handleMouseMove">
    <div class="content-wrapper">
      <!-- 动态 SVG 插图 -->
      <div class="illustration" :style="parallaxStyle">
        <svg viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg" class="brain-svg">
          <!-- 背景装饰圆圈 -->
          <circle cx="200" cy="150" r="120" fill="#f0f4f8" class="bg-circle" />

          <!-- 拟人化的大脑/云朵形状 -->
          <path
            d="M140,180 Q120,180 120,160 Q120,130 150,130 Q160,100 200,100 Q240,100 250,130 Q280,130 280,160 Q280,180 260,180 Z"
            fill="#e1e8ed"
            stroke="#cbd5e0"
            stroke-width="2"
          />

          <!-- 眼睛 (困惑的表情) -->
          <circle cx="170" cy="150" r="8" fill="#5a6b7c" class="eye left-eye" />
          <circle cx="230" cy="150" r="8" fill="#5a6b7c" class="eye right-eye" />

          <!-- 嘴巴 -->
          <path
            d="M180,170 Q190,165 200,170 Q210,175 220,170"
            fill="none"
            stroke="#5a6b7c"
            stroke-width="3"
            stroke-linecap="round"
          />

          <!-- 问号元素 (浮动动画) -->
          <text
            x="250"
            y="90"
            fill="#8da2fb"
            font-size="40"
            font-weight="bold"
            class="floating-symbol s1"
          >
            ?
          </text>
          <text
            x="120"
            y="110"
            fill="#a0aec0"
            font-size="30"
            font-weight="bold"
            class="floating-symbol s2"
          >
            404
          </text>
          <text
            x="270"
            y="190"
            fill="#8da2fb"
            font-size="24"
            font-weight="bold"
            class="floating-symbol s3"
          >
            null
          </text>

          <!-- 连接线 (断开的效果) -->
          <path d="M50,250 L100,200" stroke="#cbd5e0" stroke-width="2" stroke-dasharray="5,5" />
          <path d="M350,250 L300,200" stroke="#cbd5e0" stroke-width="2" stroke-dasharray="5,5" />
        </svg>
      </div>

      <!-- 文字内容 -->
      <div class="text-content">
        <h1 class="error-code">404</h1>
        <h2 class="error-message">Brain Overflow Exception</h2>
        <p class="description">
          看来你的请求导致了我们的脑容量溢出。<br />
          这个页面就像一个未定义的变量，找不到任何引用的对象。
        </p>

        <!-- 按钮组 -->
        <div class="actions">
          <button @click="goHome" class="btn-primary"><span class="icon">↩</span> 返回首页</button>
          <button @click="goBack" class="btn-secondary">上一步</button>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
/* 容器样式：全屏，柔和背景 */
.not-found-container {
  width: 100%;
  min-height: 100vh;
  display: flex;
  justify-content: center;
  align-items: center;
  background-color: #fcfcfc; /* 极淡的灰白色 */
  background-image: radial-gradient(#f1f5f9 1px, transparent 1px);
  background-size: 30px 30px; /* 细微的点阵背景 */
  color: #4a5568;
  overflow: hidden;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
}

.content-wrapper {
  text-align: center;
  z-index: 1;
  padding: 2rem;
  max-width: 600px;
}

.illustration {
  width: 100%;
  max-width: 400px;
  height: auto;
  margin: 0 auto 2rem;
  transition: transform 0.1s ease-out;
}

/* 动画定义 */
@keyframes float {
  0%,
  100% {
    transform: translateY(0);
  }
  50% {
    transform: translateY(-10px);
  }
}

@keyframes blink {
  0%,
  90%,
  100% {
    transform: scaleY(1);
  }
  95% {
    transform: scaleY(0.1);
  }
}

/* 应用动画 */
.floating-symbol {
  animation: float 3s ease-in-out infinite;
}

.s1 {
  animation-delay: 0s;
}
.s2 {
  animation-delay: 1s;
}
.s3 {
  animation-delay: 2s;
}

.eye {
  transform-origin: center;
  animation: blink 4s infinite;
}

/* 文字排版 */
.error-code {
  font-size: 1.5rem;
  font-weight: 600;
  color: #a0aec0; /* 浅灰色，低对比度 */
  margin-bottom: 0.5rem;
  letter-spacing: 2px;
}

.error-message {
  font-size: 2.5rem;
  font-weight: 700;
  color: #2d3748;
  margin-bottom: 1rem;
}

.description {
  font-size: 1.1rem;
  line-height: 1.6;
  color: #718096;
  margin-bottom: 2.5rem;
}

/* 按钮样式 */
.actions {
  display: flex;
  justify-content: center;
  gap: 1rem;
}

button {
  padding: 0.75rem 1.5rem;
  border-radius: 8px;
  font-size: 1rem;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
  display: flex;
  align-items: center;
  gap: 0.5rem;
}

.btn-primary {
  background-color: #ebf8ff;
  color: #3182ce;
  border: 1px solid transparent;
}

.btn-primary:hover {
  background-color: #bee3f8;
  transform: translateY(-2px);
}

.btn-secondary {
  background-color: transparent;
  color: #718096;
  border: 1px solid #e2e8f0;
}

.btn-secondary:hover {
  background-color: #f7fafc;
  color: #4a5568;
  border-color: #cbd5e0;
}

/* 响应式适配 */
@media (max-width: 600px) {
  .error-message {
    font-size: 2rem;
  }

  .illustration {
    max-width: 300px;
  }

  .actions {
    flex-direction: column;
    gap: 0.8rem;
  }

  button {
    width: 100%;
    justify-content: center;
  }
}
</style>
