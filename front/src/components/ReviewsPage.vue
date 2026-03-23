<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Toast from './Toast.vue';

const router = useRouter();

// 响应式数据
const activeTab = ref('books');
const isLoading = ref(false);
const pendingBooks = ref<any[]>([]);
const pendingComments = ref<any[]>([]);
const users = ref<any[]>([]);
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

// 切换标签页
const showReviewTab = (tab: 'books' | 'comments') => {
  activeTab.value = tab;
  if (tab === 'books') {
    loadPendingBooks();
  } else {
    loadPendingComments();
  }
};

// 加载待审核图书
const loadPendingBooks = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest('/books/pending');
    if (response.ok) {
      const result = await response.json();
      // console.log("图书审核")
      // console.log(result)
      pendingBooks.value = result.items || result || [];
    }
  } catch (error) {
    console.error('Failed to load pending books:', error);
    showToast('加载待审核图书失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 加载待审核评论
const loadPendingComments = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest('/comments/pending');
    if (response.ok) {
      const result = await response.json();
      // console.log(result)
      pendingComments.value = result.items || result || [];
    }
  } catch (error) {
    console.error('Failed to load pending comments:', error);
    showToast('加载待审核评论失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 加载用户列表
const loadUsers = async () => {
  try {
    const response = await apiRequest('/users');
    if (response.ok) {
      const result = await response.json();
      users.value = result.items || result || [];
    }
  } catch (error) {
    console.error('Failed to load users:', error);
  }
};

// 审核图书
const reviewBook = async (bookId: string, approved: boolean) => {
  try {
    const response = await apiRequest(`/books/${bookId}/review`, {
      method: 'PUT',
      body: JSON.stringify({ status: approved ? 'approved' : 'rejected' }),
    });

    if (response.ok) {
      showToast(approved ? '图书审核通过！' : '图书审核拒绝！');
      loadPendingBooks();
    } else {
      showToast('审核操作失败', 'error');
    }
  } catch (error) {
    showToast('审核操作失败', 'error');
  }
};

// 审核评论
const reviewComment = async (commentId: string, approved: boolean) => {
  try {
    const response = await apiRequest(`/comments/${commentId}/review`, {
      method: 'PUT',
      body: JSON.stringify({ status: approved ? 'approved' : 'rejected' }),
    });

    if (response.ok) {
      showToast(approved ? '评论审核通过！' : '评论审核拒绝！');
      loadPendingComments();
    } else {
      showToast('审核操作失败', 'error');
    }
  } catch (error) {
    showToast('审核操作失败', 'error');
  }
};

// 根据用户ID获取用户名
const getUsernameById = (userId: number) => {
  const user = users.value.find((u: any) => u.id === userId);
  return user ? user.username : '未知用户';
};

onMounted(async () => {
  await loadUsers();
  loadPendingBooks();
});
</script>

<template>
  <div id="reviews-page" class="page">
    <div class="mb-8">
      <h1 class="text-3xl lg:text-4xl font-bold text-slate-800">审核管理</h1>
      <p class="text-slate-500 mt-2">审核待处理的图书上传和评论</p>
    </div>
    
    <div class="flex gap-2 mb-8 bg-white rounded-xl p-1 border border-slate-200">
      <button @click="showReviewTab('books')" 
        :class="[
          'flex-1 py-3 px-4 rounded-lg font-semibold transition-all',
          activeTab === 'books' ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white' : 'text-slate-600 hover:bg-slate-50'
        ]">
        <i class="ri-book-2-line mr-2"></i>图书审核
      </button>
      <button @click="showReviewTab('comments')" 
        :class="[
          'flex-1 py-3 px-4 rounded-lg font-semibold transition-all',
          activeTab === 'comments' ? 'bg-gradient-to-r from-purple-500 to-indigo-600 text-white' : 'text-slate-600 hover:bg-slate-50'
        ]">
        <i class="ri-chat-3-line mr-2"></i>评论审核
      </button>
    </div>
    
    <!-- 图书审核内容 -->
    <div id="books-review-content" v-show="activeTab === 'books'">
      <div v-if="isLoading" class="flex justify-center items-center py-20">
        <div class="spinner"></div>
      </div>
      
      <div v-else-if="pendingBooks.length > 0" id="reviews-grid" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-6">
        <div v-for="book in pendingBooks" :key="book.id" 
          class="bg-white rounded-2xl shadow-sm border border-slate-100 overflow-hidden">
          <div class="h-48 book-cover flex items-center justify-center">
            <i class="ri-book-2-line text-5xl text-white/80"></i>
          </div>
          <div class="p-5">
            <h3 class="font-semibold text-lg text-slate-800 mb-2 truncate">{{ book.title }}</h3>
            <p class="text-slate-500 text-sm mb-3">{{ book.author || '未知作者' }}</p>
            <p class="text-xs text-slate-400 mb-4">上传者: {{ getUsernameById(book.created_by) }}</p>
            <div class="flex gap-3">
              <button @click="reviewBook(book.id, true)" 
                class="flex-1 px-4 py-2 bg-emerald-100 text-emerald-700 rounded-xl font-semibold text-sm hover:bg-emerald-200 transition-all">
                通过
              </button>
              <button @click="reviewBook(book.id, false)" 
                class="flex-1 px-4 py-2 bg-rose-100 text-rose-700 rounded-xl font-semibold text-sm hover:bg-rose-200 transition-all">
                拒绝
              </button>
            </div>
          </div>
        </div>
      </div>
      
      <div v-else id="empty-reviews" class="text-center py-20">
        <div class="inline-flex items-center justify-center w-28 h-28 bg-slate-100 rounded-full mb-6">
          <i class="ri-shield-check-line text-5xl text-slate-400"></i>
        </div>
        <h3 class="text-2xl font-semibold text-slate-800 mb-3">没有待审核的图书</h3>
        <p class="text-slate-500">所有图书都已审核完成</p>
      </div>
    </div>
    
    <!-- 评论审核内容 -->
    <div id="comments-review-content" v-show="activeTab === 'comments'">
      <div v-if="isLoading" class="flex justify-center items-center py-20">
        <div class="spinner"></div>
      </div>
      
      <div v-else-if="pendingComments.length > 0" id="comments-review-grid" class="space-y-4">
        <div v-for="comment in pendingComments" :key="comment.id" 
          class="bg-white rounded-2xl shadow-sm border border-slate-100 p-6">
          <div class="flex items-start gap-4 mb-4">
            <div class="w-10 h-10 gradient-bg rounded-full flex items-center justify-center shadow-md flex-shrink-0">
              <i class="ri-user-3-line text-lg text-white"></i>
            </div>
            <div>
              <div class="flex items-center gap-2 mb-1">
                <p class="font-semibold text-slate-800">{{ comment.username || '未知用户' }}</p>
                <span class="text-xs text-slate-400">{{ new Date(comment.created_at).toLocaleString() }}</span>
              </div>
              <p class="text-slate-600">{{ comment.content }}</p>
            </div>
          </div>
          <div class="flex gap-3 justify-end">
            <button @click="reviewComment(comment.id, true)" 
              class="px-4 py-2 bg-emerald-100 text-emerald-700 rounded-xl font-semibold text-sm hover:bg-emerald-200 transition-all">
              通过
            </button>
            <button @click="reviewComment(comment.id, false)" 
              class="px-4 py-2 bg-rose-100 text-rose-700 rounded-xl font-semibold text-sm hover:bg-rose-200 transition-all">
              拒绝
            </button>
          </div>
        </div>
      </div>
      
      <div v-else id="empty-comments-reviews" class="text-center py-20">
        <div class="inline-flex items-center justify-center w-28 h-28 bg-slate-100 rounded-full mb-6">
          <i class="ri-chat-3-line text-5xl text-slate-400"></i>
        </div>
        <h3 class="text-2xl font-semibold text-slate-800 mb-3">没有待审核的评论</h3>
        <p class="text-slate-500">所有评论都已审核完成</p>
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