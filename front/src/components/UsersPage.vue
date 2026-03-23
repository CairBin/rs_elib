<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Toast from './Toast.vue';
import ConfirmDialog from './ConfirmDialog.vue';

const router = useRouter();

// 响应式数据
const users = ref<any[]>([]);
const currentUser = ref<any>(null);
const isLoading = ref(false);
const showAddUserModal = ref(false);
const showEditUserRoleModal = ref(false);
const showEditUserPasswordModal = ref(false);
const showConfirmDialog = ref(false);
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);

// 确认对话框相关
const confirmUser = ref<any>(null);

// 表单数据
const addUserUsername = ref('');
const addUserPassword = ref('');
const addUserRole = ref('user');

const editUserId = ref('');
const editUserRole = ref('user');
const editUserPassword = ref('');

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

// 加载用户列表
const loadUsers = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest('/users');
    if (response.ok) {
      const data = await response.json();
      users.value = data;
    }
  } catch (error) {
    console.error('Failed to load users:', error);
    showToast('加载用户失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 打开添加用户模态框
const openAddUserModal = () => {
  showAddUserModal.value = true;
  // 重置表单
  addUserUsername.value = '';
  addUserPassword.value = '';
  addUserRole.value = 'user';
};

// 关闭添加用户模态框
const closeAddUserModal = () => {
  showAddUserModal.value = false;
};

// 处理添加用户
const handleAddUser = async (event: Event) => {
  event.preventDefault();
  
  if (!addUserUsername.value || !addUserPassword.value) {
    showToast('请输入用户名和密码', 'error');
    return;
  }

  isLoading.value = true;

  try {
    const response = await apiRequest('/users', {
      method: 'POST',
      body: JSON.stringify({ 
        username: addUserUsername.value, 
        password: addUserPassword.value, 
        role: addUserRole.value 
      }),
    });

    if (response.ok) {
      closeAddUserModal();
      loadUsers();
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

// 打开修改用户角色模态框
const openEditUserRoleModal = (user: any) => {
  showEditUserRoleModal.value = true;
  editUserId.value = user.id;
  editUserRole.value = user.role;
};

// 关闭修改用户角色模态框
const closeEditUserRoleModal = () => {
  showEditUserRoleModal.value = false;
};

// 处理修改用户角色
const handleEditUserRole = async (event: Event) => {
  event.preventDefault();

  isLoading.value = true;

  try {
    const response = await apiRequest(`/users/${editUserId.value}/role`, {
      method: 'PUT',
      body: JSON.stringify({ role: editUserRole.value }),
    });

    if (response.ok) {
      closeEditUserRoleModal();
      loadUsers();
      showToast('角色修改成功！');
    } else {
      showToast('修改失败', 'error');
    }
  } catch (error) {
    showToast('修改失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 打开修改用户密码模态框
const openEditUserPasswordModal = (user: any) => {
  showEditUserPasswordModal.value = true;
  editUserId.value = user.id;
  editUserPassword.value = '';
};

// 关闭修改用户密码模态框
const closeEditUserPasswordModal = () => {
  showEditUserPasswordModal.value = false;
};

// 处理修改用户密码
const handleEditUserPassword = async (event: Event) => {
  event.preventDefault();
  
  if (!editUserPassword.value) {
    showToast('请输入新密码', 'error');
    return;
  }

  isLoading.value = true;

  try {
    const response = await apiRequest(`/users/${editUserId.value}/password`, {
      method: 'PUT',
      body: JSON.stringify({ password: editUserPassword.value }),
    });

    if (response.ok) {
      closeEditUserPasswordModal();
      showToast('密码修改成功！');
    } else {
      showToast('修改失败', 'error');
    }
  } catch (error) {
    showToast('修改失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 打开确认对话框
const openConfirmDialog = (user: any) => {
  confirmUser.value = user;
  showConfirmDialog.value = true;
};

// 关闭确认对话框
const closeConfirmDialog = () => {
  showConfirmDialog.value = false;
  confirmUser.value = null;
};

// 确认禁用/启用用户
const confirmToggleUserDisabled = async () => {
  if (!confirmUser.value) return;
  
  isLoading.value = true;
  try {
    const response = await apiRequest(`/users/${confirmUser.value.id}/disabled`, {
      method: 'PUT',
      body: JSON.stringify({ disabled: !confirmUser.value.disabled }),
    });

    if (response.ok) {
      loadUsers();
      showToast(confirmUser.value.disabled ? '用户已启用！' : '用户已禁用！');
    } else {
      showToast('操作失败', 'error');
    }
  } catch (error) {
    showToast('操作失败', 'error');
  } finally {
    isLoading.value = false;
    closeConfirmDialog();
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

// 获取状态文本
const getStatusText = (status: boolean) => {
  return status ? '活跃' : '禁用';
};

// 判断是否可以修改用户角色
const canEditUserRole = (user: any) => {
  if (!currentUser.value) return false;
  
  // 超级管理员可以修改除了超级管理员以外的所有用户
  if (currentUser.value.role === 'root') {
    return user.role !== 'root';
  }
  
  // 管理员只能修改阅读者和贡献者，不能修改管理员或超级管理员
  if (currentUser.value.role === 'admin') {
    return user.role === 'user' || user.role === 'contributor';
  }
  
  // 其他角色不能修改任何用户角色
  return false;
};

// 获取可选的角色列表
const getAvailableRoles = () => {
  if (!currentUser.value) return [];
  
  const roles = [
    { value: 'user', label: '阅读者' },
    { value: 'contributor', label: '贡献者' }
  ];
  
  // 超级管理员可以添加管理员角色，但不能添加超级管理员
  if (currentUser.value.role === 'root') {
    roles.push({ value: 'admin', label: '管理员' });
  }
  
  return roles;
};

// 判断是否可以禁用用户
const canToggleUserDisabled = (user: any) => {
  if (!currentUser.value) return false;
  
  // 超级管理员不能被禁用
  if (user.role === 'root') {
    return false;
  }
  
  // 超级管理员可以禁用其他用户
  if (currentUser.value.role === 'root') {
    return true;
  }
  
  // 管理员可以禁用阅读者和贡献者，不能禁用管理员或超级管理员
  if (currentUser.value.role === 'admin') {
    return user.role === 'user' || user.role === 'contributor';
  }
  
  // 其他角色不能禁用任何用户
  return false;
};

onMounted(async () => {
  await loadCurrentUser();
  loadUsers();
});
</script>

<template>
  <div id="users-page" class="page">
    <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between mb-8 gap-6">
      <div>
        <h1 class="text-3xl lg:text-4xl font-bold text-slate-800">用户管理</h1>
        <p class="text-slate-500 mt-2">管理系统用户</p>
      </div>
      <button @click="openAddUserModal" 
        class="btn-primary text-white px-6 py-3.5 rounded-xl font-semibold flex items-center gap-2.5">
        <i class="ri-user-add-line text-xl"></i>
        <span>添加用户</span>
      </button>
    </div>
    
    <div class="bg-white rounded-2xl shadow-sm border border-slate-100 overflow-hidden">
      <div class="overflow-x-auto">
        <table class="w-full">
          <thead class="bg-slate-50">
            <tr>
              <th class="px-8 py-5 text-left text-sm font-bold text-slate-600">用户</th>
              <th class="px-8 py-5 text-left text-sm font-bold text-slate-600">角色</th>
              <th class="px-8 py-5 text-left text-sm font-bold text-slate-600">状态</th>
              <th class="px-8 py-5 text-left text-sm font-bold text-slate-600">创建时间</th>
              <th class="px-8 py-5 text-left text-sm font-bold text-slate-600">操作</th>
            </tr>
          </thead>
          <tbody id="users-list" class="divide-y divide-slate-100">
            <tr v-for="user in users" :key="user.id">
              <td class="px-8 py-5">
                <div class="flex items-center gap-4">
                  <div class="w-10 h-10 gradient-bg rounded-full flex items-center justify-center shadow-md">
                    <i class="ri-user-3-line text-lg text-white"></i>
                  </div>
                  <div>
                    <p class="font-semibold text-slate-800">{{ user.username }}</p>
                  </div>
                </div>
              </td>
              <td class="px-8 py-5">
                <span class="px-3 py-1 rounded-full text-xs font-semibold" 
                  :class="user.role === 'root' ? 'bg-purple-100 text-purple-800' : 
                         user.role === 'admin' ? 'bg-blue-100 text-blue-800' : 
                         user.role === 'contributor' ? 'bg-green-100 text-green-800' : 
                         'bg-slate-100 text-slate-800'">
                  {{ getRoleText(user.role) }}
                </span>
              </td>
              <td class="px-8 py-5">
                <span class="px-3 py-1 rounded-full text-xs font-semibold" 
                  :class="!user.disabled ? 'bg-emerald-100 text-emerald-800' : 'bg-rose-100 text-rose-800'">
                  {{ getStatusText(!user.disabled) }}
                </span>
              </td>
              <td class="px-8 py-5 text-sm text-slate-500">
                {{ new Date(user.created_at).toLocaleString() }}
              </td>
              <td class="px-8 py-5">
                <div class="flex items-center gap-3">
                  <button v-if="canEditUserRole(user)" @click="openEditUserRoleModal(user)" 
                    class="p-2 hover:bg-slate-100 rounded-xl text-slate-600 transition-all">
                    <i class="ri-shield-star-line text-lg"></i>
                  </button>
                  <button @click="openEditUserPasswordModal(user)" 
                    class="p-2 hover:bg-slate-100 rounded-xl text-slate-600 transition-all">
                    <i class="ri-lock-password-line text-lg"></i>
                  </button>
                  <button v-if="canToggleUserDisabled(user)" @click="openConfirmDialog(user)" 
                    class="p-2 hover:bg-slate-100 rounded-xl text-slate-600 transition-all">
                    <i :class="user.disabled ? 'ri-play-circle-line text-emerald-500' : 'ri-pause-circle-line text-rose-500'"></i>
                  </button>
                </div>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    
    <!-- 添加用户模态框 -->
    <div id="add-user-modal" class="modal" :class="{ show: showAddUserModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-md mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">添加用户</h2>
          <button @click="closeAddUserModal" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleAddUser" class="space-y-5">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">用户名</label>
            <input type="text" v-model="addUserUsername" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入用户名">
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">密码</label>
            <input type="password" v-model="addUserPassword" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入密码">
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">角色</label>
            <select v-model="addUserRole" 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none">
              <option value="user">阅读者</option>
              <option value="contributor">贡献者</option>
              <option v-if="currentUser && currentUser.role === 'root'" value="admin">管理员</option>
            </select>
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="closeAddUserModal" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '添加中...' : '添加' }}
            </button>
          </div>
        </form>
      </div>
    </div>
    
    <!-- 修改用户角色模态框 -->
    <div id="edit-user-role-modal" class="modal" :class="{ show: showEditUserRoleModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-md mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">修改用户角色</h2>
          <button @click="closeEditUserRoleModal" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleEditUserRole" class="space-y-5">
          <input type="hidden" v-model="editUserId">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">用户角色</label>
            <select v-model="editUserRole" 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none">
              <option v-for="role in getAvailableRoles()" :key="role.value" :value="role.value">
                {{ role.label }}
              </option>
            </select>
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="closeEditUserRoleModal" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '保存中...' : '保存' }}
            </button>
          </div>
        </form>
      </div>
    </div>
    
    <!-- 修改用户密码模态框 -->
    <div id="edit-user-password-modal" class="modal" :class="{ show: showEditUserPasswordModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-md mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">修改用户密码</h2>
          <button @click="closeEditUserPasswordModal" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleEditUserPassword" class="space-y-5">
          <input type="hidden" v-model="editUserId">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">新密码</label>
            <input type="password" v-model="editUserPassword" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入新密码">
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="closeEditUserPasswordModal" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '保存中...' : '保存' }}
            </button>
          </div>
        </form>
      </div>
    </div>
    
    <!-- 确认对话框 -->
    <ConfirmDialog 
        :show="showConfirmDialog"
        :title="confirmUser?.disabled ? '启用用户' : '禁用用户'"
        :message="confirmUser?.disabled ? `确定要启用用户 ${confirmUser?.username} 吗？` : `确定要禁用用户 ${confirmUser?.username} 吗？`"
        :confirm-text="confirmUser?.disabled ? '启用' : '禁用'"
        @confirm="confirmToggleUserDisabled"
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