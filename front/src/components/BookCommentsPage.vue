<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import Toast from './Toast.vue';

const router = useRouter();
const route = useRoute();

// 响应式数据
const bookId = ref(route.params.id as string);
const book = ref<any>(null);
const comments = ref<any[]>([]);
const isLoading = ref(false);
const commentContent = ref('');
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);
const settings = ref<any>({
  allow_comments: true,
  comment_review_enabled: false
});
const currentUser = ref<any>(null);

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

// 加载图书信息
const loadBookInfo = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest(`/books/${bookId.value}`);
    if (response.ok) {
      const data = await response.json();
      book.value = data;
    } else {
      showToast('加载图书信息失败', 'error');
    }
  } catch (error) {
    console.error('Failed to load book info:', error);
    showToast('加载图书信息失败', 'error');
  } finally {
    isLoading.value = false;
  }
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

// 加载配置
const loadSettings = async () => {
  try {
    const response = await apiRequest('/settings');
    if (response.ok) {
      try {
        const data = await response.json();
        settings.value = {
          allow_comments: data.allow_comments !== false,
          comment_review_enabled: data.enable_comment_review === true
        };
      } catch (jsonError) {
        console.error('Failed to parse settings JSON:', jsonError);
        // 解析JSON失败时使用默认值
      }
    } else if (response.status === 403) {
      // 普通用户和贡献者没有权限访问settings端点，使用默认值
      console.log('No permission to access settings, using default values');
    } else {
      // 其他错误，使用默认值
      console.error('Failed to load settings, status:', response.status);
    }
  } catch (error) {
    console.error('Failed to load settings:', error);
    // 发生错误时使用默认值
  }
};

// 加载评论
const loadComments = async () => {
  try {
    const response = await apiRequest(`/books/${bookId.value}/comments`);
    if (response.ok) {
      const data = await response.json();
      comments.value = data;
    }
  } catch (error) {
    console.error('Failed to load comments:', error);
    showToast('加载评论失败', 'error');
  }
};

// 提交评论
const submitComment = async (event: Event) => {
  event.preventDefault();
  
  if (!commentContent.value.trim()) {
    showToast('请输入评论内容', 'error');
    return;
  }

  try {
    const response = await apiRequest('/comments', {
      method: 'POST',
      body: JSON.stringify({ 
        content: commentContent.value,
        book_id: parseInt(bookId.value) 
      }),
    });

    if (response.ok) {
      // 管理员、root或未开启评论审核时，直接刷新评论
      if (isAdminOrRoot.value || !settings.value.comment_review_enabled) {
        showToast('评论发表成功！');
        loadComments();
      } else {
        showToast('评论提交成功，等待管理员审核');
      }
      commentContent.value = '';
    } else if (response.status === 403) {
      showToast('评论功能已关闭', 'error');
      // 更新settings为禁用评论状态
      settings.value.allow_comments = false;
    } else {
      showToast('评论发表失败', 'error');
    }
  } catch (error) {
    showToast('评论发表失败', 'error');
  }
};

// 返回图书列表
const backToBooks = () => {
  router.push('/');
};

// 跳转到阅读器
const navigateToReader = () => {
  router.push(`/reader/${bookId.value}`);
};

// 计算属性：是否为管理员或root
const isAdminOrRoot = computed(() => {
  return currentUser.value?.role === 'admin' || currentUser.value?.role === 'root';
});

// 计算属性：是否允许评论（管理员和root不受设置影响）
const canComment = computed(() => {
  return isAdminOrRoot.value || settings.value.allow_comments;
});

onMounted(async () => {
  await loadCurrentUser();
  loadSettings();
  loadBookInfo();
  loadComments();
});
</script>

<template>
  <div id="book-comments-page" class="page">
    <div class="flex items-center gap-4 mb-8">
      <button @click="backToBooks" class="p-3 hover:bg-white rounded-xl border border-slate-200 shadow-sm">
        <i class="ri-arrow-left-s-line text-2xl text-slate-700"></i>
      </button>
      <div class="flex-1">
        <h1 class="text-2xl lg:text-3xl font-bold text-slate-800">{{ book?.title || '图书评论' }}</h1>
        <p class="text-slate-500 mt-1">{{ book?.author || '作者' }}</p>
      </div>
      <button @click="navigateToReader" class="btn-primary text-white px-6 py-2.5 rounded-xl font-semibold flex items-center gap-2">
        <i class="ri-book-open-line"></i>
        <span>阅读</span>
      </button>
    </div>
    
    <div class="grid grid-cols-1 lg:grid-cols-3 gap-8">
      <!-- 图书信息 -->
      <div class="lg:col-span-1">
        <div class="bg-white rounded-2xl shadow-sm border border-slate-100 p-6 sticky top-8">
          <div class="h-64 book-cover flex items-center justify-center mb-6">
            <i class="ri-book-2-line text-6xl text-white/80"></i>
          </div>
          <div class="space-y-4">
            <div>
              <h3 class="font-semibold text-lg text-slate-800">{{ book?.title || '书名' }}</h3>
              <p class="text-slate-500 text-sm">{{ book?.author || '作者' }}</p>
            </div>
            <div class="flex items-center justify-between text-xs text-slate-400">
              <span>{{ book?.category || '未分类' }}</span>
              <span>{{ book?.file_type ? book.file_type.toUpperCase() : '未知格式' }}</span>
            </div>
            <div class="pt-4 border-t border-slate-200">
              <h4 class="font-semibold text-sm text-slate-700 mb-2">图书描述</h4>
              <p class="text-sm text-slate-600">{{ book?.description || '无描述' }}</p>
            </div>
          </div>
        </div>
      </div>
      
      <!-- 评论区域 -->
      <div class="lg:col-span-2">
        <div class="bg-white rounded-2xl shadow-sm border border-slate-100 p-6">
          <div class="flex items-center justify-between mb-6">
            <h3 class="font-bold text-xl text-slate-800">评论</h3>
            <span class="text-sm text-slate-500">{{ comments.length }} 条评论</span>
          </div>
          
          <!-- 添加评论表单 -->
          <div class="mb-8">
            <h4 class="font-semibold text-slate-700 mb-3">添加评论</h4>
            <form @submit="submitComment" class="space-y-4" :disabled="!canComment">
              <textarea v-model="commentContent" 
                :class="[canComment ? 'input-modern w-full px-5 py-4 bg-slate-50 border-slate-200 rounded-xl outline-none resize-none' : 'input-modern w-full px-5 py-4 bg-slate-100 border-slate-300 rounded-xl outline-none resize-none cursor-not-allowed']" 
                rows="4" 
                :placeholder="canComment ? '写下你的评论...' : '管理员已经关闭评论'"
                :disabled="!canComment"></textarea>
              <div class="flex justify-end">
                <button type="submit" :disabled="isLoading || !canComment" 
                  :class="[canComment ? 'btn-primary text-white px-6 py-2.5 rounded-xl font-semibold' : 'btn-primary text-white bg-slate-300 cursor-not-allowed px-6 py-2.5 rounded-xl font-semibold']">
                  {{ isLoading ? '提交中...' : canComment ? '提交评论' : '评论已关闭' }}
                </button>
              </div>
            </form>
          </div>
          
          <!-- 评论列表 -->
          <div id="comments-list" class="space-y-6">
            <div v-if="isLoading" class="text-slate-400 text-sm text-center py-8">加载中...</div>
            <div v-else-if="comments.length === 0" class="text-slate-400 text-sm text-center py-8">暂无评论</div>
            <div v-else v-for="comment in comments" :key="comment.id" class="bg-slate-50 p-5 rounded-xl">
              <div class="flex items-start gap-3 mb-3">
                <div class="w-10 h-10 gradient-bg rounded-full flex items-center justify-center shadow-sm flex-shrink-0">
                  <i class="ri-user-3-line text-sm text-white"></i>
                </div>
                <div>
                  <div class="flex items-center gap-2 mb-1">
                    <p class="font-semibold text-slate-800">{{ comment.username || '未知用户' }}</p>
                    <span class="text-xs text-slate-400">{{ new Date(comment.created_at).toLocaleString() }}</span>
                  </div>
                  <p class="text-slate-600">{{ comment.content }}</p>
                </div>
              </div>
            </div>
          </div>
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