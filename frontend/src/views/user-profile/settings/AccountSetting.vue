<script setup>
import { ref, computed, nextTick, markRaw } from 'vue'
import { useUserStore } from '@/stores'
import { useRouter } from 'vue-router'
import { changeUserVerification, deleteUserAccount } from '@/api/userVerify'
import { Warning } from '@element-plus/icons-vue'
defineOptions({ name: 'AccountSetting' })

const userStore = useUserStore()

const typeMap = {
  phone: { label: '手机号码', key: 'newPhone' },
  email: { label: '邮箱', key: 'newEmail' },
  password: { label: '密码', key: 'newPassword' },
}

const tableData = computed(() => [
  { name: '账户ID', current: userStore.userInfo.id ?? '', operation: 'copy', type: 'id' },
  {
    name: '手机号码',
    current: phoneNumberHash(userStore.userInfo.phone) ?? '未绑定',
    operation: userStore.userInfo.phone ? 'change' : 'bind',
    type: 'phone',
  },
  {
    name: '邮箱',
    current: emailHash(userStore.userInfo.email) ?? '未绑定',
    operation: userStore.userInfo.email ? 'change' : 'bind',
    type: 'email',
  },
  { name: '密码', current: '********', operation: 'change', type: 'password' },
  { name: '账号注销', current: '', operation: 'delete', type: 'delete' },
])

const isChangeDialogVisible = ref(false)
const changeType = ref('phone')
const changeFormRef = ref(null)

const changeFormData = ref({
  password: '', // 旧密码
  newPassword: '',
  newPhone: '',
  newEmail: '',
})

const rules = {
  password: [
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
  newPassword: [
    {
      validator: (rule, value, callback) => {
        if (!changeFormData.value.password) {
          return callback(new Error('请先输入原先的密码'))
        }
        if (!value) {
          return callback(new Error('请输入密码'))
        }
        if (changeFormData.value.password === value) {
          return callback(new Error('新旧密码不能一致'))
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

  newPhone: [
    { required: true, message: '请输入手机号码', trigger: 'blur' },
    {
      pattern: /^\+[1-9]\d{0,2}\s\d{4,14}$/,
      message: '手机格式不符合规范, 请输入正确的手机号码(E164格式)',
      trigger: 'blur',
    },
  ],
  newEmail: [
    { required: true, message: '请输入邮箱', trigger: 'blur' },
    {
      pattern: /^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$/,
      message: '邮箱格式不符合规范, 请输入正确的邮箱',
      trigger: 'blur',
    },
  ],
}

// compute current dialog title
const dialogTitle = computed(() => {
  const action =
    tableData.value.find((item) => item.type === changeType.value)?.operation === 'bind'
      ? '绑定'
      : '修改'
  const target = typeMap[changeType.value]?.label || ''
  return `${action}${target}`
})

// copy function
const handleCopy = (value) => {
  if (!value) return
  navigator.clipboard.writeText(value)
  ElMessage.success('已复制到剪贴板')
}

// open change dialog
const handleChange = (type) => {
  changeType.value = type
  isChangeDialogVisible.value = true
  // reset form when open dialog
  nextTick(() => {
    if (changeFormRef.value) changeFormRef.value.resetFields()
  })
}

// submit change
const submitChange = async () => {
  if (!changeFormRef.value) return

  const valid = await changeFormRef.value.validate()
  if (!valid) return

  const cleanedPhone = changeFormData.value.newPhone
    ? changeFormData.value.newPhone.replace(/\s+/g, '')
    : null

  await changeUserVerification(
    changeFormData.value.password,
    changeFormData.value.newPassword || null,
    changeFormData.value.newEmail || null,
    cleanedPhone || null,
  )

  if (changeType.value === 'phone') userStore.userInfo.phone = phoneNumberHash(cleanedPhone)
  if (changeType.value === 'email')
    userStore.userInfo.email = emailHash(changeFormData.value.newEmail)

  ElMessage.success('修改成功')
  isChangeDialogVisible.value = false
}

// delete account
const handleDelete = () => {
  ElMessageBox.prompt('请输入您的登录密码以删除账号，注意该操作不可撤销！', '删除账号', {
    confirmButtonText: '确认删除',
    cancelButtonText: '取消',
    confirmButtonType: 'warning',
    icon: markRaw(Warning),
  })
    .then(async ({ value }) => {
      if (!value) {
        ElMessage({
          type: 'error',
          message: '未输入密码，请重试',
        })
        return
      }
      await deleteUserAccount(value)
      ElMessage({
        type: 'success',
        message: '删除账号成功，即将跳转到主页',
      })
      const router = useRouter()
      userStore.logout()
      router.push('/')
    })
    .catch(() => {})
}

const phoneNumberHash = (phone) => {
  let arr = phone.split('')
  let length = phone.length
  if (length <= 11) throw TypeError('wrong phone number, too short')
  for (let i = 6; i < length - 2; i++) {
    arr[i] = '*'
  }
  return arr.join('')
}

const emailHash = (email) => {
  let index = email.indexOf('@')
  let arr = email.split('')
  if (index === -1) throw TypeError('wrong email format')
  for (let i = 2; i < index - 2; i++) {
    arr[i] = '*'
  }

  return arr.join('')
}
</script>

<template>
  <div class="user-setting-container">
    <el-card shadow="never" class="setting-card">
      <template #header>
        <div class="card-header">
          <span>账号安全设置</span>
        </div>
      </template>

      <el-table :data="tableData" :show-header="false" style="width: 100%">
        <el-table-column prop="name" width="140" class-name="label-col">
          <template #default="{ row }">
            <span class="text-gray-500">{{ row.name }}</span>
          </template>
        </el-table-column>

        <el-table-column prop="current">
          <template #default="{ row }">
            <span :class="{ 'text-gray-400': !row.current || row.current === '未绑定' }">
              {{ row.current }}
            </span>
          </template>
        </el-table-column>

        <el-table-column label="操作" width="120" align="right">
          <template #default="scope">
            <el-button
              v-if="scope.row.operation === 'copy'"
              link
              type="primary"
              @click="handleCopy(scope.row.current)"
            >
              复制
            </el-button>

            <el-button
              v-else-if="['change', 'bind'].includes(scope.row.operation)"
              link
              type="primary"
              @click="handleChange(scope.row.type)"
            >
              {{ scope.row.operation === 'bind' ? '立即绑定' : '修改' }}
            </el-button>

            <el-button
              v-else-if="scope.row.operation === 'delete'"
              link
              type="danger"
              @click="handleDelete()"
            >
              注销账号
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="isChangeDialogVisible" :title="dialogTitle" width="500px" destroy-on-close>
      <el-form
        label-width="100px"
        :model="changeFormData"
        :rules="rules"
        ref="changeFormRef"
        status-icon
      >
        <el-form-item label="当前密码" prop="password">
          <el-input
            v-model="changeFormData.password"
            type="password"
            show-password
            placeholder="请输入当前登录密码"
          ></el-input>
        </el-form-item>

        <el-form-item label="新手机号码" prop="newPhone" v-if="changeType === 'phone'">
          <el-input v-model="changeFormData.newPhone" placeholder="请输入新手机号"></el-input>
        </el-form-item>

        <el-form-item label="新邮箱" prop="newEmail" v-else-if="changeType === 'email'">
          <el-input v-model="changeFormData.newEmail" placeholder="请输入新邮箱地址"></el-input>
        </el-form-item>

        <el-form-item label="新密码" prop="newPassword" v-else-if="changeType === 'password'">
          <el-input
            v-model="changeFormData.newPassword"
            type="password"
            show-password
            placeholder="设置新的登录密码"
          ></el-input>
        </el-form-item>
      </el-form>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="isChangeDialogVisible = false">取消</el-button>
          <el-button type="primary" @click="submitChange"> 确认提交 </el-button>
        </div>
      </template>
    </el-dialog>
  </div>
</template>

<style scoped>
.setting-card {
  max-width: 900px;
  margin: 0 auto;
}
.text-gray-500 {
  color: #6b7280;
}
.text-gray-400 {
  color: #9ca3af;
  font-style: italic;
}
</style>
