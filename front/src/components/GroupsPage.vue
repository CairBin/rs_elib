<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Toast from './Toast.vue';

const router = useRouter();

// 响应式数据
const groups = ref<any[]>([]);
const isLoading = ref(false);
const showAddGroupModal = ref(false);
const showJoinWithInviteModal = ref(false);
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);

// 表单数据
const groupName = ref('');
const groupDescription = ref('');
const inviteCode = ref('');

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

// 加载分组
const loadGroups = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest('/groups');
    if (response.ok) {
      const result = await response.json();
      const loadedGroups = result.items || result || [];
      
      // 为每个分组加载用户和图书数量
      const groupsWithCounts = await Promise.all(loadedGroups.map(async (group: any) => {
        try {
          // 加载用户数量
          const usersResponse = await apiRequest(`/groups/${group.id}/users`);
          const users = usersResponse.ok ? await usersResponse.json() : [];
          
          // 加载图书数量
          const booksResponse = await apiRequest(`/groups/${group.id}/books`);
          const books = booksResponse.ok ? await booksResponse.json() : [];
          
          // 添加计数字段
          return {
            ...group,
            user_count: users.length,
            book_count: books.length
          };
        } catch (error) {
          console.error(`Failed to load counts for group ${group.id}:`, error);
          // 如果加载失败，使用默认值 0
          return {
            ...group,
            user_count: 0,
            book_count: 0
          };
        }
      }));
      
      groups.value = groupsWithCounts;
    }
  } catch (error) {
    console.error('Failed to load groups:', error);
    showToast('加载分组失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 打开创建分组模态框
const openAddGroupModal = () => {
  showAddGroupModal.value = true;
  // 重置表单
  groupName.value = '';
  groupDescription.value = '';
};

// 关闭创建分组模态框
const closeAddGroupModal = () => {
  showAddGroupModal.value = false;
};

// 处理创建分组
const handleAddGroup = async (event: Event) => {
  event.preventDefault();
  
  if (!groupName.value) {
    showToast('请输入分组名称', 'error');
    return;
  }

  isLoading.value = true;

  try {
    const response = await apiRequest('/groups', {
      method: 'POST',
      body: JSON.stringify({ 
        name: groupName.value, 
        description: groupDescription.value || null 
      }),
    });

    if (response.ok) {
      closeAddGroupModal();
      loadGroups();
      showToast('分组创建成功！');
    } else {
      showToast('创建失败', 'error');
    }
  } catch (error) {
    showToast('创建失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 打开通过邀请码加入模态框
const openJoinWithInviteModal = () => {
  showJoinWithInviteModal.value = true;
  // 重置表单
  inviteCode.value = '';
};

// 关闭通过邀请码加入模态框
const closeJoinWithInviteModal = () => {
  showJoinWithInviteModal.value = false;
};

// 处理通过邀请码加入
const handleJoinWithInvite = async (event: Event) => {
  event.preventDefault();
  
  if (!inviteCode.value) {
    showToast('请输入邀请码', 'error');
    return;
  }

  isLoading.value = true;

  try {
    const response = await apiRequest('/groups/join-with-invite', {
      method: 'POST',
      body: JSON.stringify({ code: inviteCode.value }),
    });

    if (response.ok) {
      closeJoinWithInviteModal();
      loadGroups();
      showToast('加入成功！');
    } else {
      showToast('加入失败，邀请码无效', 'error');
    }
  } catch (error) {
    showToast('加入失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 跳转到分组详情
const navigateToGroupDetail = (groupId: string) => {
  router.push(`/groups/${groupId}`);
};

onMounted(() => {
  loadGroups();
});
</script>

<template>
  <div id="groups-page" class="page">
    <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between mb-8 gap-6">
      <div>
        <h1 class="text-3xl lg:text-4xl font-bold text-slate-800">分组管理</h1>
        <p class="text-slate-500 mt-2">管理阅读分组和权限</p>
      </div>
      <div class="flex items-center gap-4">
        <button @click="openJoinWithInviteModal" 
          class="px-6 py-3.5 border-2 border-indigo-500 text-indigo-600 hover:bg-indigo-50 rounded-xl font-semibold flex items-center gap-2.5 transition-all">
          <i class="ri-key-2-line text-xl"></i>
          <span>通过邀请码加入</span>
        </button>
        <button @click="openAddGroupModal" 
          class="btn-primary text-white px-6 py-3.5 rounded-xl font-semibold flex items-center gap-2.5">
          <i class="ri-folder-add-line text-xl"></i>
          <span>创建分组</span>
        </button>
      </div>
    </div>
    
    <div v-if="isLoading" class="flex justify-center items-center py-20">
      <div class="spinner"></div>
    </div>
    
    <div v-else-if="groups.length > 0" id="groups-grid" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
      <div v-for="group in groups" :key="group.id" 
        class="bg-white rounded-2xl shadow-sm border border-slate-100 overflow-hidden card-hover cursor-pointer" 
        @click="navigateToGroupDetail(group.id)">
        <div class="h-40 gradient-bg flex items-center justify-center">
          <i class="ri-folder-3-line text-5xl text-white/80"></i>
        </div>
        <div class="p-5">
          <h3 class="font-semibold text-lg text-slate-800 mb-2 truncate">{{ group.name }}</h3>
          <p v-if="group.description" class="text-slate-500 text-sm mb-3 line-clamp-2">{{ group.description }}</p>
          <div class="flex items-center justify-between text-xs text-slate-400">
            <span>{{ group.user_count || 0 }} 名成员</span>
            <span>{{ group.book_count || 0 }} 本图书</span>
          </div>
        </div>
      </div>
    </div>
    
    <div v-else id="empty-groups" class="text-center py-20">
      <div class="inline-flex items-center justify-center w-28 h-28 bg-slate-100 rounded-full mb-6">
        <i class="ri-folder-3-line text-5xl text-slate-400"></i>
      </div>
      <h3 class="text-2xl font-semibold text-slate-800 mb-3">还没有分组</h3>
      <p class="text-slate-500">点击上方按钮创建你的第一个分组</p>
    </div>
    
    <!-- 创建分组模态框 -->
    <div id="add-group-modal" class="modal" :class="{ show: showAddGroupModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-xl mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">创建分组</h2>
          <button @click="closeAddGroupModal" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleAddGroup" class="space-y-5">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">分组名称 *</label>
            <input type="text" v-model="groupName" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入分组名称">
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">描述</label>
            <textarea v-model="groupDescription" rows="4" 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none resize-none" 
              placeholder="请输入分组描述"></textarea>
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="closeAddGroupModal" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '创建中...' : '创建' }}
            </button>
          </div>
        </form>
      </div>
    </div>
    
    <!-- 通过邀请码加入模态框 -->
    <div id="join-with-invite-modal" class="modal" :class="{ show: showJoinWithInviteModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-md mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">通过邀请码加入分组</h2>
          <button @click="closeJoinWithInviteModal" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleJoinWithInvite" class="space-y-5">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">邀请码</label>
            <input type="text" v-model="inviteCode" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入邀请码">
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="closeJoinWithInviteModal" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '加入中...' : '加入' }}
            </button>
          </div>
        </form>
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