<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { useRouter } from 'vue-router';
import Toast from './Toast.vue';
import ConfirmDialog from './ConfirmDialog.vue';

const router = useRouter();

// 响应式数据
const books = ref<any[]>([]);
const isLoading = ref(false);
const searchKeyword = ref('');
const searchCategory = ref('');
const searchFormat = ref('');
const categories = ref<string[]>([]);
const showAddBookModal = ref(false);
const showToastMessage = ref('');
const showToastType = ref<'success' | 'error' | 'info'>('info');
const isToastVisible = ref(false);
const user = ref<any>(null);
const showConfirmDialog = ref(false);
const bookToDelete = ref<any>(null);
const showEditBookModal = ref(false);
const bookToEdit = ref<any>(null);
const editBookTitle = ref('');
const editBookAuthor = ref('');
const editBookISBN = ref('');
const editBookCategory = ref('');
const editBookDescription = ref('');
const settings = ref<any>({
  allow_contributor_edit_book: true,
  allow_contributor_delete_book: true,
  allow_comments: true,
  comment_review_enabled: false
});

// 表单数据
const bookFile = ref<File | null>(null);
const bookTitle = ref('');
const bookAuthor = ref('');
const bookISBN = ref('');
const bookCategory = ref('');
const bookDescription = ref('');
const bookFileInput = ref<HTMLInputElement | null>(null);

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

// 加载分类
const loadCategories = async () => {
  try {
    const response = await apiRequest('/books/categories');
    if (response.ok) {
      const data = await response.json();
      categories.value = data;
    }
  } catch (error) {
    console.error('Failed to load categories:', error);
  }
};

// 加载图书
const loadBooks = async () => {
  isLoading.value = true;
  try {
    const response = await apiRequest('/books');
    if (response.ok) {
      const result = await response.json();
      // console.log(result)
      books.value = result.items || result || [];
    }
  } catch (error) {
    console.error('Failed to load books:', error);
    showToast('加载图书失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 搜索图书
const searchBooks = async () => {
  // 只在搜索内容有变化时显示加载状态
  if (searchKeyword.value || searchCategory.value || searchFormat.value) {
    isLoading.value = true;
  }
  try {
    const response = await apiRequest('/books/search', {
      method: 'POST',
      body: JSON.stringify({
        keyword: searchKeyword.value || null,
        category: searchCategory.value || null,
        format: searchFormat.value || null
      }),
    });
    if (response.ok) {
      const result = await response.json();
      // 延迟更新图书列表，确保加载状态有足够时间显示
      setTimeout(() => {
        books.value = result.items || result || [];
        isLoading.value = false;
      }, 100);
    } else {
      isLoading.value = false;
    }
  } catch (error) {
    console.error('Failed to search books:', error);
    showToast('搜索失败', 'error');
    isLoading.value = false;
  }
};



// 重置搜索
const clearSearch = () => {
  searchKeyword.value = '';
  searchCategory.value = '';
  searchFormat.value = '';
  loadBooks();
};

// 打开添加图书模态框
const openAddBookModal = () => {
  showAddBookModal.value = true;
  // 重置表单
  bookFile.value = null;
  bookTitle.value = '';
  bookAuthor.value = '';
  bookISBN.value = '';
  bookCategory.value = '';
  bookDescription.value = '';
};

// 关闭添加图书模态框
const closeAddBookModal = () => {
  showAddBookModal.value = false;
};

// 处理文件选择
const handleFileSelect = (event: Event) => {
  const target = event.target as HTMLInputElement;
  if (target.files && target.files[0]) {
    bookFile.value = target.files[0];
  }
};

// 处理添加图书
const handleAddBook = async (event: Event) => {
  event.preventDefault();
  
  if (!bookFile.value) {
    showToast('请选择图书文件', 'error');
    return;
  }

  if (!bookTitle.value) {
    showToast('请输入书名', 'error');
    return;
  }

  isLoading.value = true;

  try {
    const formData = new FormData();
    formData.append('file', bookFile.value);
    formData.append('title', bookTitle.value);
    formData.append('author', bookAuthor.value || '');
    formData.append('description', bookDescription.value || '');
    formData.append('isbn', bookISBN.value || '');
    formData.append('category', bookCategory.value || '');

    const response = await apiRequest('/books', {
      method: 'POST',
      body: formData,
    });

    if (response.ok) {
      closeAddBookModal();
      loadBooks();
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

// 跳转到阅读器
const navigateToReader = (bookId: string) => {
  router.push(`/reader/${bookId}`);
};

// 加载用户信息
const loadUserInfo = async () => {
  try {
    // 使用正确的/auth/me端点获取用户信息
    const response = await apiRequest('/auth/me');
    if (response.ok) {
      const data = await response.json();
      user.value = data;
    }
  } catch (error) {
    console.error('Failed to load user info:', error);
  }
};

// 加载配置
const loadSettings = async () => {
  try {
    const response = await apiRequest('/settings');
    // console.log(response)
    if (response.ok) {
      try {
        const data = await response.json();
        // console.log(data);
        settings.value = {
          allow_contributor_edit_book: data.allow_uploader_edit !== false,
          allow_contributor_delete_book: data.allow_uploader_delete !== false,
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

// 检查用户是否有权限添加图书
const canAddBook = () => {
  if (!user.value) return false;
  const role = user.value.role;
  return role === 'root' || role === 'admin' || role === 'contributor';
};

// 检查用户是否有权限删除图书
const canDeleteBook = (book: any) => {
  if (!user.value) return false;
  const role = user.value.role;
  // 管理员和root可以删除任何图书
  if (role === 'root' || role === 'admin') {
    return true;
  }
  // 贡献者只能删除自己上传的图书，且需要配置允许
  if (role === 'contributor' && book.created_by === user.value.id && settings.value.allow_contributor_delete_book) {
    return true;
  }
  return false;
};

// 检查用户是否有权限修改图书
const canEditBook = (book: any) => {
  if (!user.value) return false;
  const role = user.value.role;
  // 管理员和root可以修改任何图书
  if (role === 'root' || role === 'admin') {
    return true;
  }
  // 贡献者只能修改自己上传的图书，且需要配置允许
  if (role === 'contributor' && book.created_by === user.value.id && settings.value.allow_contributor_edit_book) {
    return true;
  }
  return false;
};



// 打开修改图书模态框
const openEditBookModal = (book: any) => {
  bookToEdit.value = book;
  editBookTitle.value = book.title;
  editBookAuthor.value = book.author || '';
  editBookISBN.value = book.isbn || '';
  editBookCategory.value = book.category || '';
  editBookDescription.value = book.description || '';
  showEditBookModal.value = true;
};

// 关闭修改图书模态框
const closeEditBookModal = () => {
  showEditBookModal.value = false;
  bookToEdit.value = null;
  editBookTitle.value = '';
  editBookAuthor.value = '';
  editBookISBN.value = '';
  editBookCategory.value = '';
  editBookDescription.value = '';
};

// 提交修改图书
const submitEditBook = async (event: Event) => {
  event.preventDefault();
  
  if (!bookToEdit.value) return;
  
  if (!editBookTitle.value) {
    showToast('请输入书名', 'error');
    return;
  }

  isLoading.value = true;
  try {
    const response = await apiRequest(`/books/${bookToEdit.value.id}`, {
      method: 'PUT',
      body: JSON.stringify({
        title: editBookTitle.value,
        author: editBookAuthor.value,
        isbn: editBookISBN.value,
        category: editBookCategory.value,
        description: editBookDescription.value
      }),
    });

    if (response.ok) {
      showToast('图书修改成功！');
      closeEditBookModal();
      loadBooks();
    } else {
      showToast('修改失败', 'error');
    }
  } catch (error) {
    showToast('修改失败', 'error');
  } finally {
    isLoading.value = false;
  }
};

// 打开确认删除对话框
const openDeleteConfirm = (book: any) => {
  bookToDelete.value = book;
  showConfirmDialog.value = true;
};

// 关闭确认删除对话框
const closeDeleteConfirm = () => {
  showConfirmDialog.value = false;
  bookToDelete.value = null;
};

// 确认删除图书
const confirmDeleteBook = async () => {
  if (!bookToDelete.value) return;
  
  isLoading.value = true;
  try {
    const response = await apiRequest(`/books/${bookToDelete.value.id}`, {
      method: 'DELETE',
    });

    if (response.ok) {
      showToast('图书删除成功！');
      loadBooks();
    } else {
      showToast('删除失败', 'error');
    }
  } catch (error) {
    showToast('删除失败', 'error');
  } finally {
    isLoading.value = false;
    closeDeleteConfirm();
  }
};

// 跳转到评论页面
const navigateToComments = (bookId: string) => {
  router.push(`/books/${bookId}/comments`);
};

onMounted(() => {
  loadCategories();
  loadBooks();
  loadUserInfo();
  loadSettings();
});
</script>

<template>
  <div id="books-page" class="page">
    <div class="flex flex-col lg:flex-row lg:items-center lg:justify-between mb-8 gap-6">
      <div>
        <h1 class="text-3xl lg:text-4xl font-bold text-slate-800">图书管理</h1>
        <p class="text-slate-500 mt-2">探索和管理你的图书收藏</p>
      </div>
      <button v-if="canAddBook()" @click="openAddBookModal" class="btn-primary text-white px-6 py-3.5 rounded-xl font-semibold flex items-center gap-2.5">
        <i class="ri-add-circle-line text-xl"></i>
        <span>添加图书</span>
      </button>
    </div>
    
    <div class="bg-white rounded-2xl shadow-sm border border-slate-100 p-6 mb-8">
      <div class="flex flex-col lg:flex-row gap-4 items-stretch">
        <div class="flex-1">
          <div class="relative">
            <i class="ri-search-2-line absolute left-5 top-1/2 -translate-y-1/2 text-slate-400 text-xl"></i>
            <input type="text" v-model="searchKeyword" 
              class="input-modern w-full pl-14 pr-5 py-4 bg-slate-50 rounded-xl outline-none" 
              placeholder="搜索书名、作者、ISBN...">
          </div>
        </div>
        <div class="lg:w-40">
          <select v-model="searchCategory" 
            class="input-modern w-full px-5 py-4 bg-slate-50 rounded-xl outline-none appearance-none cursor-pointer">
            <option value="">所有分类</option>
            <option v-for="category in categories" :key="category" :value="category">{{ category }}</option>
          </select>
        </div>
        <div class="lg:w-40">
          <select v-model="searchFormat" 
            class="input-modern w-full px-5 py-4 bg-slate-50 rounded-xl outline-none appearance-none cursor-pointer">
            <option value="">所有格式</option>
            <option value="epub">EPUB</option>
            <option value="pdf">PDF</option>
            <option value="txt">TXT</option>
          </select>
        </div>
        <button @click="searchBooks" 
          class="btn-primary text-white px-6 py-4 rounded-xl font-semibold flex items-center gap-2 whitespace-nowrap">
          <i class="ri-search-2-line"></i>
          <span>搜索</span>
        </button>
        <button @click="clearSearch" 
          class="px-6 py-4 border-2 border-slate-200 hover:border-slate-300 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all whitespace-nowrap flex items-center gap-2">
          <i class="ri-refresh-line"></i>
          <span>重置</span>
        </button>
      </div>
    </div>
    
    <div v-if="isLoading" class="flex justify-center items-center py-20">
      <div class="spinner"></div>
    </div>
    
    <div v-else-if="books.length > 0" id="books-grid" class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 2xl:grid-cols-5 gap-6">
      <div v-for="book in books" :key="book.id" 
        class="bg-white rounded-2xl shadow-sm border border-slate-100 overflow-hidden card-hover cursor-pointer" 
        @click="navigateToReader(book.id)">
        <div class="h-64 book-cover flex items-center justify-center">
          <i class="ri-book-2-line text-6xl text-white/80"></i>
        </div>
        <div class="p-5">
          <h3 class="font-semibold text-lg text-slate-800 mb-2 truncate">{{ book.title }}</h3>
          <p class="text-slate-500 text-sm mb-3">{{ book.author || '未知作者' }}</p>
          <div class="flex items-center justify-between text-xs text-slate-400 mb-3">
            <span>{{ book.category || '未分类' }}</span>
            <span>{{ book.file_type ? book.file_type.toUpperCase() : '未知格式' }}</span>
          </div>
          <div class="flex items-center justify-end gap-2">
            <button @click.stop="navigateToComments(book.id)" 
              class="p-2 hover:bg-slate-100 rounded-xl text-slate-600 transition-all" 
              title="评论">
              <i class="ri-message-3-line text-lg"></i>
            </button>
            <button v-if="canEditBook(book)" @click.stop="openEditBookModal(book)" 
              class="p-2 hover:bg-slate-100 rounded-xl text-indigo-500 transition-all" 
              title="修改">
              <i class="ri-edit-line text-lg"></i>
            </button>
            <button v-if="canDeleteBook(book)" @click.stop="openDeleteConfirm(book)" 
              class="p-2 hover:bg-slate-100 rounded-xl text-rose-500 transition-all" 
              title="删除">
              <i class="ri-delete-bin-line text-lg"></i>
            </button>
          </div>
        </div>
      </div>
    </div>
    
    <div v-else id="empty-books" class="text-center py-20">
      <div class="inline-flex items-center justify-center w-28 h-28 bg-slate-100 rounded-full mb-6">
        <i class="ri-book-line text-5xl text-slate-400"></i>
      </div>
      <h3 class="text-2xl font-semibold text-slate-800 mb-3">还没有图书</h3>
      <p class="text-slate-500 mb-6">点击上方按钮添加你的第一本图书</p>
    </div>
    
    <!-- 添加图书模态框 -->
    <div id="add-book-modal" class="modal" :class="{ show: showAddBookModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-xl mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">添加图书</h2>
          <button @click="closeAddBookModal" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="handleAddBook" class="space-y-5">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">图书文件 *</label>
            <div class="border-3 border-dashed border-slate-200 hover:border-indigo-400 rounded-2xl p-8 text-center transition-all cursor-pointer" @click="bookFileInput?.click()">
              <input type="file" ref="bookFileInput" accept=".epub,.md,.markdown,.txt" class="hidden" @change="handleFileSelect">
              <div id="file-preview">
                <i v-if="!bookFile" class="ri-upload-cloud-2-line text-5xl text-slate-400 mb-3"></i>
                <i v-else class="ri-file-book-line text-5xl" style="background: var(--primary-gradient); -webkit-background-clip: text; -webkit-text-fill-color: transparent; background-clip: text; margin-bottom: 0.75rem;"></i>
                <p v-if="!bookFile" class="text-slate-600 font-medium text-base">点击或拖拽上传文件</p>
                <p v-else class="text-slate-700 font-semibold text-lg">{{ bookFile.name }}</p>
                <p v-if="!bookFile" class="text-xs text-slate-400 mt-2">支持 epub, markdown, txt</p>
                <p v-else class="text-xs text-slate-400 mt-2 font-medium">{{ (bookFile.size / 1024 / 1024).toFixed(2) }} MB</p>
              </div>
            </div>
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">书名 *</label>
            <input type="text" v-model="bookTitle" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入书名">
          </div>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-semibold text-slate-700 mb-2">作者</label>
              <input type="text" v-model="bookAuthor" 
                class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
                placeholder="请输入作者">
            </div>
            <div>
              <label class="block text-sm font-semibold text-slate-700 mb-2">ISBN</label>
              <input type="text" v-model="bookISBN" 
                class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
                placeholder="请输入ISBN">
            </div>
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">分类</label>
            <input type="text" v-model="bookCategory" 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入分类（如：文学、科幻等）">
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">描述</label>
            <textarea v-model="bookDescription" rows="4" 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none resize-none" 
              placeholder="请输入图书描述"></textarea>
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="closeAddBookModal" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '添加中...' : '添加' }}
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
    
    <!-- 确认删除对话框 -->
    <ConfirmDialog 
        :show="showConfirmDialog"
        :title="'删除图书'"
        :message="`确定要删除图书 ${bookToDelete?.title} 吗？`"
        :confirm-text="'确定'"
        :cancel-text="'取消'"
        @confirm="confirmDeleteBook"
        @cancel="closeDeleteConfirm"
    />
    
    <!-- 修改图书模态框 -->
    <div id="edit-book-modal" class="modal" :class="{ show: showEditBookModal }">
      <div class="modal-content bg-white rounded-3xl shadow-2xl w-full max-w-xl mx-4 p-8">
        <div class="flex items-center justify-between mb-7">
          <h2 class="text-2xl font-bold text-slate-800">修改图书</h2>
          <button @click="closeEditBookModal" class="p-2.5 hover:bg-slate-100 rounded-xl">
            <i class="ri-close-line text-2xl text-slate-500"></i>
          </button>
        </div>
        <form @submit="submitEditBook" class="space-y-5">
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">书名 *</label>
            <input type="text" v-model="editBookTitle" required 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入书名">
          </div>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <label class="block text-sm font-semibold text-slate-700 mb-2">作者</label>
              <input type="text" v-model="editBookAuthor" 
                class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
                placeholder="请输入作者">
            </div>
            <div>
              <label class="block text-sm font-semibold text-slate-700 mb-2">ISBN</label>
              <input type="text" v-model="editBookISBN" 
                class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
                placeholder="请输入ISBN">
            </div>
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">分类</label>
            <input type="text" v-model="editBookCategory" 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none" 
              placeholder="请输入分类（如：文学、科幻等）">
          </div>
          <div>
            <label class="block text-sm font-semibold text-slate-700 mb-2">描述</label>
            <textarea v-model="editBookDescription" rows="4" 
              class="input-modern w-full px-5 py-3.5 bg-slate-50 border-slate-200 rounded-xl outline-none resize-none" 
              placeholder="请输入图书描述"></textarea>
          </div>
          <div class="flex gap-4 pt-5">
            <button type="button" @click="closeEditBookModal" 
              class="flex-1 px-6 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">取消</button>
            <button type="submit" :disabled="isLoading" 
              class="flex-1 btn-primary text-white px-6 py-3.5 rounded-xl font-semibold">
              {{ isLoading ? '修改中...' : '修改' }}
            </button>
          </div>
        </form>
      </div>
    </div>
  </div>
</template>