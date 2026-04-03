<template>
  <div class="timeline-editor" @wheel="handleWheel">
    <div class="timeline-header">
      <TrackControls
        v-for="track in tracks"
        :key="track.id"
        :track="track"
        @update="handleTrackUpdate"
        @remove="handleRemoveTrack"
      />
      <button @click="handleAddTrack" class="add-track-btn">
        + 添加轨道
      </button>
    </div>

    <div class="timeline-body" ref="timelineBody">
      <TimeRuler
        :zoom="zoom"
        :duration="duration"
        :scrollX="scrollX"
        :containerWidth="containerWidth"
      />

      <div
        class="tracks-container"
        ref="tracksContainer"
        @scroll="handleScroll"
      >
        <div
          class="tracks-inner"
          :style="{ width: `${duration * zoom}px` }"
          @dragover.prevent="handleContainerDragOver"
          @drop="handleContainerDrop"
        >
          <AudioTrack
            v-for="track in tracks"
            :key="track.id"
            :track="track"
            :zoom="zoom"
            :scrollX="scrollX"
            :selectedClipId="selectedClipId"
            :isDragOver="dragOverTrackId === track.id"
            @selectClip="handleSelectClip"
            @moveClip="handleMoveClip"
            @resizeClip="handleResizeClip"
            @dragStart="handleClipDragStart"
            @dragOver="handleTrackDragOver(track.id)"
            @dragLeave="handleTrackDragLeave"
            @drop="handleTrackDrop(track.id)"
          />

          <Playhead
            v-if="isPlaying || currentTime > 0"
            :currentTime="currentTime"
            :zoom="zoom"
            :scrollX="scrollX"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, watch } from 'vue';
import { useMixerStore } from '../stores/mixer';
import TrackControls from './TrackControls.vue';
import TimeRuler from './TimeRuler.vue';
import AudioTrack from './AudioTrack.vue';
import Playhead from './Playhead.vue';
import type { AudioTrack as AudioTrackType, AudioClip } from '../types';
import { clamp } from '../types';

const store = useMixerStore();

const tracks = computed(() => store.tracks);
const zoom = computed(() => store.zoom);
const duration = computed(() => store.duration);
const scrollX = computed(() => store.scrollX);
const currentTime = computed(() => store.currentTime);
const isPlaying = computed(() => store.isPlaying);
const selectedClipId = computed(() => store.selectedClipId);

const timelineBody = ref<HTMLElement>();
const tracksContainer = ref<HTMLElement>();
const containerWidth = ref(800);
const dragOverTrackId = ref<string | null>(null);

const isDragging = ref(false);
const dragStartPos = ref({ x: 0, y: 0 });

onMounted(() => {
  updateContainerWidth();
  window.addEventListener('resize', updateContainerWidth);
  window.addEventListener('keydown', handleKeyDown);
});

onUnmounted(() => {
  window.removeEventListener('resize', updateContainerWidth);
  window.removeEventListener('keydown', handleKeyDown);
});

watch(() => store.duration, () => {
  updateContainerWidth();
});

function updateContainerWidth() {
  if (timelineBody.value) {
    containerWidth.value = timelineBody.value.clientWidth;
  }
}

function handleWheel(e: WheelEvent) {
  if (e.ctrlKey) {
    e.preventDefault();
    const delta = e.deltaY > 0 ? -10 : 10;
    store.setZoom(zoom.value + delta);
  } else {
    const delta = e.deltaX !== 0 ? e.deltaX : e.deltaY;
    store.setScrollX(scrollX.value + delta);
  }
}

function handleScroll(e: Event) {
  const target = e.target as HTMLElement;
  store.setScrollX(target.scrollLeft);
}

function handleAddTrack() {
  store.addTrack('ambient');
}

function handleRemoveTrack(trackId: string) {
  store.removeTrack(trackId);
}

function handleTrackUpdate(trackId: string, updates: Partial<AudioTrackType>) {
  store.updateTrack(trackId, updates);
}

function handleSelectClip(clipId: string | null) {
  store.selectClip(clipId);
}

function handleMoveClip(data: { clipId: string; newStartTime: number; newTrackId?: string }) {
  store.moveClip(data.clipId, data.newStartTime, data.newTrackId);
}

function handleResizeClip(data: { clipId: string; newDuration: number; edge: 'left' | 'right' }) {
  store.resizeClip(data.clipId, data.newDuration, data.edge);
}

function handleClipDragStart(clipId: string, e: DragEvent) {
  const clip = store.getClipById(clipId);
  if (!clip) return;

  const dragData = {
    type: 'audio-clip',
    clipId: clip.id,
    operation: 'move'
  };

  e.dataTransfer!.setData('application/json', JSON.stringify(dragData));
  e.dataTransfer!.effectAllowed = 'move';
}

function handleTrackDragOver(trackId: string) {
  return (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragOverTrackId.value = trackId;
  };
}

function handleTrackDragLeave() {
  dragOverTrackId.value = null;
}

// 处理容器级别的拖拽（用于拖拽到空白区域）
function handleContainerDragOver(e: DragEvent) {
  e.preventDefault();
  console.log('[TimelineEditor] Container dragover');
}

// 处理容器级别的放置
function handleContainerDrop(e: DragEvent) {
  e.preventDefault();
  console.log('[TimelineEditor] Container drop - 未找到目标轨道');
}

// 处理轨道级别的放置
function handleTrackDrop(trackId: string) {
  return (data: { clipId: string; e: DragEvent }) => {
    const e = data.e as DragEvent;
    e.preventDefault();
    e.stopPropagation();
    dragOverTrackId.value = null;

    try {
      const dragData = JSON.parse(e.dataTransfer!.getData('application/json'));

      console.log('[TimelineEditor] Drop data:', dragData);

      if (dragData.type === 'ambient-sound' && dragData.ambientId) {
        const targetTrack = tracks.value.find(t => t.id === trackId);
        if (!targetTrack) {
          console.error('[TimelineEditor] 未找到目标轨道:', trackId);
          return;
        }

        // 计算放置位置
        const rect = (e.currentTarget as HTMLElement).getBoundingClientRect();
        const x = e.clientX - rect.left + scrollX.value;
        const startTime = clamp(x / zoom.value, 0, duration.value);

        const newClip: AudioClip = {
          id: `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
          type: 'ambient',
          trackId: targetTrack.id,
          name: dragData.name || '环境音',
          filePath: dragData.filePath || '',
          startTime: startTime,
          duration: dragData.duration || 10,
          offset: 0,
          volume: 1.0,
          fadeIn: 0,
          fadeOut: 0,
          isMuted: false,
          color: targetTrack.color,
          sourceId: dragData.ambientId,
          sourceType: 'ambient'
        };

        console.log('[TimelineEditor] 添加环境音片段:', newClip);
        store.addClip(targetTrack.id, newClip);
      }
    } catch (error) {
      console.error('[TimelineEditor] Drop error:', error);
    }
  };
}

function handleKeyDown(e: KeyboardEvent) {
  if (e.target instanceof HTMLInputElement || e.target instanceof HTMLTextAreaElement) {
    return;
  }

  if (e.ctrlKey && e.key === 'z') {
    e.preventDefault();
    store.undo();
  } else if (e.ctrlKey && e.key === 'y') {
    e.preventDefault();
    store.redo();
  } else if (e.key === 'Delete' || e.key === 'Backspace') {
    if (selectedClipId.value) {
      e.preventDefault();
      store.removeClip(selectedClipId.value);
    }
  } else if (e.key === ' ' || e.code === 'Space') {
    e.preventDefault();
    store.togglePlay();
  }
}
</script>

<style scoped>
.timeline-editor {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--mixer-bg-primary);
  border: 1px solid var(--mixer-border-color);
  border-radius: 4px;
  overflow: hidden;
}

.timeline-header {
  display: flex;
  background: var(--mixer-bg-secondary);
  border-bottom: 1px solid var(--mixer-border-color);
  overflow-x: auto;
  overflow-y: hidden;
}

.tracks-container {
  flex: 1;
  overflow: auto;
  position: relative;
}

.tracks-inner {
  position: relative;
  min-height: 100%;
}

.add-track-btn {
  width: 150px;
  min-width: 150px;
  height: 80px;
  border: none;
  background: var(--mixer-bg-secondary);
  color: var(--mixer-text-secondary);
  cursor: pointer;
  font-size: 13px;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.add-track-btn:hover {
  background: var(--mixer-primary-color);
  color: white;
}
</style>
