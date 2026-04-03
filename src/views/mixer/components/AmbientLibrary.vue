<template>
  <div class="ambient-library">
    <div class="library-header">
      <h4>🌧️ 环境音库</h4>
    </div>

    <el-input
      v-model="searchQuery"
      placeholder="搜索环境音..."
      size="small"
      clearable
      class="search-input"
    />

    <el-radio-group v-model="selectedCategory" size="small" class="category-group">
      <el-radio-button value="">全部</el-radio-button>
      <el-radio-button value="自然">🌲 自然</el-radio-button>
      <el-radio-button value="城市">🏙️ 城市</el-radio-button>
      <el-radio-button value="室内">🏠 室内</el-radio-button>
    </el-radio-group>

    <div class="ambient-list">
      <div
        v-for="ambient in filteredAmbientSounds"
        :key="ambient.id"
        class="ambient-item"
        :class="{ 'is-added': isAdded(ambient.id) }"
        @click="handleAddAmbient(ambient)"
      >
        <div class="ambient-info">
          <span class="ambient-name">{{ ambient.name }}</span>
          <span class="ambient-category">{{ ambient.category }}</span>
        </div>
        <div class="ambient-meta">
          <span class="ambient-duration">{{ formatDuration(ambient.duration) }}</span>
          <span class="ambient-status">{{ isAdded(ambient.id) ? '✓ 已添加' : '+ 点击添加' }}</span>
        </div>
      </div>

      <div v-if="filteredAmbientSounds.length === 0" class="no-results">
        没有找到环境音
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue';
import type { AmbientSound } from '../types';

const props = defineProps<{
  ambientSounds: AmbientSound[];
}>();

const emit = defineEmits<{
  (e: 'add', ambient: AmbientSound): void;
}>();

const addedIds = ref<Set<number>>(new Set());

const filteredAmbientSounds = computed(() => {
  let result = props.ambientSounds;

  if (selectedCategory.value) {
    result = result.filter(a => a.category === selectedCategory.value);
  }

  if (searchQuery.value) {
    const query = searchQuery.value.toLowerCase();
    result = result.filter(a =>
      a.name.toLowerCase().includes(query)
    );
  }

  return result;
});

const selectedCategory = ref('');
const searchQuery = ref('');

function formatDuration(seconds?: number): string {
  if (!seconds) return '0:00';
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

function isAdded(id: number): boolean {
  return addedIds.value.has(id);
}

function handleAddAmbient(ambient: AmbientSound) {
  if (isAdded(ambient.id)) {
    return;
  }

  addedIds.value.add(ambient.id);
  emit('add', ambient);
  console.log(`[AmbientLibrary] 添加环境音: ${ambient.name}`);
}

</script>

<style scoped>
.ambient-library {
  display: flex;
  flex-direction: column;
  background: var(--mixer-bg-primary);
  border: 1px solid var(--mixer-border-color);
  border-radius: 4px;
  padding: 12px;
  min-height: 200px;
}

.library-header {
  margin-bottom: 12px;
}

.library-header h4 {
  margin: 0;
  font-size: 14px;
  color: var(--mixer-text-primary);
}

.search-input {
  margin-bottom: 12px;
}

.search-input :deep(.el-input__wrapper) {
  width: 100%;
}

.category-group {
  margin-bottom: 12px;
  display: flex;
  flex-wrap: wrap;
  gap: 4px;
}

.ambient-list {
  flex: 1;
  overflow-y: auto;
  max-height: 500px;
}

.ambient-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 10px 12px;
  margin-bottom: 8px;
  border: 1px solid var(--mixer-border-light);
  border-radius: 4px;
  background: var(--mixer-bg-secondary);
  cursor: pointer;
  transition: all 0.2s;
}

.ambient-item:hover {
  background: var(--mixer-bg-tertiary);
  border-color: var(--mixer-primary-color);
}

.ambient-item.is-added {
  background: #f0f9ff;
  border-color: var(--mixer-success-color);
  cursor: default;
}

.ambient-item.is-added:hover {
  border-color: var(--mixer-success-color);
}

.ambient-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.ambient-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--mixer-text-primary);
}

.ambient-category {
  font-size: 11px;
  color: var(--mixer-text-placeholder);
}

.ambient-meta {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.ambient-duration {
  font-size: 11px;
  color: var(--mixer-text-secondary);
}

.ambient-status {
  font-size: 11px;
  color: var(--mixer-success-color);
  font-weight: 500;
}

.ambient-item:not(.is-added) .ambient-status {
  color: var(--mixer-primary-color);
}

.no-results {
  text-align: center;
  padding: 20px;
  color: var(--mixer-text-placeholder);
  font-size: 13px;
}
</style>
