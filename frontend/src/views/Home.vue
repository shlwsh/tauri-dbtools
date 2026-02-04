<template>
  <div class="home-view">
    <n-space vertical :size="24">
      <!-- Welcome Card -->
      <n-card>
        <n-space vertical :size="16">
          <n-text style="font-size: 24px" strong>PostgreSQL Database Tool</n-text>
          <n-text depth="3">
            一个现代化的 PostgreSQL 数据库管理工具，支持数据库导入导出、数据浏览和管理。
          </n-text>
        </n-space>
      </n-card>

      <!-- Quick Actions -->
      <n-card title="快速操作">
        <n-grid :cols="2" :x-gap="16" :y-gap="16">
          <n-gi>
            <n-card
              hoverable
              @click="$router.push('/export')"
              style="cursor: pointer; height: 100%"
            >
              <n-space vertical align="center">
                <n-icon size="48" color="#18a058">
                  <ExportIcon />
                </n-icon>
                <n-text strong>数据库导出</n-text>
                <n-text depth="3" style="text-align: center">
                  将数据库导出为备份文件
                </n-text>
              </n-space>
            </n-card>
          </n-gi>

          <n-gi>
            <n-card
              hoverable
              @click="$router.push('/import')"
              style="cursor: pointer; height: 100%"
            >
              <n-space vertical align="center">
                <n-icon size="48" color="#2080f0">
                  <ImportIcon />
                </n-icon>
                <n-text strong>数据库导入</n-text>
                <n-text depth="3" style="text-align: center">
                  从备份文件恢复数据库
                </n-text>
              </n-space>
            </n-card>
          </n-gi>

          <n-gi>
            <n-card
              hoverable
              @click="$router.push('/explorer')"
              style="cursor: pointer; height: 100%"
            >
              <n-space vertical align="center">
                <n-icon size="48" color="#f0a020">
                  <ExplorerIcon />
                </n-icon>
                <n-text strong>数据库资源管理器</n-text>
                <n-text depth="3" style="text-align: center">
                  浏览和管理数据库表数据
                </n-text>
              </n-space>
            </n-card>
          </n-gi>

          <n-gi>
            <n-card
              hoverable
              @click="$router.push('/settings')"
              style="cursor: pointer; height: 100%"
            >
              <n-space vertical align="center">
                <n-icon size="48" color="#d03050">
                  <SettingsIcon />
                </n-icon>
                <n-text strong>配置管理</n-text>
                <n-text depth="3" style="text-align: center">
                  管理数据库连接配置
                </n-text>
              </n-space>
            </n-card>
          </n-gi>
        </n-grid>
      </n-card>

      <!-- Features -->
      <n-card title="功能特性">
        <n-list>
          <n-list-item>
            <n-thing>
              <template #header>
                <n-text>🚀 现代化界面</n-text>
              </template>
              <template #description>
                基于 Vue3 和 Naive UI 构建，提供流畅的用户体验
              </template>
            </n-thing>
          </n-list-item>

          <n-list-item>
            <n-thing>
              <template #header>
                <n-text>🎨 主题切换</n-text>
              </template>
              <template #description>
                支持亮色和暗色主题，适应不同使用环境
              </template>
            </n-thing>
          </n-list-item>

          <n-list-item>
            <n-thing>
              <template #header>
                <n-text>🔧 多连接管理</n-text>
              </template>
              <template #description>
                支持管理多个数据库连接配置，快速切换
              </template>
            </n-thing>
          </n-list-item>

          <n-list-item>
            <n-thing>
              <template #header>
                <n-text>📊 数据管理</n-text>
              </template>
              <template #description>
                直接在应用中浏览和编辑数据库表数据
              </template>
            </n-thing>
          </n-list-item>
        </n-list>
      </n-card>

      <!-- Connection Status -->
      <n-card title="连接状态">
        <n-space vertical>
          <n-space align="center">
            <n-text>已配置连接数：</n-text>
            <n-tag type="info">{{ connections.length }}</n-tag>
          </n-space>
          <n-space align="center" v-if="defaultConnection">
            <n-text>默认连接：</n-text>
            <n-tag type="success">{{ defaultConnection.name }}</n-tag>
          </n-space>
          <n-button v-if="connections.length === 0" type="primary" @click="$router.push('/settings')">
            配置第一个连接
          </n-button>
        </n-space>
      </n-card>
    </n-space>
  </div>
</template>

<script setup lang="ts">
import { h } from 'vue';
import {
  NCard,
  NSpace,
  NText,
  NIcon,
  NGrid,
  NGi,
  NList,
  NListItem,
  NThing,
  NTag,
  NButton,
} from 'naive-ui';
import { useConfig } from '@/composables/useConfig';

// Icons
const ExportIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M9 16h6v-6h4l-7-7-7 7h4zm-4 2h14v2H5z',
    }),
  ]);

const ImportIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M9 3L5 6.99h3V14h2V6.99h3L9 3zm7 14.01V10h-2v7.01h-3L15 21l4-3.99h-3z',
    }),
  ]);

const ExplorerIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M12 3C7.58 3 4 4.79 4 7s3.58 4 8 4 8-1.79 8-4-3.58-4-8-4zM4 9v3c0 2.21 3.58 4 8 4s8-1.79 8-4V9c0 2.21-3.58 4-8 4s-8-1.79-8-4zm0 5v3c0 2.21 3.58 4 8 4s8-1.79 8-4v-3c0 2.21-3.58 4-8 4s-8-1.79-8-4z',
    }),
  ]);

const SettingsIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z',
    }),
  ]);

const { connections, defaultConnection } = useConfig();
</script>

<style scoped>
.home-view {
  padding: 24px;
  max-width: 1200px;
  margin: 0 auto;
}
</style>
