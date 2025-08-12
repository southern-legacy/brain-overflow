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
    <button type="button" @click="handleLogin" class="submitBtn">Sign in</button>

    <div class="loginToRegis">
      <span>not sign up yet?</span>
      <a href="#"  @click.prevent="loginToRegis">
        Sign up here
      </a>
    </div>
    
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
      loginValue: 'ID',
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
        // 
        res = await loginWithId(+this.loginInput, this.loginPass)
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
  background-color: transparent; /* 容器背景已是 #D5DFE6 */
  padding: 6px 0;
  width: 100%;
}

/* 输入框风格 */
.loginForm input {
  height: 42px;
  background-color: #fff;
  border: 1.5px solid #d9aeb8;
  color: #5f4959;
  border-radius: 10px;
  font-size: 15px;
  box-shadow: 0 2px 6px rgba(215, 175, 185, 0.1);
  transition: all 0.2s ease;
  margin-bottom: 12px;
  padding: 0 14px;
}

/* focus 样式 */
.loginForm input:focus {
  border-color: #b4707f;
  box-shadow: 0 0 6px 3px rgba(180, 112, 127, 0.18);
  outline: none;
}

/* 选择器容器 */
.selectBox {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}
.selectBox p {
  color: #4c3a4c;
  font-size: 15px;
  margin-right: 12px;
  min-width: max-content;
}

/* 覆盖 Element UI 的输入样式 */
::v-deep .el-select .el-input__inner {
  background: #fff;
  border: 1.5px solid #d9aeb8;
  color: #5f4959;
  border-radius: 10px;
  height: 44px;
}
::v-deep .el-select .el-input__inner:hover {
  border-color: #b4707f;
}

.submitBtn {
  height: 46px;
  border-radius: 24px;
  background: linear-gradient(135deg, #f2d6d9, #c9a5aa); /* 颜色更柔和，饱和度降低 */
  border: none;
  color: #6e4a56; /* 文字颜色变柔和暗红 */
  font-weight: 700;
  font-size: 16px;
  letter-spacing: 0.5px;
  box-shadow: 0 4px 10px rgba(201, 165, 170, 0.25); /* 阴影透明度更低 */
  transition: all 0.3s ease;
  cursor: pointer;
  margin-top: 8px;
}
.submitBtn:hover {
  background: linear-gradient(135deg, #e8c7cc, #b89095); /* 悬停色调稍微加深 */
  box-shadow: 0 8px 18px rgba(184, 144, 149, 0.3);
  transform: translateY(-2px);
}
.submitBtn:active {
  transform: translateY(1px);
}

/* 注册提示 */
.loginToRegis {
  margin-top: 14px;
  font-size: 14px;
  text-align: center;
  color: #4c3a4c;
}
.loginToRegis span {
  margin-right: 6px;
}
.loginToRegis a {
  color: #a73758;
  text-decoration: none;
  font-weight: 600;
  cursor: pointer;
}
.loginToRegis a:hover {
  text-decoration: underline;
}
</style>
