<template>
  <n-config-provider :theme="currentTheme">
    <n-message-provider>
      <n-notification-provider>
        <n-dialog-provider>
          <n-layout has-sider style="height: 100vh">
            <!-- Activity Bar (Left Icon Bar) -->
            <n-layout-sider
              bordered
              :width="48"
              :collapsed-width="48"
              style="height: 100vh"
              class="activity-bar"
            >
              <div class="activity-bar-content">
                <div class="activity-icons">
                  <n-tooltip
                    v-for="item in activityItems"
                    :key="item.key"
                    placement="right"
                  >
                    <template #trigger>
                      <div
                        :class="['activity-icon', { active: currentRoute === item.key }]"
                        @click="handleMenuSelect(item.key)"
                      >
                        <n-icon size="24">
                          <component :is="item.icon" />
                        </n-icon>
                      </div>
                    </template>
                    {{ item.label }}
                  </n-tooltip>
                </div>
                <div class="activity-icons-bottom">
                  <n-tooltip placement="right">
                    <template #trigger>
                      <div class="activity-icon" @click="toggleTheme">
                        <n-icon size="24">
                          <component :is="isDark ? SunIcon : MoonIcon" />
                        </n-icon>
                      </div>
                    </template>
                    切换主题
                  </n-tooltip>
                </div>
              </div>
            </n-layout-sider>

            <!-- Main Content -->
            <n-layout style="height: 100vh">
              <n-layout-header
                bordered
                style="
                  height: 40px;
                  padding: 0 16px;
                  display: flex;
                  align-items: center;
                  justify-content: space-between;
                "
              >
                <n-text strong style="font-size: 14px">PostgreSQL Database Tool</n-text>
              </n-layout-header>

              <n-layout-content style="height: calc(100vh - 40px); overflow: hidden">
                <router-view />
              </n-layout-content>
            </n-layout>
          </n-layout>
        </n-dialog-provider>
      </n-notification-provider>
    </n-message-provider>
  </n-config-provider>
</template>

<script setup lang="ts">
import { computed, h } from 'vue';
import { useRoute, useRouter } from 'vue-router';
import {
  NConfigProvider,
  NMessageProvider,
  NNotificationProvider,
  NDialogProvider,
  NLayout,
  NLayoutSider,
  NLayoutHeader,
  NLayoutContent,
  NText,
  NIcon,
  NTooltip,
} from 'naive-ui';
import { useTheme } from '@/composables/useTheme';

// Icons
const HomeIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z' }),
  ]);

const DatabaseIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M12 3C7.58 3 4 4.79 4 7s3.58 4 8 4 8-1.79 8-4-3.58-4-8-4zM4 9v3c0 2.21 3.58 4 8 4s8-1.79 8-4V9c0 2.21-3.58 4-8 4s-8-1.79-8-4zm0 5v3c0 2.21 3.58 4 8 4s8-1.79 8-4v-3c0 2.21-3.58 4-8 4s-8-1.79-8-4z',
    }),
  ]);

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

const SettingsIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58c.18-.14.23-.41.12-.61l-1.92-3.32c-.12-.22-.37-.29-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54c-.04-.24-.24-.41-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58c-.18.14-.23.41-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z',
    }),
  ]);

const MoonIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', { d: 'M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z' }),
  ]);

const SunIcon = () =>
  h('svg', { viewBox: '0 0 24 24', fill: 'currentColor' }, [
    h('path', {
      d: 'M12 7c-2.76 0-5 2.24-5 5s2.24 5 5 5 5-2.24 5-5-2.24-5-5-5zM2 13h2c.55 0 1-.45 1-1s-.45-1-1-1H2c-.55 0-1 .45-1 1s.45 1 1 1zm18 0h2c.55 0 1-.45 1-1s-.45-1-1-1h-2c-.55 0-1 .45-1 1s.45 1 1 1zM11 2v2c0 .55.45 1 1 1s1-.45 1-1V2c0-.55-.45-1-1-1s-1 .45-1 1zm0 18v2c0 .55.45 1 1 1s1-.45 1-1v-2c0-.55-.45-1-1-1s-1 .45-1 1zM5.99 4.58c-.39-.39-1.03-.39-1.41 0-.39.39-.39 1.03 0 1.41l1.06 1.06c.39.39 1.03.39 1.41 0s.39-1.03 0-1.41L5.99 4.58zm12.37 12.37c-.39-.39-1.03-.39-1.41 0-.39.39-.39 1.03 0 1.41l1.06 1.06c.39.39 1.03.39 1.41 0 .39-.39.39-1.03 0-1.41l-1.06-1.06zm1.06-10.96c.39-.39.39-1.03 0-1.41-.39-.39-1.03-.39-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06zM7.05 18.36c.39-.39.39-1.03 0-1.41-.39-.39-1.03-.39-1.41 0l-1.06 1.06c-.39.39-.39 1.03 0 1.41s1.03.39 1.41 0l1.06-1.06z',
    }),
  ]);

const route = useRoute();
const router = useRouter();
const { currentTheme, isDark, toggleTheme } = useTheme();

const currentRoute = computed(() => route.name as string);

const activityItems = [
  {
    label: '首页',
    key: 'Home',
    icon: HomeIcon,
  },
  {
    label: '数据库资源管理器',
    key: 'Explorer',
    icon: DatabaseIcon,
  },
  {
    label: '数据库导出',
    key: 'Export',
    icon: ExportIcon,
  },
  {
    label: '数据库导入',
    key: 'Import',
    icon: ImportIcon,
  },
  {
    label: '配置',
    key: 'Settings',
    icon: SettingsIcon,
  },
];

const handleMenuSelect = (key: string) => {
  router.push({ name: key });
};
</script>

<style>
@import './assets/styles/global.css';

.activity-bar {
  background: var(--n-color);
}

.activity-bar-content {
  display: flex;
  flex-direction: column;
  height: 100%;
  justify-content: space-between;
}

.activity-icons,
.activity-icons-bottom {
  display: flex;
  flex-direction: column;
}

.activity-icon {
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  position: relative;
  transition: background-color 0.2s;
}

.activity-icon:hover {
  background-color: var(--n-item-color-hover);
}

.activity-icon.active {
  background-color: var(--n-item-color-active);
}

.activity-icon.active::before {
  content: '';
  position: absolute;
  left: 0;
  top: 0;
  bottom: 0;
  width: 2px;
  background-color: var(--n-color-target);
}
</style>
