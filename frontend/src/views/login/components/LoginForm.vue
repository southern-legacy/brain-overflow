<!--
  * Component: LoginForm
  * Description:  the login form components used for login page
-->

<script setup>
import { ref } from 'vue'
import { userLoginService } from '@/api/userLogin'
import { useUserStore } from '@/stores'

// stats
const formModel = ref({
  id: '',
  phone: '',
  email: '',
  password: '',
})

// form ref
const form = ref(null)

const selectLogin = ref('id')
const selectOptions = [
  { value: 'id', label: 'ID登录' },
  { value: 'phone', label: '手机号登录' },
  { value: 'email', label: '邮箱登录' },
]

// validate rules
const rules = {
  id: [
    { required: true, message: '请输入id', trigger: 'blur' },
    { pattern: /^\d+$/, message: '请输入正确格式的id', trigger: 'blur' },
  ],
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
}

/**
 * login logic
 */
const handleLogin = async () => {
  await form.value.validate()
  let token = ''
  try {
    if (selectLogin.value === 'id') {
      token = await userLoginService(+formModel.value.id, '', '', formModel.value.password)
    } else if (selectLogin.value === 'email') {
      token = await userLoginService('', formModel.value.email, '', formModel.value.password)
    } else if (selectLogin.value === 'phone') {
      token = await userLoginService(
        '',
        '',
        formModel.value.phone.replace(/\s/g, ''),
        formModel.value.password,
      )
    }
  } catch (err) {
    return err
  }
  ElMessage({
    type: 'success',
    message: '恭喜您，登录成功',
  })
  const userStore = useUserStore()
  userStore.setToken(token)
}

const emit = defineEmits(['changingAuth'])
const changeAuth = () => {
  emit('changingAuth')
}
</script>

<template>
  <div class="loginForm">
    <el-form :model="formModel" :rules="rules" ref="form">
      <!-- selector -->
      <el-form-item label="请选择登陆方式" class="selectBox">
        <el-select
          v-model="selectLogin"
          placeholder="请选择"
          class="loginSelect"
          popper-class="loginSelectDropdown"
        >
          <el-option
            v-for="item in selectOptions"
            :key="item.value"
            :label="item.label"
            :value="item.value"
          />
        </el-select>
      </el-form-item>

      <!-- username -->
      <el-form-item v-if="selectLogin === 'id'" prop="id">
        <el-input v-model="formModel.id" placeholder="请输入账户" class="username" />
      </el-form-item>

      <!-- phone number -->
      <el-form-item v-else-if="selectLogin === 'phone'" prop="phone">
        <el-input v-model="formModel.phone" placeholder="请输入手机号码" class="username" />
      </el-form-item>

      <!-- email -->
      <el-form-item v-else-if="selectLogin === 'email'" prop="email">
        <el-input v-model="formModel.email" placeholder="请输入邮箱" class="username" />
      </el-form-item>

      <!-- password -->
      <el-form-item prop="password">
        <el-input
          v-model="formModel.password"
          type="password"
          placeholder="请输入密码"
          class="password"
        />
      </el-form-item>

      <!-- login button -->
      <el-form-item>
        <el-button class="submitBtn" @click="handleLogin">登录</el-button>
      </el-form-item>
    </el-form>

    <!-- registration -->
    <div class="loginToRegis">
      <span>not sign up yet?</span>
      <a href="#" @click.prevent="changeAuth">Sign up here</a>
    </div>
  </div>
</template>

<style scoped lang="scss">
.loginForm {
  display: flex;
  flex-direction: column;
  background-color: transparent;
  padding: 6px 0;
  width: 100%;
}

:deep(.el-input__wrapper) {
  height: 42px;
  background-color: #fff;
  border: 1.5px solid #d9aeb8;
  border-radius: 10px;
  font-size: 15px;
  box-shadow: 0 2px 6px rgba(215, 175, 185, 0.1);
  transition: all 0.2s ease;
  padding: 0 14px;
}
:deep(.el-input__wrapper.is-focus) {
  border-color: #b4707f;
  box-shadow: 0 0 6px 3px rgba(180, 112, 127, 0.18);
}

/* text style */
:deep(.el-input__inner) {
  color: #5f4959;
  font-size: 15px;
}

/* select style */
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

/* dropdown style */
:deep(.loginSelectDropdown) {
  border-radius: 10px;
  box-shadow: 0 6px 16px rgba(180, 112, 127, 0.15);
  overflow: hidden;
}

:deep(.loginSelectDropdown .el-select-dropdown__item) {
  font-size: 15px;
  color: #5f4959;
  padding: 8px 14px;
}

.selectBox {
  display: flex;
  align-items: center;
  margin-bottom: 12px;
}

.selectBox .el-form-item__label {
  color: #4c3a4c;
  font-size: 15px;
  margin-right: 12px;
  min-width: max-content;
}

:deep(.el-select .el-input__wrapper) {
  height: 44px;
}

:deep(.el-select:hover .el-input__wrapper) {
  border-color: #b4707f;
}

/* login btn */
.submitBtn {
  height: 46px;
  width: 100%;
  border-radius: 24px;
  background: linear-gradient(135deg, #f2d6d9, #c9a5aa);
  border: none;
  color: #6e4a56;
  font-weight: 700;
  font-size: 16px;
  letter-spacing: 0.5px;
  box-shadow: 0 4px 10px rgba(201, 165, 170, 0.25);
  transition: all 0.3s ease;
  cursor: pointer;
  margin-top: 8px;
}

.submitBtn:hover {
  background: linear-gradient(135deg, #f8dce0, #b88090);
  box-shadow: 0 8px 18px rgba(184, 144, 149, 0.3);
  transform: translateY(-2px);
}

.submitBtn:active {
  transform: translateY(1px);
}

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
