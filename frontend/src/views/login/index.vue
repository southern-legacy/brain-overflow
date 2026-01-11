<!--
  * Component: Login
  * Description:  the login page
-->

<script setup>
import LoginForm from '@/views/login/components/LoginForm.vue'
import RegisForm from '@/views/login/components/RegisForm.vue'
import { ref } from 'vue'

const isLogin = ref(true)
const showUuidDialog = ref(false)
const userUuid = ref('')

const handleChangingAuth = () => {
  isLogin.value = !isLogin.value
}

const copyUuid = async () => {
  await navigator.clipboard.writeText(userUuid.value)
  ElMessage.success('已复制到剪贴板')
}

const handleRegisSuccess = (id) => {
  userUuid.value = id
  showUuidDialog.value = true
}
</script>

<template>
  <el-row>
    <el-col :span="14" :offset="5">
      <div class="loginBox">
        <div class="left">
          <img src="@/assets/small_logo.png" alt="#" class="loginLogo" draggable="false" />
          <div class="welcomeText">
            <h2>Welcome to BrainOverflow</h2>
          </div>

          <LoginForm v-if="isLogin" @changingAuth="handleChangingAuth"></LoginForm>
          <RegisForm
            v-else
            @changingAuth="handleChangingAuth"
            @regisSuccess="handleRegisSuccess"
          ></RegisForm>
        </div>
        <div class="right">
          <img src="@/assets/login-test-pic.jpg" alt="图片加载错误" />
        </div>
      </div>
    </el-col>
  </el-row>

  <el-dialog v-model="showUuidDialog" title="注册成功 🎉" width="600px">
    <p style="margin-bottom: 8px">这是你的用户 ID（UUID），请妥善保存，可用于登录：</p>

    <el-input v-model="userUuid" readonly size="large">
      <template #append>
        <el-button @click="copyUuid">复制</el-button>
      </template>
    </el-input>

    <template #footer>
      <el-button type="primary" @click="showUuidDialog = false"> 我已保存 </el-button>
    </template>
  </el-dialog>
</template>

<style scoped lang="scss">
.loginBox {
  height: 750px;
  padding: 30px;
  background: #d5dfe6;
  border-radius: 20px;
  opacity: 0.8;
  backdrop-filter: blur(10px);
  box-shadow: 0px 10px 10px -2px #a3a0a5;
  margin-top: 100px;
  display: flex;
  justify-content: space-between;
}
.left {
  margin-left: 50px;
}
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
  margin-bottom: 10px;
  align-items: center;
}

.right {
  width: 45%;
}

.right img {
  width: 90%;
  border-radius: 16px;
  object-fit: cover;
  box-shadow: 0 10px 20px rgba(201, 92, 129, 0.2);
}
</style>
