<script setup lang="ts">
defineProps({
  message: {
    type: String,
    required: true
  },
  type: {
    type: String,
    default: 'info',
    validator: (value: string) => ['success', 'error', 'info'].includes(value)
  },
  show: {
    type: Boolean,
    default: false
  }
});
</script>

<template>
  <transition name="toast">
    <div v-show="show" :class="[
        'fixed top-4 right-4 z-50 px-6 py-4 rounded-lg shadow-lg flex items-center gap-3 transition-all duration-300 transform opacity-100',
        type === 'success' ? 'bg-emerald-50 border border-emerald-200 text-emerald-800' :
        type === 'error' ? 'bg-rose-50 border border-rose-200 text-rose-800' :
        'bg-blue-50 border border-blue-200 text-blue-800'
    ]">
        <i :class="[
            'text-xl',
            type === 'success' ? 'ri-check-circle-fill text-emerald-500' :
            type === 'error' ? 'ri-error-warning-fill text-rose-500' :
            'ri-info-line text-blue-500'
        ]"></i>
        <span class="font-medium">{{ message }}</span>
    </div>
  </transition>
</template>

<style scoped>
.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>