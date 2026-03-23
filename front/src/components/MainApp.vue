<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Sidebar from './Sidebar.vue';

const router = useRouter();
const currentUser = ref<any>(null);
const isMobileSidebarOpen = ref(false);

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
    logout();
    throw new Error('Unauthorized');
  }

  return response;
};



// 退出登录
const logout = () => {
  localStorage.removeItem('token');
  router.push('/login');
};

// 切换移动端侧边栏
const toggleMobileSidebar = () => {
  isMobileSidebarOpen.value = !isMobileSidebarOpen.value;
};

// 加载用户信息
const loadUserInfo = async () => {
  try {
    const response = await apiRequest('/auth/me');
    if (response.ok) {
      const userData = await response.json();
      currentUser.value = userData;
    }
  } catch (error) {
    console.error('Failed to load user info:', error);
  }
};

onMounted(() => {
  loadUserInfo();
});
</script>

<template>
  <div id="main-app" class="min-h-screen bg-slate-50">
    <div class="flex min-h-screen">
      <!-- 桌面端侧边栏 -->
      <Sidebar :current-user="currentUser" @logout="logout" />
      
      <!-- 移动端头部 -->
      <div id="mobile-header" class="lg:hidden fixed top-0 left-0 right-0 bg-white/95 backdrop-blur-xl border-b border-slate-200 z-30 px-4 py-3">
        <div class="flex items-center justify-between">
          <button @click="toggleMobileSidebar" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-menu-4-line text-2xl text-slate-700"></i>
          </button>
          <div class="flex items-center gap-2">
            <div class="w-9 h-9 gradient-bg rounded-xl flex items-center justify-center">
              <i class="ri-book-open-line text-lg text-white"></i>
            </div>
            <span class="font-bold text-lg text-slate-800">图书助理</span>
          </div>
          <button @click="logout" class="p-2.5 hover:bg-rose-50 rounded-xl text-rose-600">
            <i class="ri-logout-box-r-line text-2xl"></i>
          </button>
        </div>
      </div>
      
      <!-- 移动端侧边栏 -->
      <div id="mobile-sidebar" class="lg:hidden fixed inset-0 z-50" v-show="isMobileSidebarOpen">
        <div class="absolute inset-0 bg-black/60 backdrop-blur-sm" @click="toggleMobileSidebar"></div>
        <aside class="absolute left-0 top-0 bottom-0 w-72 bg-white sidebar-mobile" :class="{ open: isMobileSidebarOpen }">
          <div class="p-6 border-b border-slate-100 flex items-center justify-between">
            <div class="flex items-center gap-3">
              <div class="w-10 h-10 gradient-bg rounded-xl flex items-center justify-center">
                <i class="ri-book-open-line text-xl text-white"></i>
              </div>
              <span class="font-bold text-xl text-slate-800">图书助理</span>
            </div>
            <button @click="toggleMobileSidebar" class="p-2 hover:bg-slate-100 rounded-xl">
              <i class="ri-close-line text-2xl text-slate-500"></i>
            </button>
          </div>
          <nav class="p-4 space-y-1.5">
            <button @click="router.push('/'); toggleMobileSidebar()" class="sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium text-slate-700">
              <i class="ri-book-2-line text-xl"></i>
              <span>图书管理</span>
            </button>
            <button @click="router.push('/groups'); toggleMobileSidebar()" class="sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium text-slate-600">
              <i class="ri-group-line text-xl"></i>
              <span>分组管理</span>
            </button>
            <button v-if="currentUser?.role === 'root' || currentUser?.role === 'admin'" @click="router.push('/reviews'); toggleMobileSidebar()" class="sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium text-slate-600">
              <i class="ri-shield-check-line text-xl"></i>
              <span>审核管理</span>
            </button>
            <button v-if="currentUser?.role === 'root' || currentUser?.role === 'admin'" @click="router.push('/users'); toggleMobileSidebar()" class="sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium text-slate-600">
              <i class="ri-user-settings-line text-xl"></i>
              <span>用户管理</span>
            </button>
            <button v-if="currentUser?.role === 'root' || currentUser?.role === 'admin'" @click="router.push('/settings'); toggleMobileSidebar()" class="sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium text-slate-600">
              <i class="ri-settings-3-line text-xl"></i>
              <span>系统设置</span>
            </button>
            <button @click="router.push('/profile'); toggleMobileSidebar()" class="sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium text-slate-600">
              <i class="ri-user-smile-line text-xl"></i>
              <span>我的信息</span>
            </button>
          </nav>
        </aside>
      </div>
      
      <!-- 主内容区域 -->
      <main class="flex-1 lg:ml-72 pt-20 lg:pt-0">
        <div class="p-6 lg:p-8">
          <router-view />
        </div>
      </main>
    </div>
  </div>
</template>