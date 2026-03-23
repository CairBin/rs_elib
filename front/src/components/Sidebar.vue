<script setup lang="ts">
import { computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';

const props = defineProps<{
  currentUser: any
}>();

const emit = defineEmits<{
  (e: 'logout'): void
}>();

const router = useRouter();
const route = useRoute();

// 计算当前激活的菜单项
const activeMenuItem = computed(() => {
  const path = route.path;
  if (path === '/') return 'books';
  if (path.startsWith('/groups')) return 'groups';
  if (path === '/users') return 'users';
  if (path === '/settings') return 'settings';
  if (path === '/reviews') return 'reviews';
  if (path === '/profile') return 'profile';
  return '';
});

// 跳转到指定页面
const navigateTo = (path: string) => {
  router.push(path);
};

// 退出登录
const logout = () => {
  emit('logout');
};

// 获取角色文本
const roleText = computed(() => {
  if (!props.currentUser) return '阅读者';
  const roleMap: Record<string, string> = {
    'root': '超级管理员',
    'admin': '管理员',
    'contributor': '贡献者',
    'user': '阅读者'
  };
  return roleMap[props.currentUser.role] || '阅读者';
});
</script>

<template>
  <aside id="sidebar" class="hidden lg:flex lg:flex-col w-72 bg-white border-r border-slate-200 fixed h-full z-40">
    <div class="p-6 border-b border-slate-100">
      <div class="flex items-center gap-4">
        <div class="w-12 h-12 gradient-bg rounded-2xl flex items-center justify-center shadow-lg">
          <i class="ri-book-open-line text-2xl text-white"></i>
        </div>
        <div>
          <span class="font-bold text-xl text-slate-800">图书助理</span>
          <p class="text-xs text-slate-400">管理你的阅读世界</p>
        </div>
      </div>
    </div>
    
    <nav class="flex-1 p-4 space-y-1.5 overflow-y-auto scrollbar-thin">
      <button @click="navigateTo('/')" 
        :class="['sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium', activeMenuItem === 'books' ? 'active text-slate-700' : 'text-slate-600']">
        <i class="ri-book-2-line text-xl"></i>
        <span>图书管理</span>
      </button>
      <button @click="navigateTo('/groups')" 
        :class="['sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium', activeMenuItem === 'groups' ? 'active text-slate-700' : 'text-slate-600']">
        <i class="ri-group-line text-xl"></i>
        <span>分组管理</span>
      </button>
      <button v-if="currentUser?.role === 'root' || currentUser?.role === 'admin'" @click="navigateTo('/reviews')" 
        :class="['sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium', activeMenuItem === 'reviews' ? 'active text-slate-700' : 'text-slate-600']">
        <i class="ri-shield-check-line text-xl"></i>
        <span>审核管理</span>
      </button>
      <button v-if="currentUser?.role === 'root' || currentUser?.role === 'admin'" @click="navigateTo('/users')" 
        :class="['sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium', activeMenuItem === 'users' ? 'active text-slate-700' : 'text-slate-600']">
        <i class="ri-user-settings-line text-xl"></i>
        <span>用户管理</span>
      </button>
      <button v-if="currentUser?.role === 'root' || currentUser?.role === 'admin'" @click="navigateTo('/settings')" 
        :class="['sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium', activeMenuItem === 'settings' ? 'active text-slate-700' : 'text-slate-600']">
        <i class="ri-settings-3-line text-xl"></i>
        <span>系统设置</span>
      </button>
      <button @click="navigateTo('/profile')" 
        :class="['sidebar-item w-full flex items-center gap-3.5 px-4 py-3.5 rounded-xl text-left font-medium', activeMenuItem === 'profile' ? 'active text-slate-700' : 'text-slate-600']">
        <i class="ri-user-smile-line text-xl"></i>
        <span>我的信息</span>
      </button>
    </nav>
    
    <div class="p-4 border-t border-slate-100">
      <div class="flex items-center gap-4 mb-4 p-3 bg-slate-50 rounded-xl">
        <div class="w-12 h-12 gradient-bg rounded-full flex items-center justify-center shadow-md">
          <i class="ri-user-3-line text-xl text-white"></i>
        </div>
        <div class="flex-1 min-w-0">
          <p class="font-semibold text-slate-800 truncate">{{ currentUser?.username || '用户' }}</p>
          <p class="text-xs text-slate-500">{{ roleText }}</p>
        </div>
      </div>
      <button @click="logout" class="w-full flex items-center gap-3 px-4 py-3 text-rose-600 hover:bg-rose-50 rounded-xl transition-all font-medium">
        <i class="ri-logout-box-r-line text-xl"></i>
        <span>退出登录</span>
      </button>
    </div>
  </aside>
</template>