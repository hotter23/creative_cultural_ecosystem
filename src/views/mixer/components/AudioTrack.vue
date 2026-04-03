<template>
  <div
    class="audio-track"
    :class="{
      'is-muted': track.isMuted,
      'is-drag-over': isDragOver
    }"
    @dragover="handleDragOver"
    @dragleave="handleDragLeave"
    @drop="handleDrop"
  >
    <div class="track-background">
      <div
        class="track-grid-lines"
        :style="{ width: `${duration * zoom}px` }"
      >
        <div
          v-for="i in gridLineCount"
          :key="i"
          class="grid-line"
          :style="{ left: `${i * gridInterval * zoom}px` }"
        />
      </div>
    </div>

    <AudioClip
      v-for="clip in track.clips"
      :key="clip.id"
      :clip="clip"
      :zoom="zoom"
      :isSelected="clip.id === selectedClipId"
      @select="handleSelectClip"
      @move="handleMoveClip"
      @dragStart="handleClipDragStart"
      @dragEnd="handleClipDragEnd"
    />
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';
import { useMixerStore } from '../stores/mixer';
import AudioClip from './AudioClip.vue';
import type { AudioClip as AudioClipType } from '../types';

const props = defineProps<{
  track: any;
  zoom: number;
  scrollX: number;
  selectedClipId: string | null;
  isDragOver: boolean;
}>();

const emit = defineEmits<{
  (e: 'selectClip', clipId: string | null): void;
  (e: 'moveClip', data: { clipId: string; newStartTime: number; newTrackId?: string }): void;
  (e: 'dragStart', clipId: string, event: MouseEvent): void;
  (e: 'dragEnd'): void;
  (e: 'dragOver'): void;
  (e: 'dragLeave'): void;
  (e: 'drop', data: { trackId: string; event: DragEvent }): void;
}>();

const store = useMixerStore();
const duration = computed(() => store.duration);

const gridInterval = computed(() => {
  if (props.zoom >= 100) return 0.5;
  if (props.zoom >= 50) return 1;
  if (props.zoom >= 25) return 2;
  return 5;
});

const gridLineCount = computed(() => {
  return Math.ceil(duration.value / gridInterval.value);
});

function handleSelectClip(clipId: string) {
  emit('selectClip', clipId);
}

function handleMoveClip(data: { clipId: string; newStartTime: number }) {
  emit('moveClip', { ...data, newTrackId: undefined });
}

function handleClipDragStart(clipId: string, e: MouseEvent) {
  emit('dragStart', clipId, e);
}

function handleClipDragEnd() {
  emit('dragEnd');
}

function handleDragOver(e: DragEvent) {
  e.preventDefault();
  e.stopPropagation();
  emit('dragOver');
}

function handleDragLeave() {
  emit('dragLeave');
}

function handleDrop(e: DragEvent) {
  e.preventDefault();
  e.stopPropagation();
  emit('drop', { trackId: props.track.id, event: e });
}
</script>

<style scoped>
.audio-track {
  height: 80px;
  position: relative;
  border-bottom: 1px solid var(--mixer-border-light);
  background: var(--mixer-bg-primary);
  transition: background 0.2s;
}

.audio-track:hover {
  background: var(--mixer-bg-secondary);
}

.audio-track.is-muted {
  opacity: 0.5;
}

.audio-track.is-drag-over {
  background: rgba(64, 158, 255, 0.1);
}

.track-background {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  pointer-events: none;
}

.track-grid-lines {
  height: 100%;
  position: relative;
}

.grid-line {
  position: absolute;
  top: 0;
  height: 100%;
  width: 1px;
  background: var(--mixer-border-light);
}
</style>
