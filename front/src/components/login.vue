<script setup lang="ts">
import { ref, computed } from 'vue';
import Toast from './Toast.vue';

const API_BASE = '/api';

// 响应式数据
const activeTab = ref('login');
const loginUsername = ref('');
const loginPassword = ref('');
const registerUsername = ref('');
const registerPassword = ref('');
const registerPasswordConfirm = ref('');
const isLoading = ref(false);
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);

// 密码验证状态
const passwordRequirements = ref({
  length: false,
  uppercase: false,
  lowercase: false,
  number: false,
  special: false
});

const passwordMatch = ref(false);
const passwordMatchMessage = ref('');

// 计算属性
const isRegisterSubmitDisabled = computed(() => {
  return !registerUsername.value.trim() ||
    !Object.values(passwordRequirements.value).every(v => v) ||
    !passwordMatch.value;
});

// 切换标签
const showTab = (tab: 'login' | 'register') => {
  activeTab.value = tab;
  if (tab === 'register') {
    resetRegisterForm();
  }
};

// 重置注册表单
const resetRegisterForm = () => {
  registerUsername.value = '';
  registerPassword.value = '';
  registerPasswordConfirm.value = '';
  passwordMatch.value = false;
  passwordMatchMessage.value = '';
  passwordRequirements.value = {
    length: false,
    uppercase: false,
    lowercase: false,
    number: false,
    special: false
  };
};

// 密码验证
const validatePassword = () => {
  const password = registerPassword.value;
  
  passwordRequirements.value = {
    length: password.length >= 8 && password.length <= 16,
    uppercase: /[A-Z]/.test(password),
    lowercase: /[a-z]/.test(password),
    number: /[0-9]/.test(password),
    special: /[!@#$%^&*()_+\-=\[\]{};':"\\|,.<>\/?]/.test(password)
  };
};

// 密码匹配验证
const validatePasswordMatch = () => {
  const password = registerPassword.value;
  const confirmPassword = registerPasswordConfirm.value;
  
  if (confirmPassword) {
    if (password === confirmPassword) {
      passwordMatch.value = true;
      passwordMatchMessage.value = '<i class="ri-checkbox-circle-fill text-emerald-500 mr-1"></i><span class="text-emerald-600">密码匹配</span>';
    } else {
      passwordMatch.value = false;
      passwordMatchMessage.value = '<i class="ri-error-warning-fill text-rose-500 mr-1"></i><span class="text-rose-600">密码不匹配</span>';
    }
  } else {
    passwordMatch.value = false;
    passwordMatchMessage.value = '';
  }
};

// API 请求
const apiRequest = async (endpoint: string, options: RequestInit = {}) => {
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string>),
  };

  if (!(options.body instanceof FormData)) {
    headers['Content-Type'] = 'application/json';
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers,
  });

  if (response.status === 401) {
    // 在登录页面，401错误是预期的，不需要跳转
    throw new Error('Unauthorized');
  }

  return response;
};

// 显示提示
const showToast = (message: string, type: 'success' | 'error' | 'info' = 'info') => {
  showToastMessage.value = message;
  showToastType.value = type;
  isToastVisible.value = true;
  
  // 3秒后自动关闭
  setTimeout(() => {
    isToastVisible.value = false;
  }, 3000);
};

// 登录处理
const handleLogin = async (event: Event) => {
  event.preventDefault();
  
  if (!loginUsername.value || !loginPassword.value) {
    showToast('请输入用户名和密码', 'error');
    return;
  }

  isLoading.value = true;
  
  try {
    const response = await apiRequest('/auth/login', {
      method: 'POST',
      body: JSON.stringify({ 
        username: loginUsername.value, 
        password: loginPassword.value 
      }),
    });

    if (response.ok) {
      const data = await response.json();
      localStorage.setItem('token', data.token);
      showToast('登录成功！');
      // 跳转到主应用
      window.location.href = '/';
    } else if (response.status === 403) {
      showToast('账号已被禁用', 'error');
    } else {
      showToast('用户名或密码错误', 'error');
    }
  } catch (error) {
    showToast('登录失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 注册处理
const handleRegister = async (event: Event) => {
  event.preventDefault();
  
  if (registerPassword.value !== registerPasswordConfirm.value) {
    showToast('两次输入的密码不一致', 'error');
    return;
  }

  isLoading.value = true;
  
  try {
    const response = await apiRequest('/auth/register', {
      method: 'POST',
      body: JSON.stringify({ 
        username: registerUsername.value, 
        password: registerPassword.value 
      }),
    });

    if (response.ok) {
      showToast('注册成功，请登录！');
      showTab('login');
    } else if (response.status === 403) {
      showToast('注册功能已关闭', 'error');
    } else if (response.status === 400) {
      const error = await response.text();
      showToast(error, 'error');
    } else {
      showToast('用户名已存在', 'error');
    }
  } catch (error) {
    showToast('注册失败', 'error');
  } finally {
    isLoading.value = false;
  }
};
</script>

<template>
    <div id="login-page" class="min-h-screen gradient-bg flex items-center justify-center p-4 relative overflow-hidden">
        <div class="absolute inset-0 overflow-hidden pointer-events-none">
            <div class="absolute -top-40 -right-40 w-80 h-80 bg-white/10 rounded-full blur-3xl"></div>
            <div class="absolute -bottom-40 -left-40 w-80 h-80 bg-white/10 rounded-full blur-3xl"></div>
        </div>

        <div class="glass-effect rounded-3xl shadow-2xl w-full max-w-md p-8 relative modal-content">
            <div class="text-center mb-8">
                <div class="inline-flex items-center justify-center w-20 h-20 bg-white/90 rounded-2xl mb-4 shadow-lg"
                    style="animation: pulse 2s ease-in-out infinite;">
                    <i class="ri-book-open-line text-4xl"
                        style="background: var(--primary-gradient); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text;"></i>
                </div>
                <h1 class="text-3xl font-bold text-slate-800">个人图书助理</h1>
                <p class="text-slate-500 mt-2">欢迎回来，请登录您的账户</p>
            </div>

            <div id="auth-tabs" class="flex mb-6 bg-slate-100 rounded-xl p-1.5">
                <button @click="showTab('login')"
                    :class="[
                        'flex-1 py-2.5 px-4 rounded-lg font-semibold text-sm transition-all',
                        activeTab === 'login' ? 'bg-white shadow text-indigo-600' : 'text-slate-600'
                    ]">登录</button>
                <button @click="showTab('register')"
                    :class="[
                        'flex-1 py-2.5 px-4 rounded-lg font-semibold text-sm transition-all',
                        activeTab === 'register' ? 'bg-white shadow text-indigo-600' : 'text-slate-600'
                    ]">注册</button>
            </div>

            <form @submit="handleLogin" class="space-y-5" v-show="activeTab === 'login'">
                <div>
                    <label class="block text-sm font-semibold text-slate-700 mb-2">用户名</label>
                    <div class="relative">
                        <i class="ri-user-3-line absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 text-lg"></i>
                        <input type="text" v-model="loginUsername" required
                            class="input-modern w-full pl-12 pr-4 py-3.5 bg-slate-50 border-slate-200 rounded-xl focus:bg-white outline-none"
                            placeholder="请输入用户名">
                    </div>
                </div>
                <div>
                    <label class="block text-sm font-semibold text-slate-700 mb-2">密码</label>
                    <div class="relative">
                        <i
                            class="ri-lock-password-line absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 text-lg"></i>
                        <input type="password" v-model="loginPassword" required
                            class="input-modern w-full pl-12 pr-4 py-3.5 bg-slate-50 border-slate-200 rounded-xl focus:bg-white outline-none"
                            placeholder="请输入密码">
                    </div>
                </div>
                <button type="submit" :disabled="isLoading"
                    class="w-full btn-primary text-white py-3.5 rounded-xl font-semibold text-base flex items-center justify-center gap-2">
                    <span>{{ isLoading ? '登录中...' : '登录' }}</span>
                    <i class="ri-arrow-right-line"></i>
                </button>
            </form>

            <form @submit="handleRegister" class="space-y-5" v-show="activeTab === 'register'">
                <div>
                    <label class="block text-sm font-semibold text-slate-700 mb-2">用户名</label>
                    <div class="relative">
                        <i class="ri-user-3-line absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 text-lg"></i>
                        <input type="text" v-model="registerUsername" required
                            class="input-modern w-full pl-12 pr-4 py-3.5 bg-slate-50 border-slate-200 rounded-xl focus:bg-white outline-none"
                            placeholder="请输入用户名">
                    </div>
                </div>
                <div>
                    <label class="block text-sm font-semibold text-slate-700 mb-2">密码</label>
                    <div class="relative">
                        <i
                            class="ri-lock-password-line absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 text-lg"></i>
                        <input type="password" v-model="registerPassword" required
                            class="input-modern w-full pl-12 pr-4 py-3.5 bg-slate-50 border-slate-200 rounded-xl focus:bg-white outline-none"
                            placeholder="请输入密码" @input="validatePassword">
                    </div>
                    <div class="mt-3 text-xs text-slate-500 space-y-1.5">
                        <div class="flex items-center gap-2">
                            <i :class="passwordRequirements.length ? 'ri-checkbox-circle-fill text-emerald-500' : 'ri-checkbox-blank-circle-line text-slate-300'"></i>
                            <span :class="passwordRequirements.length ? 'text-emerald-600' : 'text-slate-500'">8-16 个字符</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <i :class="passwordRequirements.uppercase ? 'ri-checkbox-circle-fill text-emerald-500' : 'ri-checkbox-blank-circle-line text-slate-300'"></i>
                            <span :class="passwordRequirements.uppercase ? 'text-emerald-600' : 'text-slate-500'">包含大写字母 (A-Z)</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <i :class="passwordRequirements.lowercase ? 'ri-checkbox-circle-fill text-emerald-500' : 'ri-checkbox-blank-circle-line text-slate-300'"></i>
                            <span :class="passwordRequirements.lowercase ? 'text-emerald-600' : 'text-slate-500'">包含小写字母 (a-z)</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <i :class="passwordRequirements.number ? 'ri-checkbox-circle-fill text-emerald-500' : 'ri-checkbox-blank-circle-line text-slate-300'"></i>
                            <span :class="passwordRequirements.number ? 'text-emerald-600' : 'text-slate-500'">包含数字 (0-9)</span>
                        </div>
                        <div class="flex items-center gap-2">
                            <i :class="passwordRequirements.special ? 'ri-checkbox-circle-fill text-emerald-500' : 'ri-checkbox-blank-circle-line text-slate-300'"></i>
                            <span :class="passwordRequirements.special ? 'text-emerald-600' : 'text-slate-500'">包含特殊字符 (!@#$%^&*)</span>
                        </div>
                    </div>
                </div>
                <div>
                    <label class="block text-sm font-semibold text-slate-700 mb-2">确认密码</label>
                    <div class="relative">
                        <i class="ri-lock-2-line absolute left-4 top-1/2 -translate-y-1/2 text-slate-400 text-lg"></i>
                        <input type="password" v-model="registerPasswordConfirm" required
                            class="input-modern w-full pl-12 pr-4 py-3.5 bg-slate-50 border-slate-200 rounded-xl focus:bg-white outline-none"
                            placeholder="请再次输入密码" @input="validatePasswordMatch">
                    </div>
                    <div class="mt-2 text-xs" v-html="passwordMatchMessage"></div>
                </div>
                <button type="submit" :disabled="isRegisterSubmitDisabled || isLoading"
                    class="w-full btn-primary text-white py-3.5 rounded-xl font-semibold text-base flex items-center justify-center gap-2">
                    <span>{{ isLoading ? '注册中...' : '注册' }}</span>
                    <i class="ri-user-add-line"></i>
                </button>
            </form>
        </div>
        
        <!-- Toast 提示框 -->
        <Toast 
            :message="showToastMessage"
            :type="showToastType"
            :show="isToastVisible"
        />
    </div>
</template>