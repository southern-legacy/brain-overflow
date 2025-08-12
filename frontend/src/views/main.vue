<template>
  <div class="main">
    <p>这里是首页</p>
    <input type="text" v-model="testPass"> 
    <button @click="handleDeleteTest">点我测试删除账号接口</button>
    <button @click="handleCheckMyTest">点我测试查看自己账户</button>
    <button @click="handleCheckOthersTest(1)">点我测试查看别人</button>
    <router-link to="/login">点击我前往登录</router-link>
  </div>
</template>

<script>
import { deleteAccount } from '@/api/login';
import { checkMyInfo, checkOthersInfo } from '@/api/info';
  export default {
      name: 'mainPage',
      data() {
        return {
          testPass: ''
        }
      },
      methods:{
        async handleDeleteTest () {
          const res = await deleteAccount(this.testPass)
          console.log(res)
          
        },
        async handleCheckMyTest () {
          const res = await checkMyInfo()
          console.log(res)
          
        },
        async handleCheckOthersTest (id) {
          const res = await checkOthersInfo(1)
          console.log(res)
        }

      }
  }
</script>

<style>
</style>