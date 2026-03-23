<script setup lang="ts">
import { onMounted } from 'vue';
import { useRouter } from 'vue-router';

const router = useRouter();

onMounted(() => {
  const token = localStorage.getItem('token');
  if (token) {
    router.push('/');
  } else {
    router.push('/login');
  }
});
</script>

<template>
  <router-view />
</template>

<style>
/* 全局样式 */
@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+SC:wght@300;400;500;600;700&display=swap');

* {
  font-family: 'Noto Sans SC', sans-serif;
}

:root {
  --primary-gradient: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  --secondary-gradient: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
  --success-gradient: linear-gradient(135deg, #11998e 0%, #38ef7d 100%);
  --dark-gradient: linear-gradient(135deg, #2d3436 0%, #000000 100%);
}

.gradient-bg {
  background: var(--primary-gradient);
}

.glass-effect {
  background: rgba(255, 255, 255, 0.95);
  backdrop-filter: blur(20px);
  -webkit-backdrop-filter: blur(20px);
}

.card-hover {
  transition: all 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

.card-hover:hover {
  transform: translateY(-8px) scale(1.02);
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.15);
}

.sidebar-item {
  transition: all 0.3s ease;
  position: relative;
  overflow: hidden;
}

.sidebar-item::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  height: 100%;
  width: 4px;
  background: var(--primary-gradient);
  transform: scaleY(0);
  transition: transform 0.3s ease;
}

.sidebar-item:hover::before,
.sidebar-item.active::before {
  transform: scaleY(1);
}

.sidebar-item:hover,
.sidebar-item.active {
  background: rgba(102, 126, 234, 0.08);
}

.modal {
  display: none;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  z-index: 1000;
  align-items: center;
  justify-content: center;
  animation: fadeIn 0.3s ease;
}

.modal.show {
  display: flex;
}

.modal-content {
  animation: slideUp 0.4s cubic-bezier(0.175, 0.885, 0.32, 1.275);
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

@keyframes slideUp {
  from { 
    opacity: 0;
    transform: translateY(30px) scale(0.95);
  }
  to { 
    opacity: 1;
    transform: translateY(0) scale(1);
  }
}

@keyframes pulse {
  0%, 100% { transform: scale(1); }
  50% { transform: scale(1.05); }
}

.book-cover {
  background: var(--secondary-gradient);
}

.btn-primary {
  background: var(--primary-gradient);
  transition: all 0.3s ease;
}

.btn-primary:hover {
  transform: translateY(-2px);
  box-shadow: 0 10px 20px rgba(102, 126, 234, 0.4);
}

.spinner {
  border: 3px solid rgba(255, 255, 255, 0.3);
  border-top: 3px solid white;
  border-radius: 50%;
  width: 24px;
  height: 24px;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

#epub-viewer {
  background: white;
  border-radius: 16px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.08);
}

#epub-viewer iframe {
  border: none;
  width: 100%;
  height: 70vh;
}

.switch {
  position: relative;
  display: inline-block;
  width: 56px;
  height: 30px;
}

.switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: linear-gradient(135deg, #e2e8f0 0%, #cbd5e1 100%);
  transition: 0.3s;
  border-radius: 30px;
  box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.1);
}

.slider:before {
  position: absolute;
  content: "";
  height: 22px;
  width: 22px;
  left: 4px;
  bottom: 4px;
  background-color: white;
  transition: 0.3s cubic-bezier(0.175, 0.885, 0.32, 1.275);
  border-radius: 50%;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

input:checked + .slider {
  background: var(--primary-gradient);
}

input:checked + .slider:before {
  transform: translateX(26px);
}

input:focus + .slider {
  box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.3);
}

.chapter-html {
  line-height: 1.9;
}

.chapter-html p {
  margin-bottom: 1.5rem;
  text-indent: 2em;
}

.chapter-html h1,
.chapter-html h2,
.chapter-html h3 {
  margin-top: 2rem;
  margin-bottom: 1rem;
  font-weight: 700;
}

.chapter-html img {
  max-width: 100%;
  height: auto;
  border-radius: 12px;
  margin: 1.5rem auto;
  display: block;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.08);
}

.chapter-html ul,
.chapter-html ol {
  margin: 1rem 0;
  padding-left: 2rem;
}

.chapter-html li {
  margin-bottom: 0.5rem;
}

.chapter-html blockquote {
  border-left: 4px solid #667eea;
  padding-left: 1.25rem;
  margin: 1.5rem 0;
  color: #6b7280;
  font-style: italic;
  background: linear-gradient(90deg, rgba(102, 126, 234, 0.05) 0%, transparent 100%);
  padding-top: 0.75rem;
  padding-bottom: 0.75rem;
  border-radius: 0 8px 8px 0;
}

.chapter-html table {
  width: 100%;
  border-collapse: collapse;
  margin: 1.5rem 0;
  font-size: 0.95rem;
  border-radius: 12px;
  overflow: hidden;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.06);
}

.chapter-html thead {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
}

.chapter-html th {
  padding: 1rem 1.25rem;
  text-align: left;
  font-weight: 600;
  letter-spacing: 0.025em;
}

.chapter-html td {
  padding: 0.875rem 1.25rem;
  border-bottom: 1px solid #e5e7eb;
  color: #4b5563;
}

.chapter-html tbody tr {
  background: white;
  transition: background-color 0.2s ease;
}

.chapter-html tbody tr:hover {
  background-color: #f9fafb;
}

.chapter-html tbody tr:last-child td {
  border-bottom: none;
}

.chapter-html a {
  color: #2563eb;
  text-decoration: underline;
  transition: color 0.2s ease;
}

.chapter-html a:hover {
  color: #1d4ed8;
  text-decoration: none;
}

.input-modern {
  transition: all 0.3s ease;
  border: 2px solid #e5e7eb;
}

.input-modern:focus {
  border-color: #667eea;
  box-shadow: 0 0 0 4px rgba(102, 126, 234, 0.15);
}

.scrollbar-thin::-webkit-scrollbar {
  width: 6px;
}

.scrollbar-thin::-webkit-scrollbar-track {
  background: #f1f1f1;
  border-radius: 3px;
}

.scrollbar-thin::-webkit-scrollbar-thumb {
  background: linear-gradient(180deg, #667eea 0%, #764ba2 100%);
  border-radius: 3px;
}

.scrollbar-thin::-webkit-scrollbar-thumb:hover {
  background: linear-gradient(180deg, #5a67d8 0%, #6b46c1 100%);
}

@media (max-width: 768px) {
  .sidebar-mobile {
    transform: translateX(-100%);
    transition: transform 0.3s ease;
  }
  
  .sidebar-mobile.open {
    transform: translateX(0);
  }
}
</style>
