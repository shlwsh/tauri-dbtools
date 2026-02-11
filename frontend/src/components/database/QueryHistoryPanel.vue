<template>
  <div class="query-history-panel">
    <!-- Search -->
    <div class="search-bar">
      <n-input
        v-model:value="searchText"
        placeholder="Search query history..."
        clearable
      >
        <template #prefix>
          <n-icon><SearchOutline /></n-icon>
        </template>
      </n-input>
    </div>

    <!-- History List -->
    <div class="history-list">
      <div v-if="filteredHistory.length === 0" class="empty-state">
        <n-empty description="No query history found" />
      </div>

      <div
        v-for="item in filteredHistory"
        :key="item.id"
        class="history-item"
        @click="handleLoadQuery(item)"
      >
        <div class="history-item-header">
          <n-tag
            :type="item.success ? 'success' : 'error'"
            size="small"
          >
            {{ item.success ? 'Success' : 'Failed' }}
          </n-tag>
          <span class="history-item-time">
            {{ formatTime(item.executedAt) }}
          </span>
        </div>

        <div class="history-item-query">
          {{ truncateQuery(item.query) }}
        </div>

        <div class="history-item-footer">
          <n-space size="small">
            <n-tag
              size="tiny"
              type="info"
            >
              {{ item.database }}
            </n-tag>
            <n-tag size="tiny">
              {{ item.duration }}ms
            </n-tag>
          </n-space>

          <n-button
            text
            size="tiny"
            type="error"
            @click.stop="handleDeleteItem(item.id)"
          >
            <template #icon>
              <n-icon><TrashOutline /></n-icon>
            </template>
          </n-button>
        </div>
      </div>
    </div>

    <!-- Actions -->
    <div class="history-actions">
      <n-button
        block
        type="error"
        :disabled="filteredHistory.length === 0"
        @click="handleClearHistory"
      >
        Clear All History
      </n-button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import {
  NInput,
  NIcon,
  NEmpty,
  NTag,
  NSpace,
  NButton,
  useDialog,
} from 'naive-ui';
import { SearchOutline, TrashOutline } from '@vicons/ionicons5';
import { useSQLEditorStore } from '@/stores/sql-editor';
import type { QueryHistoryItem } from '@/types/sql-editor';

// Emits
const emit = defineEmits<{
  loadQuery: [query: string];
}>();

// Store
const sqlEditorStore = useSQLEditorStore();
const dialog = useDialog();

// State
const searchText = ref('');

// Computed
const filteredHistory = computed(() => {
  if (!searchText.value.trim()) {
    return sqlEditorStore.queryHistory;
  }
  
  return sqlEditorStore.searchHistory(searchText.value);
});

// Methods
const formatTime = (date: Date): string => {
  const now = new Date();
  const diff = now.getTime() - date.getTime();
  
  // Less than 1 minute
  if (diff < 60000) {
    return 'Just now';
  }
  
  // Less than 1 hour
  if (diff < 3600000) {
    const minutes = Math.floor(diff / 60000);
    return `${minutes} minute${minutes > 1 ? 's' : ''} ago`;
  }
  
  // Less than 1 day
  if (diff < 86400000) {
    const hours = Math.floor(diff / 3600000);
    return `${hours} hour${hours > 1 ? 's' : ''} ago`;
  }
  
  // Format as date
  return date.toLocaleString();
};

const truncateQuery = (query: string): string => {
  const maxLength = 100;
  
  if (query.length <= maxLength) {
    return query;
  }
  
  return query.substring(0, maxLength) + '...';
};

const handleLoadQuery = (item: QueryHistoryItem) => {
  emit('loadQuery', item.query);
};

const handleDeleteItem = (itemId: string) => {
  dialog.warning({
    title: 'Delete History Item',
    content: 'Are you sure you want to delete this history item?',
    positiveText: 'Delete',
    negativeText: 'Cancel',
    onPositiveClick: () => {
      sqlEditorStore.removeHistoryItem(itemId);
    },
  });
};

const handleClearHistory = () => {
  dialog.warning({
    title: 'Clear All History',
    content: 'Are you sure you want to clear all query history? This action cannot be undone.',
    positiveText: 'Clear All',
    negativeText: 'Cancel',
    onPositiveClick: () => {
      sqlEditorStore.clearHistory();
    },
  });
};
</script>

<style scoped>
.query-history-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.search-bar {
  padding: 16px;
  border-bottom: 1px solid var(--n-border-color);
}

.history-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
}

.history-item {
  padding: 12px;
  margin-bottom: 8px;
  border: 1px solid var(--n-border-color);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.history-item:hover {
  background-color: var(--n-color-hover);
  border-color: var(--n-border-color-hover);
}

.history-item-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 8px;
}

.history-item-time {
  font-size: 12px;
  color: var(--n-text-color-3);
}

.history-item-query {
  font-family: 'Courier New', monospace;
  font-size: 13px;
  color: var(--n-text-color-2);
  margin-bottom: 8px;
  white-space: pre-wrap;
  word-break: break-word;
}

.history-item-footer {
  display: flex;
  align-items: center;
  justify-content: space-between;
}

.history-actions {
  padding: 16px;
  border-top: 1px solid var(--n-border-color);
}
</style>
