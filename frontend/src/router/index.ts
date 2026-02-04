/**
 * Vue Router configuration
 */

import { createRouter, createWebHistory } from 'vue-router';

const routes = [
  {
    path: '/',
    name: 'Home',
    component: () => import('@/views/Home.vue'),
  },
  {
    path: '/export',
    name: 'Export',
    component: () => import('@/views/DatabaseExport.vue'),
  },
  {
    path: '/import',
    name: 'Import',
    component: () => import('@/views/DatabaseImport.vue'),
  },
  {
    path: '/explorer',
    name: 'Explorer',
    component: () => import('@/views/DatabaseExplorer.vue'),
  },
  {
    path: '/settings',
    name: 'Settings',
    component: () => import('@/views/Settings.vue'),
  },
];

export const router = createRouter({
  history: createWebHistory(),
  routes,
});
