<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import Toast from './Toast.vue';
import ConfirmDialog from './ConfirmDialog.vue';

const router = useRouter();
const route = useRoute();

// 响应式数据
const group = ref<any>(null);
const currentUser = ref<any>(null);
const users = ref<any[]>([]);
const allUsers = ref<any[]>([]);
const books = ref<any[]>([]);
const invites = ref<any[]>([]);
const myBooks = ref<any[]>([]);
const isLoading = ref(false);
const activeTab = ref('info');
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);
const showAddUserModal = ref(false);
const showAddBookModal = ref(false);
const showCreateInviteModal = ref(false);
const showConfirmDialog = ref(false);

// 表单数据
const inviteCode = ref('');
const inviteLimit = ref(1);
const inviteExpiry = ref(24); // 小时
const selectedBookId = ref('');
const selectedUserId = ref('');
const userToRemove = ref<any>(null);
const bookToRemove = ref<any>(null);
const inviteToRemove = ref<any>(null);

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

// 加载当前用户信息
const loadCurrentUser = async () => {
  try {
    const response = await apiRequest('/auth/me');
    if (response.ok) {
      const userData = await response.json();
      currentUser.value = userData;
    }
  } catch (error) {
    console.error('Failed to load current user:', error);
  }
};

// 加载分组详情
const loadGroupDetail = async () => {
  const groupId = route.params.id as string;
  isLoading.value = true;
  try {
    const response = await apiRequest(`/groups/${groupId}`);
    if (response.ok) {
      const data = await response.json();
      // console.log('Group data:', data); // 调试信息
      group.value = data;
    } else {
      showToast('加载分组详情失败', 'error');
    }
  } catch (error) {
    console.error('Failed to load group detail:', error);
    showToast('加载分组详情失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 加载分组成员
const loadGroupUsers = async () => {
  const groupId = route.params.id as string;
  try {
    const response = await apiRequest(`/groups/${groupId}/users`);
    if (response.ok) {
      const data = await response.json();
      users.value = data;
    }
  } catch (error) {
    console.error('Failed to load group users:', error);
  }
};

// 加载分组图书
const loadGroupBooks = async () => {
  const groupId = route.params.id as string;
  try {
    const response = await apiRequest(`/groups/${groupId}/books`);
    if (response.ok) {
      const data = await response.json();
      books.value = data;
    }
  } catch (error) {
    console.error('Failed to load group books:', error);
  }
};

// 加载我的图书
const loadMyBooks = async () => {
  try {
    const response = await apiRequest('/books');
    if (response.ok) {
      const data = await response.json();
      myBooks.value = data.items || data || [];
    }
  } catch (error) {
    console.error('Failed to load my books:', error);
  }
};

// 加载所有用户
const loadAllUsers = async () => {
  try {
    const response = await apiRequest('/users');
    if (response.ok) {
      const data = await response.json();
      allUsers.value = data.items || data || [];
    }
  } catch (error) {
    console.error('Failed to load all users:', error);
  }
};

// 加载邀请码
const loadInvites = async () => {
  const groupId = route.params.id as string;
  try {
    const response = await apiRequest(`/groups/${groupId}/invite-codes`);
    if (response.ok) {
      const data = await response.json();
      invites.value = data;
    }
  } catch (error) {
    console.error('Failed to load invites:', error);
  }
};

// 切换标签页
const switchTab = (tab: string) => {
  activeTab.value = tab;
  if (tab === 'users') {
    loadGroupUsers();
  } else if (tab === 'books') {
    loadGroupBooks();
    loadMyBooks();
  } else if (tab === 'invites') {
    loadInvites();
  }
};

// 打开添加用户模态框
const openAddUserModal = () => {
  showAddUserModal.value = true;
  // 重置表单
  inviteCode.value = '';
  selectedUserId.value = '';
  // 如果是管理员，加载所有用户
  if (isAdmin.value) {
    loadAllUsers();
  }
};

// 处理添加用户
const handleAddUser = async (event: Event) => {
  event.preventDefault();
  
  const groupId = route.params.id as string;
  isLoading.value = true;

  try {
    if (!selectedUserId.value) {
      showToast('请选择用户', 'error');
      return;
    }

    const response = await apiRequest(`/groups/${groupId}/users`, {
      method: 'POST',
      body: JSON.stringify({ user_id: selectedUserId.value }),
    });

    if (response.ok) {
      showAddUserModal.value = false;
      loadGroupUsers();
      loadGroupDetail(); // 重新加载分组详情以更新用户数量
      showToast('用户添加成功！');
    } else {
      showToast('添加失败', 'error');
    }
  } catch (error) {
    showToast('添加失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 处理添加图书
const handleAddBook = async (event: Event) => {
  event.preventDefault();
  
  if (!selectedBookId.value) {
    showToast('请选择图书', 'error');
    return;
  }

  const groupId = route.params.id as string;
  isLoading.value = true;

  try {
    const response = await apiRequest(`/groups/${groupId}/books`, {
      method: 'POST',
      body: JSON.stringify({ book_id: selectedBookId.value }),
    });

    if (response.ok) {
        showAddBookModal.value = false;
        loadGroupBooks();
        loadGroupDetail(); // 重新加载分组详情以更新图书数量
        showToast('图书添加成功！');
      } else {
        showToast('添加失败', 'error');
      }
  } catch (error) {
    showToast('添加失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 处理创建邀请码
const handleCreateInvite = async (event: Event) => {
  event.preventDefault();

  const groupId = route.params.id as string;
  isLoading.value = true;

  try {
    const response = await apiRequest(`/groups/${groupId}/invite-codes`, {
      method: 'POST',
      body: JSON.stringify({ 
        max_users: inviteLimit.value, 
        expires_in_days: Math.ceil(inviteExpiry.value / 24) 
      }),
    });

    if (response.ok) {
      showCreateInviteModal.value = false;
      loadInvites();
      showToast('邀请码创建成功！');
    } else {
      // 尝试获取错误信息
      try {
        const errorData = await response.json();
        showToast(`创建失败: ${errorData.message || '未知错误'}`, 'error');
      } catch {
        showToast('创建失败', 'error');
      }
    }
  } catch (error) {
    showToast('创建失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 打开确认对话框
const openConfirmDialog = (type: string, item: any) => {
  if (type === 'user') {
    userToRemove.value = item;
  } else if (type === 'book') {
    bookToRemove.value = item;
  } else if (type === 'invite') {
    inviteToRemove.value = item;
  }
  showConfirmDialog.value = true;
};

// 关闭确认对话框
const closeConfirmDialog = () => {
  showConfirmDialog.value = false;
  userToRemove.value = null;
  bookToRemove.value = null;
  inviteToRemove.value = null;
};

// 确认移除用户
const confirmRemoveUser = async () => {
  if (!userToRemove.value) return;
  
  const groupId = route.params.id as string;
  isLoading.value = true;
  try {
    const response = await apiRequest(`/groups/${groupId}/users/${userToRemove.value.id}`, {
      method: 'DELETE',
    });

    if (response.ok) {
        loadGroupUsers();
        loadGroupDetail(); // 重新加载分组详情以更新用户数量
        showToast('用户移除成功！');
      } else {
        showToast('移除失败', 'error');
      }
  } catch (error) {
    showToast('移除失败', 'error');
  } finally {
    isLoading.value = false;
    closeConfirmDialog();
  }
};

// 确认移除图书
const confirmRemoveBook = async () => {
  if (!bookToRemove.value) return;
  
  const groupId = route.params.id as string;
  isLoading.value = true;
  try {
    const response = await apiRequest(`/groups/${groupId}/books/${bookToRemove.value.id}`, {
      method: 'DELETE',
    });

    if (response.ok) {
        loadGroupBooks();
        loadGroupDetail(); // 重新加载分组详情以更新图书数量
        showToast('图书移除成功！');
      } else {
        showToast('移除失败', 'error');
      }
  } catch (error) {
    showToast('移除失败', 'error');
  } finally {
    isLoading.value = false;
    closeConfirmDialog();
  }
};

// 确认停用邀请码
const confirmRemoveInvite = async () => {
  if (!inviteToRemove.value) return;
  
  const groupId = route.params.id as string;
  isLoading.value = true;
  try {
    const response = await apiRequest(`/groups/${groupId}/invite-codes/${inviteToRemove.value.id}`, {
      method: 'PUT',
    });

    if (response.ok) {
        loadInvites();
        showToast('邀请码停用成功！');
      } else {
        showToast('停用失败', 'error');
      }
  } catch (error) {
    showToast('停用失败', 'error');
  } finally {
    isLoading.value = false;
    closeConfirmDialog();
  }
};

// 返回分组列表
const goBack = () => {
  router.push('/groups');
};

// 计算属性：是否为管理员
const isAdmin = computed(() => {
  return currentUser.value?.role === 'admin' || currentUser.value?.role === 'root';
});

// 计算属性：是否为贡献者
const isContributor = computed(() => {
  return currentUser.value?.role === 'contributor';
});

// 计算属性：是否可以管理用户
const canManageUsers = computed(() => {
  return isAdmin.value || isContributor.value;
});

// 计算属性：是否可以管理图书
const canManageBooks = computed(() => {
  return isAdmin.value || isContributor.value;
});

// 计算属性：是否可以创建邀请码
const canCreateInvite = computed(() => {
  return isAdmin.value || currentUser.value.id === group.value.created_by;
});

// 计算属性：是否可以管理邀请码（只有分组创建者、管理员和root可以）
const canManageInvites = computed(() => {
  if (!currentUser.value || !group.value) return false;
  
  // 检查是否是root或管理员
  if (currentUser.value.role === 'root' || currentUser.value.role === 'admin') {
    return true;
  }
  
  // 检查是否是分组创建者
  // console.log(group.value)
  return currentUser.value.id === group.value.created_by;
});

onMounted(async () => {
  await loadCurrentUser();
  loadGroupDetail();
  loadGroupUsers();
  loadGroupBooks();
});
</script>

<template>
  <div id="group-detail-page" class="page">
    <div class="flex items-center gap-4 mb-8">
      <button @click="goBack" class="p-2 hover:bg-slate-100 rounded-xl text-slate-600 transition-all">
        <i class="ri-arrow-left-line text-xl"></i>
      </button>
      <div>
        <h1 class="text-3xl lg:text-4xl font-bold text-slate-800">分组详情</h1>
        <p class="text-slate-500 mt-2">查看分组的详细信息</p>
      </div>
    </div>
    
    <div v-if="isLoading" class="flex justify-center items-center py-20">
      <div class="spinner"></div>
    </div>
    
    <div v-else-if="group" class="bg-white rounded-2xl shadow-sm border border-slate-100 overflow-hidden">
      <!-- 标签页导航 -->
      <div class="flex border-b border-slate-200">
        <button @click="switchTab('info')" 
          :class="[activeTab === 'info' ? 'border-b-2 border-indigo-600 text-indigo-600' : 'text-slate-600', 'px-8 py-4 font-semibold transition-all']">
          基本信息
        </button>
        <button @click="switchTab('users')" 
          :class="[activeTab === 'users' ? 'border-b-2 border-indigo-600 text-indigo-600' : 'text-slate-600', 'px-8 py-4 font-semibold transition-all']">
          成员管理
        </button>
        <button @click="switchTab('books')" 
          :class="[activeTab === 'books' ? 'border-b-2 border-indigo-600 text-indigo-600' : 'text-slate-600', 'px-8 py-4 font-semibold transition-all']">
          图书管理
        </button>
        <button v-if="canManageInvites" @click="switchTab('invites')" 
          :class="[activeTab === 'invites' ? 'border-b-2 border-indigo-600 text-indigo-600' : 'text-slate-600', 'px-8 py-4 font-semibold transition-all']">
          邀请码管理
        </button>
      </div>
      
      <!-- 基本信息标签页 -->
      <div v-show="activeTab === 'info'" class="p-8">
        <div class="space-y-6">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">分组名称</label>
            <div class="flex items-center gap-4 px-5 py-4 bg-slate-50 border-2 border-slate-200 rounded-xl">
              <i class="ri-folder-3-line text-slate-400 text-xl"></i>
              <span class="font-semibold text-slate-800">{{ group.name }}</span>
            </div>
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">描述</label>
            <div class="flex items-start gap-4 px-5 py-4 bg-slate-50 border-2 border-slate-200 rounded-xl">
              <i class="ri-file-text-line text-slate-400 text-xl mt-1"></i>
              <span class="text-slate-600">{{ group.description || '无描述' }}</span>
            </div>
          </div>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-6">
            <div>
              <label class="block text-sm font-semibold text-slate-700 mb-2">成员数量</label>
              <div class="flex items-center gap-4 px-5 py-4 bg-slate-50 border-2 border-slate-200 rounded-xl">
                <i class="ri-user-group-line text-slate-400 text-xl"></i>
                <span class="font-semibold text-slate-800">{{ users.length }} 名成员</span>
              </div>
            </div>
            <div>
              <label class="block text-sm font-semibold text-slate-700 mb-2">图书数量</label>
              <div class="flex items-center gap-4 px-5 py-4 bg-slate-50 border-2 border-slate-200 rounded-xl">
                <i class="ri-book-2-line text-slate-400 text-xl"></i>
                <span class="font-semibold text-slate-800">{{ books.length }} 本图书</span>
              </div>
            </div>
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">创建时间</label>
            <div class="flex items-center gap-4 px-5 py-4 bg-slate-50 border-2 border-slate-200 rounded-xl">
              <i class="ri-calendar-line text-slate-400 text-xl"></i>
              <span class="text-slate-600">{{ new Date(group.created_at).toLocaleString() }}</span>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 成员管理标签页 -->
      <div v-show="activeTab === 'users'" class="p-8">
        <div class="flex justify-between items-center mb-6">
          <h3 class="text-xl font-semibold text-slate-800">成员列表</h3>
          <button v-if="isAdmin" @click="openAddUserModal()" 
            class="btn-primary text-white px-6 py-2.5 rounded-xl font-semibold flex items-center gap-2">
            <i class="ri-user-add-line"></i>
            <span>添加成员</span>
          </button>
        </div>
        
        <div v-if="users.length > 0" class="space-y-4">
          <div v-for="user in users" :key="user.id" 
            class="flex items-center justify-between p-4 bg-slate-50 border border-slate-200 rounded-xl">
            <div class="flex items-center gap-4">
              <div class="w-10 h-10 gradient-bg rounded-full flex items-center justify-center shadow-md">
                <i class="ri-user-3-line text-lg text-white"></i>
              </div>
              <div>
                <p class="font-semibold text-slate-800">{{ user.username }}</p>
                <p class="text-sm text-slate-500">{{ user.role }}</p>
              </div>
            </div>
            <button v-if="canManageUsers && user.id !== currentUser.id" 
              @click="openConfirmDialog('user', user)" 
              class="p-2 hover:bg-slate-200 rounded-xl text-rose-500 transition-all">
              <i class="ri-delete-bin-line text-lg"></i>
            </button>
          </div>
        </div>
        
        <div v-else class="text-center py-12">
          <div class="inline-flex items-center justify-center w-20 h-20 bg-slate-100 rounded-full mb-4">
            <i class="ri-user-group-line text-3xl text-slate-400"></i>
          </div>
          <h4 class="text-lg font-semibold text-slate-800 mb-2">暂无成员</h4>
          <p class="text-slate-500">点击添加成员按钮邀请用户加入分组</p>
        </div>
      </div>
      
      <!-- 图书管理标签页 -->
      <div v-show="activeTab === 'books'" class="p-8">
        <div class="flex justify-between items-center mb-6">
          <h3 class="text-xl font-semibold text-slate-800">图书列表</h3>
          <button v-if="canManageBooks && myBooks.length > 0" @click="showAddBookModal = true" 
            class="btn-primary text-white px-6 py-2.5 rounded-xl font-semibold flex items-center gap-2">
            <i class="ri-book-add-line"></i>
            <span>添加图书</span>
          </button>
        </div>
        
        <div v-if="books.length > 0" class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
          <div v-for="book in books" :key="book.id" 
            class="flex flex-col p-4 bg-slate-50 border border-slate-200 rounded-xl">
            <div class="flex items-center gap-4 mb-3">
              <div class="w-12 h-12 book-cover flex items-center justify-center rounded-lg flex-shrink-0">
                <i class="ri-book-2-line text-xl text-white"></i>
              </div>
              <div class="flex-1">
                <p class="font-semibold text-slate-800 truncate">{{ book.title }}</p>
                <p class="text-sm text-slate-500">{{ book.author || '未知作者' }}</p>
                <p class="text-xs text-slate-400">{{ book.format ? book.format.toUpperCase() : '未知格式' }}</p>
              </div>
            </div>
            <div class="flex justify-end mt-auto">
              <button v-if="canManageBooks" 
                @click="openConfirmDialog('book', book)" 
                class="p-2 hover:bg-slate-200 rounded-xl text-rose-500 transition-all">
                <i class="ri-delete-bin-line text-lg"></i>
              </button>
            </div>
          </div>
        </div>
        
        <div v-else class="text-center py-12">
          <div class="inline-flex items-center justify-center w-20 h-20 bg-slate-100 rounded-full mb-4">
            <i class="ri-book-2-line text-3xl text-slate-400"></i>
          </div>
          <h4 class="text-lg font-semibold text-slate-800 mb-2">暂无图书</h4>
          <p class="text-slate-500">点击添加图书按钮将你的图书添加到分组</p>
        </div>
      </div>
      
      <!-- 邀请码管理标签页 -->
      <div v-show="activeTab === 'invites'" class="p-8">
        <div class="flex justify-between items-center mb-6">
          <h3 class="text-xl font-semibold text-slate-800">邀请码列表</h3>
          <button v-if="canCreateInvite" @click="showCreateInviteModal = true" 
            class="btn-primary text-white px-6 py-2.5 rounded-xl font-semibold flex items-center gap-2">
            <i class="ri-key-2-line"></i>
            <span>创建邀请码</span>
          </button>
        </div>
        
        <div v-if="invites.length > 0" class="space-y-4">
          <div v-for="invite in invites" :key="invite.id" 
            :class="[invite.is_active === false ? 'bg-slate-100 border-slate-300' : 'bg-slate-50 border-slate-200', 'p-4 border rounded-xl']">
            <div class="flex justify-between items-center mb-3">
              <div>
                <p class="font-semibold text-slate-800">邀请码</p>
                <p :class="invite.is_active === false ? 'text-slate-400' : 'text-slate-600'" class="text-sm font-mono">{{ invite.code }}</p>
              </div>
              <div class="flex items-center gap-3">
                <span :class="invite.is_active === false ? 'bg-slate-200 text-slate-600' : invite.used_count < invite.limit ? 'bg-emerald-100 text-emerald-800' : 'bg-rose-100 text-rose-800'" 
                  class="px-3 py-1 rounded-full text-xs font-semibold">
                  {{ invite.is_active === false ? '已停用' : `${invite.used_count}/${invite.max_users}` }}
                </span>
                <button v-if="canCreateInvite && invite.is_active !== false" 
                  @click="openConfirmDialog('invite', invite)" 
                  class="p-2 hover:bg-slate-200 rounded-xl text-rose-500 transition-all" 
                  title="停用邀请码">
                  <i class="ri-delete-bin-line text-lg"></i>
                </button>
              </div>
            </div>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-3 text-sm">
              <div>
                <span class="text-slate-500">创建时间:</span>
                <span class="text-slate-700 ml-2">{{ new Date(invite.created_at).toLocaleString() }}</span>
              </div>
              <div>
                <span class="text-slate-500">过期时间:</span>
                <span class="text-slate-700 ml-2">{{ new Date(invite.expires_at).toLocaleString() }}</span>
              </div>
            </div>
          </div>
        </div>
        
        <div v-else class="text-center py-12">
          <div class="inline-flex items-center justify-center w-20 h-20 bg-slate-100 rounded-full mb-4">
            <i class="ri-key-2-line text-3xl text-slate-400"></i>
          </div>
          <h4 class="text-lg font-semibold text-slate-800 mb-2">暂无邀请码</h4>
          <p v-if="canCreateInvite" class="text-slate-500">点击创建邀请码按钮生成邀请码</p>
          <p v-else class="text-slate-500">只有管理员可以创建邀请码</p>
        </div>
      </div>
    </div>
    
    <div v-else class="text-center py-20">
      <div class="inline-flex items-center justify-center w-28 h-28 bg-slate-100 rounded-full mb-6">
        <i class="ri-folder-3-line text-5xl text-slate-400"></i>
      </div>
      <h3 class="text-2xl font-semibold text-slate-800 mb-3">分组不存在</h3>
      <p class="text-slate-500 mb-6">该分组可能已被删除</p>
      <button @click="goBack" class="btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
        返回分组列表
      </button>
    </div>
    
    <!-- 添加用户模态框 -->
    <div id="add-user-modal" class="modal" :class="{ show: showAddUserModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-md mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">添加成员</h2>
          <button @click="showAddUserModal = false" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleAddUser" class="space-y-5">
          <!-- 直接添加方式 -->
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">选择用户</label>
            <select v-model="selectedUserId" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none">
              <option value="">请选择用户</option>
              <option v-for="user in allUsers" :key="user.id" :value="user.id">
                {{ user.username }} - {{ user.role }}
              </option>
            </select>
          </div>
          
          <div class="flex gap-4 pt-5">
            <button type="button" @click="showAddUserModal = false" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '添加中...' : '添加' }}
            </button>
          </div>
        </form>
      </div>
    </div>
    
    <!-- 添加图书模态框 -->
    <div id="add-book-modal" class="modal" :class="{ show: showAddBookModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-md mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">添加图书</h2>
          <button @click="showAddBookModal = false" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleAddBook" class="space-y-5">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">选择图书</label>
            <select v-model="selectedBookId" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none">
              <option value="">请选择图书</option>
              <option v-for="book in myBooks" :key="book.id" :value="book.id">
                {{ book.title }} - {{ book.author || '未知作者' }}
              </option>
            </select>
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="showAddBookModal = false" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '添加中...' : '添加' }}
            </button>
          </div>
        </form>
      </div>
    </div>
    
    <!-- 创建邀请码模态框 -->
    <div id="create-invite-modal" class="modal" :class="{ show: showCreateInviteModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-md mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">创建邀请码</h2>
          <button @click="showCreateInviteModal = false" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleCreateInvite" class="space-y-5">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">使用次数限制</label>
            <input type="number" v-model.number="inviteLimit" min="1" max="100" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入使用次数限制">
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">过期时间（小时）</label>
            <input type="number" v-model.number="inviteExpiry" min="1" max="168" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入过期时间">
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="showCreateInviteModal = false" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '创建中...' : '创建' }}
            </button>
          </div>
        </form>
      </div>
    </div>
    
    <!-- 确认对话框 -->
    <ConfirmDialog 
        :show="showConfirmDialog"
        :title="userToRemove ? '移除成员' : bookToRemove ? '移除图书' : '停用邀请码'"
        :message="userToRemove ? `确定要移除用户 ${userToRemove.username} 吗？` : bookToRemove ? `确定要移除图书 ${bookToRemove?.title} 吗？` : `确定要停用邀请码 ${inviteToRemove?.code} 吗？`"
        :confirm-text="'确定'"
        :cancel-text="'取消'"
        @confirm="userToRemove ? confirmRemoveUser() : bookToRemove ? confirmRemoveBook() : confirmRemoveInvite()"
        @cancel="closeConfirmDialog"
    />
    
    <!-- Toast 提示框 -->
    <Toast 
        :message="showToastMessage"
        :type="showToastType"
        :show="isToastVisible"
    />
  </div>
</template>