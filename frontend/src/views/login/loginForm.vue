<template>
  <div class="loginForm">
    <div class="selectBox">
      <p>请选择登录方式</p>
      <el-select v-model="loginValue" placeholder="请选择" class="loginSelect">
        <el-option
          v-for="item in loginOptions"
          :key="item.value"
          :label="item.label"
          :value="item.value"
        >
        </el-option>
      </el-select>
    </div>

    <input 
      type="text" 
      placeholder="请输入账户" 
      v-model="loginInput" 
      @blur="validateAccount"
    />
    <input 
      type="password" 
      placeholder="请输入密码" 
      v-model="loginPass" 
      @blur="validatePassword"
    />
    <el-divider><i class="el-icon-mobile-phone"></i></el-divider>
    <button type="button" @click="handleLogin">Sign in</button>

    <a href="#" class="loginToRegis" @click.prevent="loginToRegis">
      not sign up yet?
    </a>
  </div>
</template>

<script>
import Vue from 'vue'
import { loginWithEmail, loginWithId, loginWithPhone } from '@/api/login'
import { Message } from 'element-ui'
import { Select, Option } from 'element-ui'
Vue.use(Select)
Vue.use(Option)
const { jwtDecode } = require('jwt-decode')

export default {
  data() {
    return {
      loginInput: '',
      loginPass: '',
      loginValue: '',
      loginOptions: [
        { value: 'ID', label: 'ID登录' },
        { value: 'Email', label: '邮箱登录' },
        { value: 'Phone', label: '手机登录' }
      ]
    }
  },
  methods: {
    validateAccount() {
      if (!this.loginInput.trim()) {
        Message({ type: 'warning', duration: 2000, message: '请输入账户' })
        return false
      }
      return true
    },
    validatePassword() {
      if (!this.loginPass.trim()) {
        Message({ type: 'warning', duration: 2000, message: '请输入密码' })
        return false
      }
      return true
    },
    validateLogin() {
      return this.validateAccount() && this.validatePassword()
    },
    async handleLogin() {
      if (!this.validateLogin()) return
      if (!this.loginValue) {
        Message({ type: 'warning', duration: 2000, message: '请选择登陆方式' })
        return
      }
      let res
      if (this.loginValue === 'ID') {
        res = await loginWithId(this.loginInput, this.loginPass)
      } else if (this.loginValue === 'Phone') {
        const cleanPhone = this.loginInput.replace(/\s+/g, '')
        res = await loginWithPhone(cleanPhone, this.loginPass)
      } else if (this.loginValue === 'Email') {
        res = await loginWithEmail(this.loginInput, this.loginPass)
      }
      // 如果后端返回的是对象，要确认token怎么拿，比如 res.token 或 res.data.token
      localStorage.setItem('token', res)
      this.$store.commit('setToken', res)
      Message({ type: 'success', duration: 2000, message: '恭喜你登陆成功' })
    },
    loginToRegis() {
      this.loginInput = ''
      this.loginPass = ''
      this.$emit('change-login')
    }
  }
}
</script>

<style scoped lang="less">
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

.selectBox {
  display: flex;
  width: 100%;
  height: 40px;
  line-height: 40px;
  margin-bottom: 10px;
  color: #000;

  .loginSelect {
    width: 200px;
    border: 1px solid #f1f1f1;
    border-radius: 20px;
    margin-left: 20px;
    height: 40px;
  }
}
</style>
