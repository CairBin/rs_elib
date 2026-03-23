<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import { useRouter, useRoute } from 'vue-router';
import Toast from './Toast.vue';

const router = useRouter();
const route = useRoute();

// 响应式数据
const bookId = computed(() => route.params.id as string);
const book = ref<any>(null);
const chapters = ref<any[]>([]);
const currentChapterIndex = ref(0);
const isLoading = ref(false);
const isChapterListOpen = ref(true);
const zoomLevel = ref(100);
const commentContent = ref('');
const comments = ref<any[]>([]);
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);
const settings = ref<any>({
  allow_comments: true,
  comment_review_enabled: false
});
const currentUser = ref<any>(null);

// 阅读器状态
const readerContent = ref('');
const currentChapter = ref<any>(null);
const totalChapters = ref(0);
const currentPage = ref(1);

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
    }
  } catch (error) {
    console.error('Failed to load book info:', error);
    showToast('加载图书信息失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 加载章节列表
const loadChapters = async () => {
  try {
    const response = await apiRequest(`/books/${bookId.value}/chapters`);
    if (response.ok) {
      const data = await response.json();
      chapters.value = data;
      totalChapters.value = data.length;
      if (data.length > 0) {
        loadChapter(0);
      }
    }
  } catch (error) {
    console.error('Failed to load chapters:', error);
    showToast('加载章节列表失败', 'error');
  }
};

// 加载章节内容
const loadChapter = async (index: number) => {
  if (index < 0 || index >= chapters.value.length) return;
  
  currentChapterIndex.value = index;
  currentChapter.value = chapters.value[index];
  currentPage.value = 1;
  
  try {
    const response = await apiRequest(`/books/${bookId.value}/chapters/${index + 1}`);
    if (response.ok) {
      const resJson = await response.json();
      // console.log('Chapter content:', resJson);
      readerContent.value = resJson.content || '';
      // 加载当前章节的评论
      loadComments();
    }
  } catch (error) {
    console.error('Failed to load chapter content:', error);
    showToast('加载章节内容失败', 'error');
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
  if (!currentChapter.value) return;
  
  try {
    const response = await apiRequest(`/chapters/${currentChapter.value.id}/comments`);
    if (response.ok) {
      const data = await response.json();
      // console.log(data)
      comments.value = data;
    }
  } catch (error) {
    console.error('Failed to load comments:', error);
  }
};

// 提交评论
const submitComment = async () => {
  if (!commentContent.value.trim()) {
    showToast('请输入评论内容', 'error');
    return;
  }

  if (!currentChapter.value) {
    showToast('请先选择章节', 'error');
    return;
  }

  try {
    const response = await apiRequest('/comments', {
      method: 'POST',
      body: JSON.stringify({ 
        content: commentContent.value,
        chapter_id: currentChapter.value.id 
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

// 上一章
const prevChapter = () => {
  loadChapter(currentChapterIndex.value - 1);
};

// 下一章
const nextChapter = () => {
  loadChapter(currentChapterIndex.value + 1);
};

// 切换章节列表
const toggleChapterList = () => {
  isChapterListOpen.value = !isChapterListOpen.value;
};

// 放大字体
const zoomIn = () => {
  if (zoomLevel.value < 150) {
    zoomLevel.value += 10;
  }
};

// 缩小字体
const zoomOut = () => {
  if (zoomLevel.value > 70) {
    zoomLevel.value -= 10;
  }
};

// 返回图书列表
const backToBooks = () => {
  router.push('/');
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
  loadChapters();
  loadComments();
});
</script>

<template>
  <div id="reader-page" class="page">
    <div class="mb-8 flex items-center gap-4">
      <button @click="backToBooks" class="p-3 hover:bg-white rounded-xl border border-slate-200 shadow-sm">
        <i class="ri-arrow-left-s-line text-2xl text-slate-700"></i>
      </button>
      <div class="flex-1">
        <h1 class="text-2xl lg:text-3xl font-bold text-slate-800" id="reader-title">{{ book?.title || '书名' }}</h1>
        <p class="text-slate-500 mt-1" id="reader-author">{{ book?.author || '作者' }}</p>
      </div>
    </div>
    
    <div class="flex flex-col xl:flex-row gap-8">
      <div class="xl:w-72 flex-shrink-0">
        <div class="bg-white rounded-2xl shadow-sm border border-slate-100 p-6 sticky top-8">
          <div class="flex items-center justify-between mb-5">
            <h3 class="font-bold text-slate-800 text-lg">章节列表</h3>
            <button @click="toggleChapterList" class="p-2 hover:bg-slate-100 rounded-xl xl:hidden">
              <i class="ri-menu-2-line text-lg text-slate-600"></i>
            </button>
          </div>
          <div id="chapter-list" class="space-y-1.5 max-h-[500px] overflow-y-auto scrollbar-thin" v-show="isChapterListOpen">
            <div v-if="isLoading" class="text-slate-400 text-sm text-center py-8">加载中...</div>
            <button 
              v-for="(chapter, index) in chapters" 
              :key="chapter.id"
              @click="loadChapter(index)"
              :class="[
                'w-full text-left px-4 py-3 rounded-xl transition-all text-sm',
                index === currentChapterIndex ? 'bg-indigo-50 text-indigo-700 font-medium' : 'hover:bg-slate-50 text-slate-700'
              ]"
            >
              {{ chapter.title }}
            </button>
            <div v-if="chapters.length === 0 && !isLoading" class="text-slate-400 text-sm text-center py-8">暂无章节</div>
          </div>
        </div>
      </div>
      
      <div class="flex-1">
        <div class="bg-white rounded-2xl shadow-sm border border-slate-100 p-6 lg:p-10">
          <div class="flex flex-col md:flex-row items-center justify-between mb-8 pb-6 border-b border-slate-200 gap-4">
            <div class="flex items-center gap-3" id="txt-controls">
              <button @click="prevChapter" :disabled="currentChapterIndex === 0" 
                class="p-3 hover:bg-slate-100 rounded-xl border border-slate-200">
                <i class="ri-arrow-left-s-line text-2xl text-slate-700"></i>
              </button>
              <span class="text-sm text-slate-500 font-medium">
                <span id="current-chapter" class="font-bold text-slate-700">{{ currentChapterIndex + 1 }}</span> / <span id="total-chapters" class="font-bold text-slate-700">{{ totalChapters }}</span>
              </span>
              <button @click="nextChapter" :disabled="currentChapterIndex === totalChapters - 1" 
                class="p-3 hover:bg-slate-100 rounded-xl border border-slate-200">
                <i class="ri-arrow-right-s-line text-2xl text-slate-700"></i>
              </button>
            </div>
            <div class="flex items-center gap-3">
              <button @click="zoomOut" class="p-3 hover:bg-slate-100 rounded-xl border border-slate-200">
                <i class="ri-subtract-line text-xl text-slate-700"></i>
              </button>
              <span class="text-sm text-slate-500 font-semibold" id="zoom-level">{{ zoomLevel }}%</span>
              <button @click="zoomIn" class="p-3 hover:bg-slate-100 rounded-xl border border-slate-200">
                <i class="ri-add-line text-xl text-slate-700"></i>
              </button>
            </div>
          </div>
          
          <div id="reader-content" class="prose prose-slate max-w-none leading-relaxed text-slate-800" :style="{ fontSize: `${zoomLevel}%` }">
            <div v-if="isLoading" class="text-slate-500 text-center py-16">加载中...</div>
            <div v-else-if="readerContent" class="chapter-html" v-html="readerContent"></div>
            <div v-else class="text-slate-500 text-center py-16">暂无内容</div>
          </div>
          
          <div class="flex items-center justify-center mt-10 pt-8 border-t border-slate-200" id="bottom-navigation">
            <div class="flex items-center gap-4" id="txt-bottom-controls">
              <button @click="prevChapter" :disabled="currentChapterIndex === 0" 
                class="px-8 py-3 bg-slate-100 hover:bg-slate-200 rounded-xl font-semibold transition-all text-slate-700">
                <i class="ri-arrow-left-line mr-2"></i>上一章
              </button>
              <button @click="nextChapter" :disabled="currentChapterIndex === totalChapters - 1" 
                class="px-8 py-3 btn-primary text-white rounded-xl font-semibold transition-all">
                下一章<i class="ri-arrow-right-line ml-2"></i>
              </button>
            </div>
          </div>
        </div>
        
        <div class="bg-white rounded-2xl shadow-sm border border-slate-100 p-6 lg:p-8 mt-8">
          <div class="flex items-center justify-between mb-6">
            <h3 class="font-bold text-xl text-slate-800" id="comments-title">评论</h3>
            <span id="comments-count" class="text-sm text-slate-500">{{ comments.length }} 条评论</span>
          </div>
          
          <div class="mb-6">
            <textarea v-model="commentContent" 
              :class="[canComment ? 'input-modern w-full px-5 py-4 bg-slate-50 border-slate-200 rounded-xl outline-none resize-none' : 'input-modern w-full px-5 py-4 bg-slate-100 border-slate-300 rounded-xl outline-none resize-none cursor-not-allowed']" 
              rows="3" 
              :placeholder="canComment ? '写下你的评论...' : '管理员已经关闭评论'"
              :disabled="!canComment"></textarea>
            <div class="flex justify-end mt-3">
              <button @click="submitComment" :disabled="!canComment"
                :class="[canComment ? 'btn-primary text-white px-6 py-2.5 rounded-xl font-semibold' : 'btn-primary text-white bg-slate-300 cursor-not-allowed px-6 py-2.5 rounded-xl font-semibold']">
                {{ canComment ? '发表评论' : '评论已关闭' }}
              </button>
            </div>
          </div>
          
          <div id="comments-list" class="space-y-4 max-h-[500px] overflow-y-auto scrollbar-thin">
            <div v-if="comments.length === 0" class="text-slate-400 text-sm text-center py-8">暂无评论</div>
            <div v-for="comment in comments" :key="comment.id" class="bg-slate-50 p-4 rounded-xl">
              <div class="flex items-start gap-3 mb-2">
                <div class="w-8 h-8 gradient-bg rounded-full flex items-center justify-center shadow-sm flex-shrink-0">
                  <i class="ri-user-3-line text-sm text-white"></i>
                </div>
                <div>
                  <div class="flex items-center gap-2 mb-1">
                    <p class="font-semibold text-slate-800 text-sm">{{ comment.username || '未知用户' }}</p>
                    <span class="text-xs text-slate-400">{{ new Date(comment.created_at).toLocaleString() }}</span>
                  </div>
                  <p class="text-slate-600 text-sm">{{ comment.content }}</p>
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