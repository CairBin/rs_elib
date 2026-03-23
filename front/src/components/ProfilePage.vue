<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Toast from './Toast.vue';

const router = useRouter();

// 响应式数据
const currentUser = ref<any>(null);
const isLoading = ref(false);
const profilePassword = ref('');
const profilePasswordConfirm = ref('');
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);

// API 请求
const API_BASE = '/api';

const apiRequest = async (endpoint: string, options: RequestInit = {}) => {
  const token = localStorage.getItem('token');
  const headers: Record<string, string> = {
    ...(options.headers as Record<string, string>),
  };

  if (token) {
    headers['Authorization'] = `Bearer ${token}`;
  }

  if (!(options.body instanceof FormData)) {
    headers['Content-Type'] = 'application/json';
  }

  const response = await fetch(`${API_BASE}${endpoint}`, {
    ...options,
    headers,
  });

  if (response.status === 401) {
    localStorage.removeItem('token');
    router.push('/login');
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

// 加载用户信息
const loadProfile = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest('/auth/me');
    if (response.ok) {
      const userData = await response.json();
      currentUser.value = userData;
    }
  } catch (error) {
    console.error('Failed to load user info:', error);
    showToast('加载个人信息失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 获取角色文本
const getRoleText = (role: string) => {
  const roleMap: Record<string, string> = {
    'root': '超级管理员',
    'admin': '管理员',
    'contributor': '贡献者',
    'user': '阅读者'
  };
  return roleMap[role] || '阅读者';
};

// 更新个人信息
const updateProfile = async () => {
  if (profilePassword.value !== profilePasswordConfirm.value) {
    showToast('两次输入的密码不一致', 'error');
    return;
  }

  if (profilePassword.value) {
    isLoading.value = true;
    try {
      const response = await apiRequest('/users/me/profile', {
        method: 'PUT',
        body: JSON.stringify({ password: profilePassword.value }),
      });

      if (response.ok) {
        showToast('密码修改成功！');
        // 重置密码输入
        profilePassword.value = '';
        profilePasswordConfirm.value = '';
      } else {
        showToast('修改失败', 'error');
      }
    } catch (error) {
      showToast('修改失败', 'error');
    } finally {
      isLoading.value = false;
    }
  } else {
    showToast('请输入新密码', 'error');
  }
};

onMounted(() => {
  loadProfile();
});
</script>

<template>
  <div id="profile-page" class="page">
    <div class="mb-8">
      <h1 class="text-3xl lg:text-4xl font-bold text-slate-800">我的信息</h1>
      <p class="text-slate-500 mt-2">查看和修改您的个人信息</p>
    </div>
    
    <div v-if="isLoading" class="flex justify-center items-center py-20">
      <div class="spinner"></div>
    </div>
    
    <div v-else class="bg-white rounded-2xl shadow-sm border border-slate-100 p-8 max-w-2xl">
      <div class="space-y-6">
        <div>
          <label class="block text-sm font-semibold text-slate-700 mb-2">用户名</label>
          <div class="flex items-center gap-4 px-5 py-4 bg-slate-50 border-2 border-slate-200 rounded-xl">
            <i class="ri-user-3-line text-slate-400 text-xl"></i>
            <span id="profile-username" class="font-semibold text-slate-800">{{ currentUser?.username || '-' }}</span>
          </div>
          <p class="text-xs text-slate-400 mt-2">用户名不可修改</p>
        </div>
        <div>
          <label class="block text-sm font-semibold text-slate-700 mb-2">角色</label>
          <div class="flex items-center gap-4 px-5 py-4 bg-slate-50 border-2 border-slate-200 rounded-xl">
            <i class="ri-shield-star-line text-slate-400 text-xl"></i>
            <span id="profile-role" class="font-semibold text-slate-800">{{ currentUser ? getRoleText(currentUser.role) : '-' }}</span>
          </div>
        </div>
        <div>
          <label class="block text-sm font-semibold text-slate-700 mb-2">新密码（留空则不修改）</label>
          <input type="password" v-model="profilePassword" 
            class="input-modern w-full px-5 py-4 bg-slate-50 border-slate-200 rounded-xl outline-none" 
            placeholder="请输入新密码">
        </div>
        <div>
          <label class="block text-sm font-semibold text-slate-700 mb-2">确认新密码</label>
          <input type="password" v-model="profilePasswordConfirm" 
            class="input-modern w-full px-5 py-4 bg-slate-50 border-slate-200 rounded-xl outline-none" 
            placeholder="请再次输入新密码">
        </div>
        <button @click="updateProfile" :disabled="isLoading" 
          class="w-full btn-primary text-white px-6 py-4 rounded-xl font-semibold text-base">
          {{ isLoading ? '保存中...' : '保存修改' }}
        </button>
      </div>
    </div>
    
    <!-- Toast 提示框 -->
    <Toast 
        :message="showToastMessage"
        :type="showToastType"
        :show="isToastVisible"
    />
  </div>
</template>