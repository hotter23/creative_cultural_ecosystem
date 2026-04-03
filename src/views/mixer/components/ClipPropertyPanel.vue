<template>
  <div class="clip-property-panel" v-if="selectedClip">
    <div class="property-row">
      <span class="property-label">名称:</span>
      <input
        type="text"
        class="property-input"
        :value="selectedClip.name"
        @input="updateName"
      />
    </div>

    <div class="property-row">
      <span class="property-label">音量:</span>
      <input
        type="range"
        class="property-slider"
        :value="selectedClip.volume * 100"
        min="0"
        max="100"
        @input="updateVolume"
      />
      <span class="property-value">{{ Math.round(selectedClip.volume * 100) }}%</span>
    </div>

    <div class="property-row">
      <span class="property-label">开始:</span>
      <input
        type="number"
        class="property-input time-input"
        :value="selectedClip.startTime.toFixed(2)"
        step="0.1"
        @input="updateStartTime"
      />
      <span class="property-label">时长:</span>
      <input
        type="number"
        class="property-input time-input"
        :value="selectedClip.duration.toFixed(2)"
        step="0.1"
        @input="updateDuration"
      />
    </div>

    <div class="property-row">
      <span class="property-label">淡入:</span>
      <input
        type="range"
        class="property-slider"
        :value="selectedClip.fadeIn"
        min="0"
        max="5"
        step="0.1"
        @input="updateFadeIn"
      />
      <span class="property-value">{{ selectedClip.fadeIn.toFixed(1) }}s</span>

      <span class="property-label">淡出:</span>
      <input
        type="range"
        class="property-slider"
        :value="selectedClip.fadeOut"
        min="0"
        max="5"
        step="0.1"
        @input="updateFadeOut"
      />
      <span class="property-value">{{ selectedClip.fadeOut.toFixed(1) }}s</span>
    </div>

    <div class="property-row">
      <label class="checkbox-label">
        <input
          type="checkbox"
          :checked="selectedClip.isMuted"
          @change="toggleMute"
        />
        静音
      </label>

      <button class="toolbar-btn danger" @click="deleteClip">
        🗑️ 删除片段
      </button>
    </div>
  </div>

  <div class="no-selection" v-else>
    <span>选择片段以查看和编辑属性</span>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useMixerStore } from '../stores/mixer';
import type { AudioClip } from '../types';
import { clamp } from '../types';

const store = useMixerStore();

const selectedClip = computed(() => store.selectedClip);

function updateName(e: Event) {
  const name = (e.target as HTMLInputElement).value;
  if (selectedClip.value) {
    store.updateClip(selectedClip.value.id, { name });
  }
}

function updateVolume(e: Event) {
  const value = parseInt((e.target as HTMLInputElement).value) / 100;
  if (selectedClip.value) {
    store.updateClip(selectedClip.value.id, { volume: value });
  }
}

function updateStartTime(e: Event) {
  const value = parseFloat((e.target as HTMLInputElement).value);
  if (selectedClip.value && !isNaN(value)) {
    store.moveClip(selectedClip.value.id, clamp(value, 0, Infinity));
  }
}

function updateDuration(e: Event) {
  const value = parseFloat((e.target as HTMLInputElement).value);
  if (selectedClip.value && !isNaN(value)) {
    store.updateClip(selectedClip.value.id, { duration: clamp(value, 0.1, Infinity) });
  }
}

function updateFadeIn(e: Event) {
  const value = parseFloat((e.target as HTMLInputElement).value);
  if (selectedClip.value && !isNaN(value)) {
    store.updateClip(selectedClip.value.id, { fadeIn: clamp(value, 0, 5) });
  }
}

function updateFadeOut(e: Event) {
  const value = parseFloat((e.target as HTMLInputElement).value);
  if (selectedClip.value && !isNaN(value)) {
    store.updateClip(selectedClip.value.id, { fadeOut: clamp(value, 0, 5) });
  }
}

function toggleMute(e: Event) {
  const checked = (e.target as HTMLInputElement).checked;
  if (selectedClip.value) {
    store.updateClip(selectedClip.value.id, { isMuted: checked });
  }
}

function deleteClip() {
  if (selectedClip.value) {
    store.removeClip(selectedClip.value.id);
  }
}
</script>

<style scoped>
.clip-property-panel {
  padding: 12px 16px;
  background: var(--mixer-bg-primary);
  border-top: 1px solid var(--mixer-border-color);
}

.property-row {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 8px;
}

.property-row:last-child {
  margin-bottom: 0;
}

.property-label {
  font-size: 13px;
  color: var(--mixer-text-secondary);
  min-width: 50px;
}

.property-input {
  flex: 1;
  padding: 4px 8px;
  border: 1px solid var(--mixer-border-color);
  border-radius: 4px;
  font-size: 13px;
  outline: none;
}

.property-input:focus {
  border-color: var(--mixer-primary-color);
}

.property-input.time-input {
  max-width: 80px;
}

.property-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--mixer-bg-tertiary);
  border-radius: 2px;
  outline: none;
}

.property-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--mixer-primary-color);
  cursor: pointer;
}

.property-value {
  font-size: 12px;
  color: var(--mixer-text-secondary);
  min-width: 40px;
  text-align: right;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--mixer-text-secondary);
  cursor: pointer;
}

.checkbox-label input {
  cursor: pointer;
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

.toolbar-btn:hover {
  background: var(--mixer-primary-color);
  color: white;
  border-color: var(--mixer-primary-color);
}

.toolbar-btn.danger {
  background: var(--mixer-danger-color);
  color: white;
  border-color: var(--mixer-danger-color);
}

.toolbar-btn.danger:hover {
  background: #f78989;
  border-color: #f78989;
}

.no-selection {
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 20px;
  background: var(--mixer-bg-primary);
  border-top: 1px solid var(--mixer-border-color);
  color: var(--mixer-text-placeholder);
  font-size: 13px;
}
</style>
