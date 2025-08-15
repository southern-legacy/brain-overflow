<template>
  <div class="loginBox">
    <div class="left">
      <img src="@/assets/small_logo.png" alt="#" class="loginLogo" draggable="false" @click="jumpToGitHub">
      <div class="welcomeText">
        <h2>Welcome to BrainOverflow</h2>
        <div>
          <button v-if="!isLogin" @click="handleChangeRegis">切换注册模式</button>
          <h4 v-if="!isLogin">当前为 {{ isRegisWithPhone ? '手机注册' : '邮箱注册' }}</h4>
        </div>
        
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
    },
    jumpToGitHub () {
      window.open("https://github.com/southern-legacy/brain-overflow")
    }
  }
}
</script>

<style scoped>
.loginBox {
  width: 900px;
  height: 675px;
  background: #D5DFE6;
  border-radius: 20px;
  opacity: 0.8;
  backdrop-filter: blur(10px);
  box-shadow: 0px 10px 10px -2px #A3A0A5;
  margin: 100px auto;
  display: flex;
  justify-content: space-around;
}
.left {
  position: relative;
  width: 400px;
  background-color: #D5DFE6;
  padding: 10px;
}

/* 优化后的内部文字和按钮颜色 */
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
  margin-bottom: 30px;
  align-items: center;

  h4{
    font-size: 14px;
    text-align: center;
  }
}

.left .welcomeText h2 {
  margin: 0;
  width: 70%;
}

.left .welcomeText button {
  border: none;
  margin-bottom: 5px;
  background-color: #f9dce0;
  color: #a73758;
  border-radius: 14px;
  padding: 8px 16px;
  font-weight: 700;
  box-shadow: 0 3px 8px rgba(167, 55, 88, 0.25);
  cursor: pointer;
  transition: all 0.3s ease;
}

.left .welcomeText button:hover {
  background-color: #f7c7d0;
  box-shadow: 0 6px 14px rgba(167, 55, 88, 0.4);
  transform: translateY(-2px);
}

.right {
  width: 400px;
  padding: 10px;
}

.right img {
  margin-top: 50px;
  width: 400px;
  border-radius: 16px;
  object-fit: cover;
  box-shadow: 0 10px 20px rgba(201, 92, 129, 0.2);
}
</style>