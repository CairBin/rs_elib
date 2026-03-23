<script setup lang="ts">
defineProps({
  show: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: '确认操作'
  },
  message: {
    type: String,
    default: '确定要执行此操作吗？'
  },
  confirmText: {
    type: String,
    default: '确定'
  },
  cancelText: {
    type: String,
    default: '取消'
  }
});

const emit = defineEmits(['confirm', 'cancel']);

const handleConfirm = () => {
  emit('confirm');
};

const handleCancel = () => {
  emit('cancel');
};
</script>

<template>
  <transition name="modal">
    <div v-if="show" class="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
      <div class="bg-white rounded-2xl shadow-2xl w-full max-w-md">
        <div class="p-6">
          <h3 class="text-xl font-bold text-slate-800 mb-4">{{ title }}</h3>
          <p class="text-slate-600 mb-6">{{ message }}</p>
          <div class="flex gap-4">
            <button @click="handleCancel" 
              class="flex-1 px-6 py-3 border-2 border-slate-200 hover:bg-slate-50 rounded-xl font-semibold text-slate-700 transition-all">
              {{ cancelText }}
            </button>
            <button @click="handleConfirm" 
              class="flex-1 px-6 py-3 bg-rose-600 hover:bg-rose-700 text-white rounded-xl font-semibold transition-all">
              {{ confirmText }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </transition>
</template>

<style scoped>
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
  transform: scale(0.95);
}
</style>