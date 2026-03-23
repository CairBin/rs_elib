import { createRouter, createWebHistory } from 'vue-router';
import Login from '../components/login.vue';

// 懒加载组件
const MainApp = () => import('../components/MainApp.vue');
const BooksPage = () => import('../components/BooksPage.vue');
const GroupsPage = () => import('../components/GroupsPage.vue');
const GroupDetailPage = () => import('../components/GroupDetailPage.vue');
const UsersPage = () => import('../components/UsersPage.vue');
const SettingsPage = () => import('../components/SettingsPage.vue');
const ReviewsPage = () => import('../components/ReviewsPage.vue');
const ProfilePage = () => import('../components/ProfilePage.vue');
const ReaderPage = () => import('../components/ReaderPage.vue');
const BookCommentsPage = () => import('../components/BookCommentsPage.vue');
const NotFoundPage = () => import('../components/NotFoundPage.vue');

const routes = [
  {
    path: '/login',
    name: 'Login',
    component: Login
  },
  {
    path: '/',
    name: 'MainApp',
    component: MainApp,
    meta: { requiresAuth: true },
    children: [
      {
        path: '',
        name: 'Books',
        component: BooksPage
      },
      {
        path: 'groups',
        name: 'Groups',
        component: GroupsPage
      },
      {
        path: 'groups/:id',
        name: 'GroupDetail',
        component: GroupDetailPage
      },

      {
        path: 'users',
        name: 'Users',
        component: UsersPage,
        meta: { requiresAdmin: true }
      },
      {
        path: 'settings',
        name: 'Settings',
        component: SettingsPage,
        meta: { requiresAdmin: true }
      },
      {
        path: 'reviews',
        name: 'Reviews',
        component: ReviewsPage,
        meta: { requiresAdmin: true }
      },
      {
        path: 'profile',
        name: 'Profile',
        component: ProfilePage
      },
      {
        path: 'reader/:id',
        name: 'Reader',
        component: ReaderPage
      },
      {
        path: 'books/:id/comments',
        name: 'BookComments',
        component: BookCommentsPage
      }
    ]
  },
  // 404页面
  {
    path: '/:pathMatch(.*)*',
    name: 'NotFound',
    component: NotFoundPage
  }
];

const router = createRouter({
  history: createWebHistory(),
  routes
});

// 路由守卫
router.beforeEach((to, _from) => {
  const token = localStorage.getItem('token');
  
  // 检查是否需要认证
  if (to.meta.requiresAuth && !token) {
    return '/login';
  }
});

export default router;