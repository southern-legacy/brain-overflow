<template>
  <div class="regisForm">
    <input type="text" placeholder="请输入名称" v-model="regisId" />
    <div class="codeBox">
      <input type="text" placeholder="请输入图形验证码" />
      <img src="@/assets/code.png" alt="验证码" />
    </div>
    <input
      type="text"
      placeholder="请输入手机号码"
      v-if="isRegisWithPhone"
      v-model="regisPhone"
    />
    <input
      type="text"
      placeholder="请输入邮箱"
      v-else
      v-model="regisEmail"
    />
    <div class="connectCode">
      <input type="text" placeholder="请输入验证码" />
      <button v-if="isRegisWithPhone" @click="validatePhone">
        点击获取手机验证码
      </button>
      <button v-else @click="validateEmail">
        点击获取邮箱验证码
      </button>
    </div>
    <input
      type="password"
      placeholder="请输入密码"
      v-model="regisPass"
    />
    <input
      type="password"
      placeholder="请确认您的密码"
      v-model="regisPassComfirm"
    />

    <el-divider v-if="isRegisWithPhone"><i class="el-icon-mobile-phone"></i></el-divider>
    <el-divider v-else><i class="el-icon-s-promotion"></i></el-divider>

    <button
      type="button"
      @click="regisWithPhone"
      v-if="isRegisWithPhone"
    >
      Sign up with Phone Number
    </button>
    <button type="button" @click="regisWithEmail" v-else>
      Sign up with Email
    </button>

    <a href="#" class="regisToLogin" @click.prevent="$emit('change-login')">
      already got an account?
    </a>
  </div>
</template>

<script>
import { regisAccountWithEmail, regisAccountWithPhone } from '@/api/login'
import { Message } from 'element-ui'
import jwtDecode from 'jwt-decode'

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
    validatePhone() {
      const cleanPhone = this.regisPhone.replace(/\s+/g, '')
      const regex = /^\+\d{1,15}$/
      if (!regex.test(cleanPhone)) {
        Message({
          message: '电话号码格式错误（必须为 E.164 格式，如 +8613800138000)',
          type: 'warning',
          duration: 2500
        })
        return
      }
      Message({
        message: '格式正确，正在获取短信验证码',
        type: 'success',
        duration: 1000
      })
      console.log('提交的号码：', cleanPhone)
    },
    validateEmail() {
      const emailReg = /^([A-Za-z0-9_\-\.])+\@([A-Za-z0-9_\-\.])+\.([A-Za-z]{2,4})$/
      if (!emailReg.test(this.regisEmail)) {
        Message({ message: '邮箱格式有误，请检查', type: 'warning' })
        return
      }
      Message({ message: '格式正确，正在获取邮箱验证码', type: 'success', duration: 1000 })
    },
    validatePass() {
      if (this.regisPass === '' || this.regisPassComfirm === '') {
        Message({ message: '请不要输入空的密码', type: 'warning', duration: 2500 })
        return false
      }
      if (this.regisPass !== this.regisPassComfirm) {
        Message({ message: '前后两次密码输入不一致', type: 'warning', duration: 2500 })
        return false
      }
      return true
    },
    async regisWithPhone() {
      if (!this.validatePass()) return
      const cleanPhone = this.regisPhone.replace(/\s+/g, '')
      const res = await regisAccountWithPhone(this.regisId, cleanPhone, this.regisPass)
      const resDecoded = jwtDecode(res)
      console.log(resDecoded)
      Message({ type: 'success', message: '注册成功，欢迎使用brain overflow！ ', duration: 2000 })
    },
    async regisWithEmail() {
      if (!this.validatePass()) return
      const res = await regisAccountWithEmail(this.regisId, this.regisEmail, this.regisPass)
      const resDecoded = jwtDecode(res)
      console.log(resDecoded)
      Message({ type: 'success', message: '注册成功，欢迎使用brain overflow！ ', duration: 2000 })
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