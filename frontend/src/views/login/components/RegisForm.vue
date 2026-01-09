<script setup>
import { ref } from 'vue'
import { userRegisService } from '@/api/userVerify'
import { jwtDecode } from 'jwt-decode'
import { ElMessage } from 'element-plus'

// form ref
const form = ref(null)

// regis stats
const formModel = ref({
  regisType: 'email',
  name: '',
  email: '',
  phone: '',
  imgCode: '',
  verifyCode: '',
  password: '',
  rePassword: '',
})

// default regis stats
const defaultForm = {
  regisType: 'email',
  email: '',
  phone: '',
  imgCode: '',
  verifyCode: '',
  password: '',
  rePassword: '',
}

const emit = defineEmits(['changingAuth', 'regisSuccess'])
const changeAuth = () => {
  formModel.value = defaultForm
  emit('changingAuth')
}

const rules = ref({
  name: [{ required: true, message: '请输入用户名', trigger: 'blur' }],
  email: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    {
      pattern: /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/,
      message: '邮箱格式不符合规范, 请输入正确的邮箱',
      trigger: 'blur',
    },
  ],
  phone: [
    { required: true, message: '请输入手机号码', trigger: 'blur' },
    {
      pattern: /^\+[1-9]\d{0,2}\s\d{4,14}$/,
      message: '手机格式不符合规范, 请输入正确的手机号码(E164格式)',
      trigger: 'blur',
    },
  ],
  password: [
    { required: true, message: '请输入密码', trigger: 'blur' },
    {
      validator: (rule, value, callback) => {
        if (!value) {
          return callback(new Error('请输入密码'))
        }
        if (value.length < 8) {
          return callback(new Error('密码长度必须大于8位'))
        }

        let hasNumber = /\d/.test(value) // 数字
        let hasAlpha = /[a-zA-Z]/.test(value) // 字母
        let hasSpecial = /[^a-zA-Z0-9]/.test(value) // 特殊字符（包括中文）

        let typeCount = [hasNumber, hasAlpha, hasSpecial].filter(Boolean).length

        if (typeCount < 2) {
          return callback(new Error('密码必须包含数字、字母、特殊字符中的至少两种'))
        }

        return callback() // 验证通过
      },
      trigger: 'blur',
    },
  ],
  rePassword: [
    { required: true, message: '请再次输入密码', trigger: 'blur' },
    {
      validator: (rule, value, callback) => {
        if (!value) {
          return callback(new Error('请再次输入密码'))
        }
        if (value !== formModel.value.password) return callback(new Error('前后两次密码输入不一致'))
        return callback()
      },
      trigger: 'blur',
    },
  ],
})
const handleRegis = async () => {
  await form.value.validate()
  let res = null
  try {
    if (formModel.value.regisType === 'phone') {
      res = await userRegisService(
        formModel.value.name,
        '',
        formModel.value.phone.replace(/\s/g, ''),
        formModel.value.password,
      )
      console.log(res)
    } else {
      res = await userRegisService(
        formModel.value.name,
        formModel.value.email,
        '',
        formModel.value.password,
      )
      console.log(res)
    }
  } catch (err) {
    if (err.status === 422 && err.code === 'unique') {
      return ElMessage({
        type: 'error',
        message: '当前账号已被人注册，请修改你的用户名或注册方式',
      })
    }
    return ElMessage({
      type: 'error',
      message: '注册错误',
    })
  }
  emit('regisSuccess', res.id)
  emit('changingAuth')
}
</script>

<template>
  <div class="regisForm">
    <el-form :model="formModel" ref="form" :rules="rules">
      <!-- 注册方式选择 -->
      <el-form-item label="请选择注册方式" class="selectBox" prop="regisType">
        <el-select
          placeholder="请选择"
          class="loginSelect"
          popper-class="loginSelectDropdown"
          v-model="formModel.regisType"
        >
          <el-option label="邮箱注册" value="email"></el-option>
          <el-option label="手机注册" value="phone"></el-option>
        </el-select>
      </el-form-item>
      <el-form-item prop="name">
        <el-input placeholder="请输入用户名" v-model="formModel.name"></el-input>
      </el-form-item>
      <!-- 邮箱/手机号 -->
      <el-form-item v-if="formModel.regisType === 'email'" prop="email">
        <el-input
          placeholder="请输入您的邮箱"
          class="username"
          v-model="formModel.email"
        ></el-input>
      </el-form-item>
      <el-form-item v-else prop="phone">
        <el-input
          placeholder="请输入您的手机号码"
          class="username"
          v-model="formModel.phone"
        ></el-input>
      </el-form-item>

      <!-- 图形验证码 -->
      <el-form-item prop="imgCode">
        <el-input
          placeholder="请输入验证码"
          style="width: 60%"
          class="verifyInput"
          v-model="formModel.imgCode"
        ></el-input>
        <img src="@/assets/code.png" alt="#" class="imgCode" loading="lazy" />
      </el-form-item>

      <!-- 短信/邮箱验证码 -->
      <el-form-item prop="verifyCode">
        <el-input
          placeholder="请输入验证码"
          class="verifyInput"
          style="width: 60%"
          v-model="formModel.verifyCode"
        ></el-input>
        <el-button class="codeBtn">点击获取验证码</el-button>
      </el-form-item>

      <!-- 密码 -->
      <el-form-item prop="password">
        <el-input
          type="password"
          placeholder="请输入密码"
          class="password"
          v-model="formModel.password"
        ></el-input>
      </el-form-item>

      <!-- 确认密码 -->
      <el-form-item prop="rePassword">
        <el-input
          type="password"
          placeholder="请再次输入密码"
          class="password"
          v-model="formModel.rePassword"
        ></el-input>
      </el-form-item>

      <!-- 注册按钮 -->
      <el-form-item>
        <el-button class="submitBtn" @click="handleRegis">注册</el-button>
      </el-form-item>
    </el-form>

    <!-- 登录提示 -->
    <div class="regisToLogin">
      <span>already got an account?</span>
      <a href="#" @click.prevent="changeAuth">Log in here</a>
    </div>
  </div>
</template>

<style scoped lang="scss">
.regisForm {
  display: flex;
  flex-direction: column;
  background-color: transparent;
  padding: 6px 0;
  width: 100%;
}

:deep(.el-form-item) {
  margin-bottom: 16px;
}

:deep(.el-form-item__label) {
  line-height: 42px;
}

/* 输入框基础样式 */
:deep(.el-input__wrapper) {
  height: 42px;
  background-color: #fff;
  border: 1.5px solid #d9aeb8;
  border-radius: 10px;
  font-size: 15px;
  box-shadow: 0 2px 6px rgba(215, 175, 185, 0.1);
  transition: all 0.3s ease;
}

:deep(.el-input__wrapper:hover) {
  border-color: #c97c92;
  box-shadow: 0 4px 10px rgba(215, 175, 185, 0.2);
}

:deep(.el-input__wrapper.is-focus) {
  border-color: #a73758;
  box-shadow: 0 4px 12px rgba(215, 175, 185, 0.3);
}

/* select 输入框样式 */
:deep(.el-select__wrapper) {
  height: 42px;
  background: #fff;
  border: 1.5px solid #d9aeb8;
  border-radius: 10px;
  transition: all 0.2s ease;
  padding: 0 14px;
  color: #5f4959;
  box-shadow: 0 2px 6px rgba(215, 175, 185, 0.1);
}
:deep(.el-select__wrapper.is-focused) {
  border-color: #b4707f;
  background-color: #fff8fa;
  box-shadow: 0 0 6px 3px rgba(180, 112, 127, 0.18);
}
:deep(.el-select__placeholder) {
  color: #5f4959;
  font-size: 15px;
}
:deep(.el-select__input) {
  color: #5f4959;
  font-size: 15px;
}

/* 验证码图片 */
.imgCode {
  width: auto;
  height: 42px;
  margin-left: 7px;
  border-radius: 8px;
  border: 1px solid #d9aeb8;
  box-shadow: 0 2px 6px rgba(215, 175, 185, 0.15);
}

/* 获取验证码按钮 */
.codeBtn {
  margin-left: 9px;
  height: 42px;
  border-radius: 20px;
  background: linear-gradient(135deg, #f7c8d1, #d797a3);
  border: none;
  color: #520d27;
  font-weight: 600;
  padding: 0 14px;
  transition: all 0.3s ease;
}
.codeBtn:hover {
  background: linear-gradient(135deg, #f5b6c0, #c78391);
}

/* 注册按钮 */
.submitBtn {
  height: 46px;
  border-radius: 24px;
  background: linear-gradient(135deg, #f7c8d1, #d797a3);
  border: none;
  color: #520d27;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: 0.5px;
  box-shadow: 0 6px 14px rgba(215, 175, 185, 0.3);
  transition: all 0.3s ease;
  width: 100%;
}
.submitBtn:hover {
  background: linear-gradient(135deg, #f5b6c0, #c78391);
  transform: translateY(-1px);
  box-shadow: 0 8px 18px rgba(215, 175, 185, 0.4);
}

/* 登录提示 */
.regisToLogin {
  font-size: 14px;
  text-align: center;
  color: #4c3a4c;
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
</style>
