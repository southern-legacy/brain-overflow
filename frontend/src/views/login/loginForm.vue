<template>
  <div class="loginForm">
    <input type="text" placeholder="请输入账户" v-model="loginInput" />
    <input type="password" placeholder="请输入密码" v-model="loginPass" />
    <button type="button" @click="handleLogin">Sign in</button>
    <el-divider><i class="el-icon-mobile-phone"></i></el-divider>
    <button type="button" @click="handleLoginPhone">Sign in with Phone Number</button>
    <el-divider><i class="el-icon-s-promotion"></i></el-divider>
    <button type="button" @click="handleLoginEmail">Sign up with Email</button>
    <a href="#" class="loginToRegis" @click.prevent="$emit('change-login')">
      not sign up yet?
    </a>
  </div>
</template>

<script>
import { loginWithEmail, loginWithId, loginWithPhone } from '@/api/login'
import { Message } from 'element-ui'

export default {
  data() {
    return {
      loginInput: '',
      loginPass: ''
    }
  },
  methods: {
    validateLogin() {
      if (this.loginInput === '' || this.loginPass === '') {
        Message({
          type: 'warning',
          duration: 2000,
          message: '请输入账户和密码'
        })
        return false
      }
      return true
    },
    async handleLogin() {
      if (!this.validateLogin()) return
      const res = await loginWithId(this.loginInput, this.loginPass)
      console.log(res)
      Message({ type: 'success', duration: 2000, message: '恭喜你登陆成功' })
    },
    async handleLoginPhone() {
      if (!this.validateLogin()) return
      const cleanPhone = this.loginInput.replace(/\s+/g, '')
      const res = await loginWithPhone(cleanPhone, this.loginPass)
      console.log(res)
      Message({ type: 'success', duration: 2000, message: '恭喜你登陆成功' })
    },
    async handleLoginEmail() {
      if (!this.validateLogin()) return
      const res = await loginWithEmail(this.loginInput, this.loginPass)
      console.log(res)
      Message({ type: 'success', duration: 2000, message: '恭喜你登陆成功' })
    }
  }
}
</script>

<style scoped>
.loginForm {
  display: flex;
  flex-direction: column;
}
.loginForm input {
  height: 40px;
  border-radius: 20px;
  margin-bottom: 10px;
  padding: 0 15px;
}
.loginForm button {
  height: 40px;
  border-radius: 20px;
  margin-bottom: 10px;
}
.loginToRegis {
  color: #000;
  text-decoration: none;
  margin-top: 10px;
  align-self: flex-end;
}
</style>