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
        class="connectCodeButton"
      >
        获取手机验证码
      </button>
      <button
        v-else
        @click="onGetEmailCode"
        class="connectCodeButton"
      >
        获取邮箱验证码
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
    <div class="regisToLogin">
       <span> already got an account?</span>
       <a
         href="#"
      
         @click.prevent="$emit('change-login')"
        >
        Log in here
      </a>
    </div>
   
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

    // 密码校验：检查密码非空且长度 >= 12
    validatePassword() {
      let numbers = 0
      let letters = 0
      let specials = 0
      if (!this.regisPass || this.regisPass.trim() === '') {
        Message({
          message: '请输入密码',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      if (this.regisPass.length < 12) {
        Message({
          message: '密码长度至少12位',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      let passStringArray = this.regisPass.split("")
      passStringArray.forEach((char)=>{
          const charCode = char.charCodeAt(0);
  
          // 判断数字 (Unicode 48-57)
          if (charCode >= 48 && charCode <= 57) {
            numbers++
            return
          }
          
          // 判断字母 (Unicode 65-90 大写, 97-122 小写)
          if ((charCode >= 65 && charCode <= 90) || 
              (charCode >= 97 && charCode <= 122)) {
            letters++
            return
          }
          
          // 判断汉字 (Unicode 19968-40959 常用汉字范围)
          if (charCode >= 19968 && charCode <= 40959) {
            specials++
            return
          }
          
          specials++
          return
      })
      if([numbers, specials, letters].filter(n => n > 0).length >= 2)
      return true
      else{
        Message({
          message: '密码必须包含：数字，字母，特殊字符中的两种及以上',
          type: 'warning',
          duration: 2500
        })
        return false
      }

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
      console.log(resDecoded)
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
      console.log(resDecoded)
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

<style scoped lang="less">
.regisForm {
  display: flex;
  flex-direction: column;
  background-color: transparent; /* 容器背景已是 #D5DFE6 */
  padding: 6px 0;
  width: 100%;
}
.codeBox,
.connectCode {
  display: flex;
  justify-content: space-between;
}
.codeBox input,
.connectCode input {
  width: 200px;
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
  &:focus{
     border-color: #b4707f;
     box-shadow: 0 0 6px 3px rgba(180, 112, 127, 0.18);
     outline: none;
  }
}
.connectCode .connectCodeButton{
  margin-top: 0;
  margin-bottom: 12px;
  font-size: 14px;
  width: 150px;
}
.codeBox img,
.connectCode img {
  width: 150px;
  height: 45px;
  cursor: pointer;
}
.regisForm input {
  
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
  &:focus{
     border-color: #b4707f;
     box-shadow: 0 0 6px 3px rgba(180, 112, 127, 0.18);
     outline: none;
  }
}
.regisForm button {
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
.regisForm button:hover {
  background: linear-gradient(135deg, #e8c7cc, #b89095); /* 悬停色调稍微加深 */
  box-shadow: 0 8px 18px rgba(184, 144, 149, 0.3);
  transform: translateY(-2px);
}
.regisForm button:active {
  transform: translateY(1px);
}
.el-divider i {
  font-size: 20px;
}
.regisToLogin span {
  margin-right: 6px;
}
.regisToLogin a {
  color: #a73758;
  text-decoration: none;
  font-weight: 600;
  cursor: pointer;
}
.regisToLogin a:hover {
  text-decoration: underline;
}

.regisToLogin {
  margin-top: 14px;
  font-size: 14px;
  text-align: center;
  color: #4c3a4c;
}
</style>
