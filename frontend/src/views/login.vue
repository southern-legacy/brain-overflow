<template>
  <div class="loginBox">
    <div class="left">
      <div class="loginLogo">这里是我们的logo</div>
      <div class="welcomeText">
        <h2>Welcome to BrainOverflow</h2>
        <button v-if="!isLogin" @click="handleChangeRegis">点击我切换注册模式</button>
      </div>

      <!-- 登录状态 -->
      <template v-if="isLogin">
        <div class="loginForm">
          <input type="text" placeholder="请输入账户"  v-model="loginInput"/>
          <input type="password" placeholder="请输入密码" v-model="loginPass"/>
          <button type="button" @click="handleLogin">Sign in</button>
          <el-divider><i class="el-icon-mobile-phone"></i></el-divider>
          <button type="button" @click="handleLoginPhone">Sign in with Phone Number</button>
          <el-divider ><i class="el-icon-s-promotion"></i></el-divider>
          <button type="button" @click="handleLoginEmail">Sign up with Email</button>
        </div>
        <a href="#" class="loginToRegis" @click.prevent="handleChangeLogin">
          not sign up yet?
        </a>
      </template>

      <!-- 注册状态 -->
      <template v-else>
        <div class="regisForm">
          <input type="text" placeholder="请输入账户" v-model="regisId"/>
          <div class="codeBox">
            <input type="text" placeholder="请输入图形验证码">
            <img src="@/assets/code.png" alt="#">
          </div>
          <input type="text" placeholder="请输入手机号码"  v-if="isRegisWithPhone" v-model="regisPhone"/>
          <input type="text" placeholder="请输入邮箱" v-else v-model="regisEmail"/>
          <div class="connectCode">
            <input type="text" placeholder="请输入验证码">
            <button v-if="isRegisWithPhone" @click="validatePhone">点击获取手机验证码</button>
            <button v-else @click="validateEmail">点击获取邮箱验证码</button>
          </div>
          <input type="password" placeholder="请输入密码" v-model="regisPass"/>
          <input type="password" placeholder="请确认您的密码" v-model="regisPassComfirm"/>
         
          <el-divider v-if="isRegisWithPhone"><i class="el-icon-mobile-phone"></i></el-divider>
          <el-divider v-else><i class="el-icon-s-promotion"></i></el-divider>
          <button type="button" @click="regisWithPhone" v-if="isRegisWithPhone">Sign up with Phone Number</button>
          <button type="button" @click="regisWithEmail" v-else>Sign up with Email</button>
        </div>
        <a href="#" class="regisToLogin" @click.prevent="handleChangeLogin">
          already got an account?
        </a>
      </template>
    </div>

    <div class="right">
      <img src="../assets/login-test-pic.jpg" alt="图片加载错误" />
    </div>
  </div>
</template>

<script>
import { regisAccountWithEmail,regisAccountWithPhone,loginWithEmail,loginWithId,loginWithPhone } from '@/api/login'
import { Message } from 'element-ui'
export default {
  data() {
    return {
      isLogin: true, // 是否处于登录状态： 用于登录界面和注册界面的切换
      isRegisWithPhone: true, // 是否处于手机注册模式, 用于邮箱注册和手机注册的切换
      regisPhone: '', // 该变量用于绑定注册状态下的手机号码输入框
      regisEmail: '', // 该变量用于绑定注册状态下的邮箱输入框
      regisId: '',  // 注册模式下的账户
      regisPass: '',// 注册模式下的密码
      regisPassComfirm: '', // 确认密码
      loginInput: '', //登录账户/邮箱/手机
      loginPass: '' //密码
    }
  },
  methods: {
    validateLogin (){
      if(this.loginInput === '' || this.loginPass === '') {
        Message({
          type:'warning',
          duration:2000,
          message:'请输入账户和密码'
        })
        return false
      }
      return true
      
    },
    validatePhone () {
       // 去掉空格
      const cleanPhone = this.regisPhone.replace(/\s+/g, '');
      
      // E.164 格式正则：+ 后 1~15 位数字
      const regex = /^\+\d{1,15}$/;

      if (!regex.test(cleanPhone)) {
        Message({
          message:'电话号码格式错误（必须为 E.164 格式，如 +8613800138000)',
          type:'warning',
          duration: 2500
        })
        
        return;
      }
      Message({
        message:'格式正确，正在获取短信验证码',
        type:'success',
        duration:1000
      })
      console.log('提交的号码：', cleanPhone);
    },
    validateEmail () {
      if(!/^([A-Za-z0-9_\-\.])+\@([A-Za-z0-9_\-\.])+\.([A-Za-z]{2,4})$/.test(this.regisEmail)){
        Message({
          message:'邮箱格式有误，请检查',
          type:'warning'
        })
        return
      }
       Message({
        message:'格式正确，正在获取邮箱验证码',
        type:'success',
        duration:1000
      })
    },
    validatePass () {
      if(this.regisPass==='' || this.regisPassComfirm === ''){
        Message({
          message: '请不要输入空的密码',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      if(this.regisPass !== this.regisPassComfirm){
        
        
        Message({
          message: '前后两次密码输入不一致',
          type: 'warning',
          duration: 2500
        })
        return false
      }
      return true
    },
    // 改变登录/注册状体函数
    handleChangeLogin() {
      this.isLogin = !this.isLogin
    },
    // 登陆逻辑函数
    async handleLogin() {
      console.log('执行登录逻辑')
      if(this.validateLogin()){
        const res = await loginWithId(this.loginInput,this.loginPass)
        console.log(res);
        Message({
          type:'success',
          duration:2000,
          message:'恭喜你登陆成功'
        })
        
      }
      
      
    },
    // 手机登录逻辑函数
    async handleLoginPhone() {
      console.log('执行手机号登录逻辑')
      const cleanPhone = this.loginInput.replace(/\s+/g, '');
       if(this.validateLogin()){
        const res = await loginWithPhone(cleanPhone,this.loginPass)
        console.log(res);
        Message({
          type:'success',
          duration:2000,
          message:'恭喜你登陆成功'
        })
        
      }
    },
    async handleLoginEmail() {
      console.log('执行邮箱登录逻辑')
       if(this.validateLogin()){
        const res = await loginWithEmail(this.loginInput,this.loginPass)
        console.log(res);
        Message({
          type:'success',
          duration:2000,
          message:'恭喜你登陆成功'
        })
        
      }
    },
    // 注册逻辑函数
    async regisWithPhone() {
      console.log('执行手机注册逻辑')
      if(this.validatePass()){
      const res = await regisAccountWithPhone(this.regisId,this.regisPhone,this.regisPass)
      console.log(res)
      Message({
        type:'success',
        message:'注册成功，欢迎使用brain overflow！ ',
        duration: 2000
      })
      }
    },
    async regisWithEmail(){
      console.log('执行邮箱注册逻辑');
      if(this.validatePass()){
      const res = await regisAccountWithEmail(this.regisId,this.regisEmail,this.regisPass)
      console.log(res)
      Message({
        type:'success',
        message:'注册成功，欢迎使用brain overflow！ ',
        duration: 2000
      })
      }
      
    },
    // 改变注册状态逻辑函数
    handleChangeRegis () {
      this.isRegisWithPhone = !this.isRegisWithPhone
    },
   
  }
}
</script>

<style lang="less" scoped>
.loginBox {
  width: 900px;
  height: 675px;
  background: rgba(255, 255, 255, 0.8);
  border-radius: 10px;
  opacity: 0.8;
  backdrop-filter: blur(10px);
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin: 100px auto;
  display: flex;
  justify-content: space-around;
  .left {
    position: relative;
    width: 400px;
    background-color: #fff;
    padding: 10px;

    .loginLogo {
      margin-top: 100px;
      color: #000;
    }

    .welcomeText {
      width: 100%;
      font-size: 16px;
      color: #000;
      display: flex;
      justify-content: space-between;
      margin-bottom: 20px;
      h2{
        display: block;
        width: 140px;
      }
      button{
        display: block;
        border: none;
        background-color: #fff;
        font-size: 14px;
        font-weight: 600;
       
      }
     
    }

    .loginForm {
      display: flex;
      flex-direction: column;
      input {
        height: 40px;
        border-radius: 20px;
        margin-bottom: 10px;
        padding: 0 15px;
      }
      button {
        height: 40px;
        border-radius: 20px;
        margin-bottom: 10px;
      }
      el-divider i {
        font-size: 20px;
      }
    }

    .regisForm {
      display: flex;
      flex-direction: column;
      .codeBox{
        display: flex;
        justify-content: space-between;
        input{
          width: 200px;
        }
        img{
          width: 150px;
          height: 45px;
        };
      }
      .connectCode{
        display: flex;
        justify-content: space-between;
        input{
          width: 200px;
        }
        img{
          display: block;
          width: 150px;
        };
      }
      input {
        height: 40px;
        border-radius: 20px;
        margin-bottom: 10px;
        padding: 0 15px;
      }
      button {
        height: 40px;
        border-radius: 20px;
        margin-bottom: 10px;
      }
      el-divider i {
        font-size: 20px;
      }
    }

    .loginToRegis {
      color: #000;
      text-decoration: none;
      position: absolute;
      right: 0;
      margin: 10px;
    }
    .regisToLogin {
      color: #000;
      text-decoration: none;
      position: absolute;
      right: 0;
      margin: 10px;
    }
  }
  .right {
    width: 400px;
    padding: 10px;
    img {
      margin-top: 50px;
      width: 400px;
    }
  }
}
</style>