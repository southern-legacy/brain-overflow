<template>
  <div class="regisForm">
    <input
      type="text"
      placeholder="请输入名称"
      v-model="regisId"
    />

    <div class="codeBox">
      <input type="text" placeholder="请输入图形验证码" />
      <img
        src="@/assets/code.png"
        alt="验证码"
        @click="onCaptchaClick"
      />
    </div>

    <input
      type="text"
      placeholder="请输入手机号码"
      v-if="isRegisWithPhone"
      v-model="regisPhone"
      @blur="validatePhone"
    />
    <input
      type="text"
      placeholder="请输入邮箱"
      v-else
      v-model="regisEmail"
      @blur="validateEmail"
    />

    <div class="connectCode">
      <input type="text" placeholder="请输入验证码" />
      <button
        v-if="isRegisWithPhone"
        @click="onGetPhoneCode"
      >
        点击获取手机验证码
      </button>
      <button
        v-else
        @click="onGetEmailCode"
      >
        点击获取邮箱验证码
      </button>
    </div>

    <input
      type="password"
      placeholder="请输入密码"
      v-model="regisPass"
      @blur="validatePassword"
    />
    <input
      type="password"
      placeholder="请确认您的密码"
      v-model="regisPassComfirm"
      @blur="validatePasswordConfirm"
    />

    <el-divider v-if="isRegisWithPhone">
      <i class="el-icon-mobile-phone"></i>
    </el-divider>
    <el-divider v-else>
      <i class="el-icon-s-promotion"></i>
    </el-divider>

    <button
      type="button"
      @click="regisWithPhone"
      v-if="isRegisWithPhone"
    >
      Sign up with Phone Number
    </button>
    <button
      type="button"
      @click="regisWithEmail"
      v-else
    >
      Sign up with Email
    </button>

    <a
      href="#"
      class="regisToLogin"
      @click.prevent="$emit('change-login')"
    >
      already got an account?
    </a>
  </div>
</template>

<script>
import { regisAccountWithEmail, regisAccountWithPhone } from '@/api/login'
import { Message } from 'element-ui'
const { jwtDecode } = require('jwt-decode')

export default {
  props: ['isRegisWithPhone'],
  data() {
    return {
      regisPhone: '',
      regisEmail: '',
      regisId: '',
      regisPass: '',
      regisPassComfirm: ''
    }
  },
  methods: {
    // 点击验证码图片
    onCaptchaClick() {
      if (this.isRegisWithPhone) {
        this.validatePhone()
      } else {
        this.validateEmail()
      }
      // 这里可以加刷新验证码图片的逻辑
    },

    // 点击获取验证码按钮
    onGetPhoneCode() {
      if (this.validatePhone()) {
        Message({
          message: '格式正确，正在获取短信验证码',
          type: 'success',
          duration: 1000
        })
        // 调用接口获取短信验证码
      }
    },
    onGetEmailCode() {
      if (this.validateEmail()) {
        Message({
          message: '格式正确，正在获取邮箱验证码',
          type: 'success',
          duration: 1000
        })
        // 调用接口获取邮箱验证码
      }
    },

    // 手机号验证（blur触发 + 按钮触发）
    validatePhone() {
      const cleanPhone = this.regisPhone.replace(/\s+/g, '')
      const regex = /^\+[1-9]\d{1,14}$/ // E.164
      if (!regex.test(cleanPhone)) {
        Message({
          message: '电话号码格式错误（必须为 E.164 格式，如 +8613800138000)',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      return true
    },

    // 邮箱验证（blur触发 + 按钮触发）
    validateEmail() {
      const emailReg = /^[^\s@]+@[^\s@]+\.[^\s@]{2,}$/
      if (!emailReg.test(this.regisEmail)) {
        Message({
          message: '邮箱格式有误，请检查',
          type: 'warning'
        })
        return false
      }
      return true
    },

    // 密码校验：检查密码非空且长度 >= 6
    validatePassword() {
      if (!this.regisPass || this.regisPass.trim() === '') {
        Message({
          message: '请输入密码',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      if (this.regisPass.length < 6) {
        Message({
          message: '密码长度至少6位',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      return true
    },

    // 确认密码校验：非空且和密码一致
    validatePasswordConfirm() {
      if (!this.regisPassComfirm || this.regisPassComfirm.trim() === '') {
        Message({
          message: '请确认密码',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      if (this.regisPass !== this.regisPassComfirm) {
        Message({
          message: '前后两次密码输入不一致',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      return true
    },

    async regisWithPhone() {
      if (!this.validatePhone() || !this.validatePassword() || !this.validatePasswordConfirm()) return
      const cleanPhone = this.regisPhone.replace(/\s+/g, '')
      const res = await regisAccountWithPhone(
        this.regisId,
        cleanPhone,
        this.regisPass
      )
      const resDecoded = jwtDecode(res)
      localStorage.setItem('token', res)
      this.$store.commit('setToken', res)
      Message({
        type: 'success',
        message: `注册成功，欢迎使用brain overflow！ 您的id为：${resDecoded.id}`,
        duration: 0,
        showClose: true
      })
      this.$emit('change-login', resDecoded.id)
    },

    async regisWithEmail() {
      if (!this.validateEmail() || !this.validatePassword() || !this.validatePasswordConfirm()) return
      const res = await regisAccountWithEmail(
        this.regisId,
        this.regisEmail,
        this.regisPass
      )
      const resDecoded = jwtDecode(res)
      localStorage.setItem('token', res)
      this.$store.commit('setToken', res)
      Message({
        type: 'success',
        message: `注册成功，欢迎使用brain overflow！ 您的id为：${resDecoded.id}`,
        duration: 0,
        showClose: true
      })
      this.$emit('change-login', resDecoded.id)
    }
  }
}
</script>

<style scoped>
.regisForm {
  display: flex;
  flex-direction: column;
}
.codeBox,
.connectCode {
  display: flex;
  justify-content: space-between;
}
.codeBox input,
.connectCode input {
  width: 200px;
}
.codeBox img,
.connectCode img {
  width: 150px;
  height: 45px;
  cursor: pointer;
}
.regisForm input,
.regisForm button {
  height: 40px;
  border-radius: 20px;
  margin-bottom: 10px;
  padding: 0 15px;
}
.el-divider i {
  font-size: 20px;
}
.regisToLogin {
  color: #000;
  text-decoration: none;
  margin-top: 10px;
  align-self: flex-end;
}
</style>
