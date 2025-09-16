<script setup>
import LoginForm from '@/views/login/LoginForm.vue'
import RegisForm from '@/views/login/RegisForm.vue'
import { ref } from 'vue'

const isLogin = ref(true)
const handleChangingAuth = () => {
  isLogin.value = !isLogin.value
}
</script>

<template>
  <el-row>
    <el-col :span="14" :offset="5">
      <div class="loginBox">
        <div class="left">
          <img src="@/assets/small_logo.png" alt="#" class="loginLogo" draggable="false" />
          <div class="welcomeText">
            <h2>Welcome to BrainOverflow</h2>
          </div>

          <LoginForm v-if="isLogin" @changingAuth="handleChangingAuth"></LoginForm>
          <RegisForm v-else @changingAuth="handleChangingAuth"></RegisForm>
        </div>
        <div class="right">
          <img src="@/assets/login-test-pic.jpg" alt="图片加载错误" />
        </div>
      </div>
    </el-col>
  </el-row>
</template>

<style scoped lang="scss">
.loginBox {
  height: 700px;
  padding: 30px;
  background: #d5dfe6;
  border-radius: 20px;
  opacity: 0.8;
  backdrop-filter: blur(10px);
  box-shadow: 0px 10px 10px -2px #a3a0a5;
  margin-top: 50px;
  display: flex;
  justify-content: space-between;
}
.left {
  margin-left: 50px;
}
.left .loginLogo {
  width: 100px;
  height: 100px;
}

.left .welcomeText {
  width: 100%;
  font-size: 18px;
  color: #4c3a4c;
  font-weight: 600;

  display: flex;
  justify-content: space-between;
  margin-bottom: 10px;
  align-items: center;
}

.right {
  width: 45%;
}

.right img {
  margin-top: 40px;
  width: 90%;
  border-radius: 16px;
  object-fit: cover;
  box-shadow: 0 10px 20px rgba(201, 92, 129, 0.2);
}
</style>
