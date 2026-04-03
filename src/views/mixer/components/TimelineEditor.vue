<template>
  <div class="timeline-editor" @wheel="handleWheel">
    <div class="timeline-header">
      <TrackControls
        v-for="track in tracks"
        :key="track.id"
        :track="track"
        @update="handleTrackUpdate"
      />
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
          @dragend="handleDragEnd"
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
            @dragStart="handleClipDragStart"
            @dragEnd="handleClipDragEnd"
            @dragOver="handleTrackDragOver(track.id)"
            @dragLeave="handleTrackDragLeave"
            @drop="handleTrackDrop"
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
import { ElMessage } from 'element-plus';

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

// 本地拖拽数据状态
const localDragData = ref<any>(null);

const emit = defineEmits<{
  (e: 'dragEnd'): void;
}>();

// 监听父组件的拖拽状态变化
const props = defineProps<{
  currentDragData: any;
}>();

watch(() => props.currentDragData, (newVal) => {
  localDragData.value = newVal;
}, { immediate: true });

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

function handleTrackUpdate(trackId: string, updates: Partial<AudioTrackType>) {
  store.updateTrack(trackId, updates);
}

function handleSelectClip(clipId: string | null) {
  store.selectClip(clipId);
}

function handleMoveClip(data: { clipId: string; newStartTime: number; newTrackId?: string }) {
  store.moveClip(data.clipId, data.newStartTime, data.newTrackId);
}

function handleClipDragStart(clipId: string, e: MouseEvent) {
  const clip = store.getClipById(clipId);
  if (!clip) return;

  const dragData = {
    type: 'audio-clip',
    clipId: clip.id,
    name: clip.name,
    duration: clip.duration,
    operation: 'move'
  };

  localDragData.value = dragData;
}

function handleClipDragEnd() {
  localDragData.value = null;
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

function handleDragEnd() {
  localDragData.value = null;
  emit('dragEnd');
}

// 处理容器级别的拖拽（用于拖拽到空白区域）
function handleContainerDragOver(e: DragEvent) {
  e.preventDefault();
}

// 处理容器级别的放置
function handleContainerDrop(e: DragEvent) {
  e.preventDefault();
}

// 处理轨道级别的放置
function handleTrackDrop(data: { trackId: string; event: DragEvent }) {
  const e = data.event as DragEvent;
  const trackId = data.trackId;
  e.preventDefault();
  e.stopPropagation();
  dragOverTrackId.value = null;

  try {
    let dragData = localDragData.value;

    if (!dragData && props.currentDragData) {
      dragData = props.currentDragData;
    }

    if (!dragData && e.dataTransfer) {
      try {
        const jsonStr = e.dataTransfer.getData('application/json');
        if (jsonStr) {
          dragData = JSON.parse(jsonStr);
        }
      } catch (err) {
        // 忽略
      }
    }

    if (!dragData) {
      ElMessage.error('未获取到拖拽数据');
      return;
    }

    if (dragData.type === 'ambient-sound' && dragData.ambientId) {
      const targetTrack = tracks.value.find(t => t.id === trackId);
      if (!targetTrack) {
        ElMessage.error('未找到目标轨道');
        return;
      }

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

      store.addClip(targetTrack.id, newClip);
      ElMessage.success(`已添加 ${newClip.name} 到轨道`);
    } else {
      ElMessage.error('不支持的拖拽类型');
    }

    localDragData.value = null;
  } catch (error) {
    ElMessage.error('拖拽放置失败');
  }
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
</style>
