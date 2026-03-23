<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Toast from './Toast.vue';

const router = useRouter();

// 响应式数据
const isLoading = ref(false);
const settings = ref({
  registration_enabled: true,
  allow_uploader_edit: true,
  allow_uploader_delete: true,
  enable_upload_review: true,
  allow_comments: true,
  enable_comment_review: true
});
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

// 加载设置
const loadSettings = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest('/settings');
    if (response.ok) {
      const data = await response.json();
      settings.value = data;
    }
  } catch (error) {
    console.error('Failed to load settings:', error);
    showToast('加载设置失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 保存设置
const saveSetting = async (key: keyof typeof settings.value, value: boolean) => {
  try {
    const response = await apiRequest(`/settings/${key}`, {
      method: 'PUT',
      body: JSON.stringify({ value: value.toString() }),
    });

    if (response.ok) {
      showToast('设置保存成功！');
    } else {
      showToast('保存失败', 'error');
      // 恢复原来的值
      loadSettings();
    }
  } catch (error) {
    showToast('保存失败', 'error');
    // 恢复原来的值
    loadSettings();
  }
};

// 切换设置
const toggleSetting = (key: keyof typeof settings.value) => {
  const newValue = !settings.value[key];
  settings.value[key] = newValue;
  saveSetting(key, newValue);
};

onMounted(() => {
  loadSettings();
});
</script>

<template>
  <div id="settings-page" class="page">
    <div class="mb-8">
      <h1 class="text-3xl lg:text-4xl font-bold text-slate-800">系统设置</h1>
      <p class="text-slate-500 mt-2">配置系统参数</p>
    </div>
    
    <div class="bg-white rounded-2xl shadow-sm border border-slate-100 p-8">
      <div class="space-y-8">
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex-1">
            <h3 class="font-semibold text-lg text-slate-800 mb-1">开启用户注册</h3>
            <p class="text-slate-500 text-sm">允许新用户自行注册账号</p>
          </div>
          <label class="switch flex-shrink-0">
            <input type="checkbox" :checked="settings.registration_enabled" @change="toggleSetting('registration_enabled')">
            <span class="slider"></span>
          </label>
        </div>
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex-1">
            <h3 class="font-semibold text-lg text-slate-800 mb-1">允许上传者修改书籍</h3>
            <p class="text-slate-500 text-sm">允许图书上传者修改自己上传的书籍信息</p>
          </div>
          <label class="switch flex-shrink-0">
            <input type="checkbox" :checked="settings.allow_uploader_edit" @change="toggleSetting('allow_uploader_edit')">
            <span class="slider"></span>
          </label>
        </div>
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex-1">
            <h3 class="font-semibold text-lg text-slate-800 mb-1">允许上传者删除书籍</h3>
            <p class="text-slate-500 text-sm">允许图书上传者删除自己上传的书籍</p>
          </div>
          <label class="switch flex-shrink-0">
            <input type="checkbox" :checked="settings.allow_uploader_delete" @change="toggleSetting('allow_uploader_delete')">
            <span class="slider"></span>
          </label>
        </div>
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex-1">
            <h3 class="font-semibold text-lg text-slate-800 mb-1">开启上传审核</h3>
            <p class="text-slate-500 text-sm">贡献者上传的图书需要经过管理员审核</p>
          </div>
          <label class="switch flex-shrink-0">
            <input type="checkbox" :checked="settings.enable_upload_review" @change="toggleSetting('enable_upload_review')">
            <span class="slider"></span>
          </label>
        </div>
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex-1">
            <h3 class="font-semibold text-lg text-slate-800 mb-1">允许评论</h3>
            <p class="text-slate-500 text-sm">允许用户对图书和章节发表评论</p>
          </div>
          <label class="switch flex-shrink-0">
            <input type="checkbox" :checked="settings.allow_comments" @change="toggleSetting('allow_comments')">
            <span class="slider"></span>
          </label>
        </div>
        <div class="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
          <div class="flex-1">
            <h3 class="font-semibold text-lg text-slate-800 mb-1">开启评论审核</h3>
            <p class="text-slate-500 text-sm">非管理员用户的评论需要经过审核才能显示</p>
          </div>
          <label class="switch flex-shrink-0">
            <input type="checkbox" :checked="settings.enable_comment_review" @change="toggleSetting('enable_comment_review')">
            <span class="slider"></span>
          </label>
        </div>
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