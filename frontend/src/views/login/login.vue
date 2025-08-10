<template>
  <div class="loginBox">
    <div class="left">
      <div class="loginLogo">这里是我们的logo</div>
      <div class="welcomeText">
        <h2>Welcome to BrainOverflow</h2>
        <button v-if="!isLogin" @click="handleChangeRegis">点击我切换注册模式</button>
      </div>

      <LoginForm v-if="isLogin" @change-login="handleChangeLogin" />
      <RegisterForm
        v-else
        :isRegisWithPhone="isRegisWithPhone"
        @change-login="handleChangeLogin"
      />
    </div>

    <div class="right">
      <img src="@/assets/login-test-pic.jpg" alt="图片加载错误" />
    </div>
  </div>
</template>

<script>
import LoginForm from '@/views/login/loginForm.vue'
import RegisterForm from '@/views/login/regisForm.vue'

export default {
  components: { LoginForm, RegisterForm },
  data() {
    return {
      isLogin: true,
      isRegisWithPhone: true
    }
  },
  methods: {
    handleChangeLogin(newVal) {
      
      this.isLogin = !this.isLogin
    },
    handleChangeRegis() {
      this.isRegisWithPhone = !this.isRegisWithPhone
    }
  }
}
</script>

<style scoped>
.loginBox {
  width: 900px;
  height: 675px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 10px;
  opacity: 0.8;
  backdrop-filter: blur(10px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin: 100px auto;
  display: flex;
  justify-content: space-around;
}
.left {
  position: relative;
  width: 400px;
  background-color: #fff;
  padding: 10px;
}
.left .loginLogo {
  margin-top: 100px;
  color: #000;
}
.left .welcomeText {
  width: 100%;
  font-size: 16px;
  color: #000;
  display: flex;
  justify-content: space-between;
  margin-bottom: 20px;
}
.left .welcomeText h2 {
  display: block;
  width: 140px;
}
.left .welcomeText button {
  display: block;
  border: none;
  background-color: #fff;
  font-size: 14px;
  font-weight: 600;
}
.right {
  width: 400px;
  padding: 10px;
}
.right img {
  margin-top: 50px;
  width: 400px;
}
</style>