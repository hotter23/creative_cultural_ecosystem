<template>
  <div class="mixer-toolbar">
    <div class="toolbar-group">
      <button @click="handleUndo" class="toolbar-btn" :disabled="!canUndo" title="撤销 (Ctrl+Z)">
        ↩️ 撤销
      </button>
      <button @click="handleRedo" class="toolbar-btn" :disabled="!canRedo" title="重做 (Ctrl+Y)">
        ↪️ 重做
      </button>
    </div>

    <div class="toolbar-group">
      <span class="toolbar-label">缩放:</span>
      <button @click="handleZoomOut" class="toolbar-btn">➖</button>
      <input
        type="range"
        class="zoom-slider"
        :value="zoom"
        min="10"
        max="200"
        @input="handleZoomChange"
      />
      <button @click="handleZoomIn" class="toolbar-btn">➕</button>
      <span class="zoom-value">{{ Math.round(zoom) }}%</span>
    </div>

    <div class="toolbar-group">
      <label class="snap-toggle">
        <input
          type="checkbox"
          :checked="snapEnabled"
          @change="toggleSnap"
        />
        吸附
      </label>
    </div>

    <div class="toolbar-group">
      <button @click="handleSave" class="toolbar-btn" :disabled="!canSave">
        💾 保存
      </button>
      <button @click="handleExport" class="toolbar-btn" :disabled="!canExport">
        📤 导出
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useMixerStore } from '../stores/mixer';

const store = useMixerStore();

const zoom = computed(() => store.zoom);
const canUndo = computed(() => store.canUndo());
const canRedo = computed(() => store.canRedo());
const snapEnabled = computed(() => store.snapConfig.enabled);
const canSave = computed(() => store.chapterId !== null);
const canExport = computed(() => store.tracks.some(t => t.clips.length > 0));

function handleUndo() {
  store.undo();
}

function handleRedo() {
  store.redo();
}

function handleZoomIn() {
  store.zoomIn();
}

function handleZoomOut() {
  store.zoomOut();
}

function handleZoomChange(e: Event) {
  const value = parseInt((e.target as HTMLInputElement).value);
  store.setZoom(value);
}

function toggleSnap(e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  store.setSnapConfig({ enabled: checked });
}

function handleSave() {
  console.log('保存项目...');
}

function handleExport() {
  console.log('导出混音...');
}
</script>

<style scoped>
.mixer-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: var(--mixer-bg-secondary);
  border-bottom: 1px solid var(--mixer-border-color);
  gap: 16px;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.toolbar-btn {
  padding: 6px 12px;
  border: 1px solid var(--mixer-border-color);
  background: var(--mixer-bg-primary);
  color: var(--mixer-text-secondary);
  border-radius: 4px;
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
}

.toolbar-btn:hover:not(:disabled) {
  background: var(--mixer-primary-color);
  color: white;
  border-color: var(--mixer-primary-color);
}

.toolbar-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.toolbar-label {
  font-size: 13px;
  color: var(--mixer-text-secondary);
}

.zoom-slider {
  width: 100px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--mixer-bg-tertiary);
  border-radius: 2px;
  outline: none;
}

.zoom-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--mixer-primary-color);
  cursor: pointer;
}

.zoom-value {
  font-size: 12px;
  color: var(--mixer-text-secondary);
  min-width: 40px;
}

.snap-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--mixer-text-secondary);
  cursor: pointer;
}

.snap-toggle input {
  cursor: pointer;
}
</style>
