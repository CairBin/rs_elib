<script setup lang="ts">
import { useRouter } from 'vue-router';

const router = useRouter();

const goBack = () => {
  router.go(-1);
};

const goHome = () => {
  router.push('/');
};
</script>

<template>
  <div id="not-found-page" class="page flex flex-col items-center justify-center min-h-[80vh] py-16">
    <div class="text-center max-w-2xl mx-auto">
      <div class="inline-flex items-center justify-center w-32 h-32 bg-slate-100 rounded-full mb-8">
        <i class="ri-error-warning-line text-6xl text-slate-400"></i>
      </div>
      <h1 class="text-5xl lg:text-6xl font-bold text-slate-800 mb-6">404</h1>
      <h2 class="text-2xl lg:text-3xl font-semibold text-slate-700 mb-4">页面未找到</h2>
      <p class="text-slate-500 mb-8">
        抱歉，您访问的页面不存在或已被移除。
      </p>
      <div class="flex flex-col sm:flex-row gap-4 justify-center">
        <button @click="goBack" 
          class="px-8 py-3.5 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">
          返回上一页
        </button>
        <button @click="goHome" 
          class="btn-primary text-white px-8 py-3.5 rounded-xl font-semibold transition-all">
          回到首页
        </button>
      </div>
    </div>
  </div>
</template>