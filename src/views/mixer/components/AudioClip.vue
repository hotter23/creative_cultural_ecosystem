<template>
  <div
    class="audio-clip"
    :class="{
      'is-selected': isSelected,
      'is-dragging': isDragging
    }"
    :style="clipStyle"
    draggable="true"
    @click.stop="handleClick"
    @dragstart="handleDragStart"
    @drag="handleDrag"
    @dragend="handleDragEnd"
  >
    <div
      class="clip-resize-handle left"
      @mousedown.stop="startResize($event, 'left')"
    />

    <div class="clip-content">
      <WaveformCanvas
        v-if="clip.waveformData"
        :data="clip.waveformData"
        :color="clip.color"
      />
      <div v-else class="clip-placeholder" :style="{ backgroundColor: clip.color }" />
    </div>

    <div class="clip-label">{{ clip.name }}</div>

    <div
      class="clip-resize-handle right"
      @mousedown.stop="startResize($event, 'right')"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue';
import type { AudioClip } from '../types';
import { clamp } from '../types';
import WaveformCanvas from './WaveformCanvas.vue';

const props = defineProps<{
  clip: AudioClip;
  zoom: number;
  isSelected: boolean;
}>();

const emit = defineEmits<{
  (e: 'select', clipId: string): void;
  (e: 'move', data: { clipId: string; newStartTime: number }): void;
  (e: 'resize', data: { clipId: string; newDuration: number; edge: 'left' | 'right' }): void;
  (e: 'dragStart', clipId: string, e: DragEvent): void;
}>();

const isDragging = ref(false);
const isResizing = ref(false);
const resizeEdge = ref<'left' | 'right'>('right');

const clipStyle = computed(() => ({
  left: `${props.clip.startTime * props.zoom}px`,
  width: `${props.clip.duration * props.zoom}px`,
  backgroundColor: props.clip.color
}));

let dragStartX = 0;
let dragStartTime = 0;
let dragStartDuration = 0;
let resizeStartX = 0;
let resizeStartDuration = 0;
let resizeStartOffset = 0;

// 【P6修复】添加点击和拖拽状态追踪
let clickStartTime = 0;
let clickStartPos = { x: 0, y: 0 };
let hasMoved = false;

function handleClick(e: MouseEvent) {
  // 只有在非拖拽、非调整大小状态下才触发选中
  if (!isDragging.value && !isResizing.value && !hasMoved) {
    emit('select', props.clip.id);
  }
}

function handleDragStart(e: DragEvent) {
  isDragging.value = true;
  hasMoved = false;
  clickStartTime = Date.now();
  clickStartPos = { x: e.clientX, y: e.clientY };
  dragStartX = e.clientX;
  dragStartTime = props.clip.startTime;

  emit('dragStart', props.clip.id, e);
}

function handleDrag(e: DragEvent) {
  if (!isDragging.value) return;

  // 检查是否真的移动了
  const deltaX = e.clientX - clickStartPos.x;
  const deltaY = e.clientY - clickStartPos.y;
  if (Math.abs(deltaX) > 5 || Math.abs(deltaY) > 5) {
    hasMoved = true;
  }

  const deltaTime = deltaX / props.zoom;
  const newStartTime = clamp(dragStartTime + deltaTime, 0, Infinity);

  emit('moveClip', {
    clipId: props.clip.id,
    newStartTime
  });
}

function handleDragEnd() {
  // 延迟重置状态，确保 click 事件不会触发
  setTimeout(() => {
    isDragging.value = false;
    hasMoved = false;
  }, 50);
}

function startResize(e: MouseEvent, edge: 'left' | 'right') {
  isResizing.value = true;
  resizeEdge.value = edge;
  resizeStartX = e.clientX;
  resizeStartDuration = props.clip.duration;
  resizeStartOffset = props.clip.offset;

  document.addEventListener('mousemove', handleResize);
  document.addEventListener('mouseup', stopResize);
}

function handleResize(e: MouseEvent) {
  if (!isResizing.value) return;

  const deltaX = e.clientX - resizeStartX;
  const deltaDuration = deltaX / props.zoom;

  let newDuration: number;

  if (resizeEdge.value === 'right') {
    newDuration = clamp(resizeStartDuration + deltaDuration, 0.1, Infinity);
  } else {
    newDuration = clamp(resizeStartDuration - deltaDuration, 0.1, resizeStartDuration + resizeStartOffset);
  }

  emit('resize', {
    clipId: props.clip.id,
    newDuration,
    edge: resizeEdge.value
  });
}

function stopResize() {
  isResizing.value = false;
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopResize);
}

onUnmounted(() => {
  document.removeEventListener('mousemove', handleResize);
  document.removeEventListener('mouseup', stopResize);
});
</script>

<style scoped>
.audio-clip {
  position: absolute;
  top: 4px;
  height: calc(100% - 8px);
  border-radius: 4px;
  cursor: grab;
  overflow: hidden;
  transition: box-shadow 0.2s, transform 0.1s;
  display: flex;
  flex-direction: column;
}

.audio-clip:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.15);
}

.audio-clip.is-selected {
  box-shadow: 0 0 0 2px var(--mixer-primary-color);
  z-index: 10;
}

.audio-clip.is-dragging {
  cursor: grabbing;
  opacity: 0.8;
  transform: scale(1.02);
}

.clip-content {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.clip-placeholder {
  width: 100%;
  height: 100%;
  opacity: 0.6;
}

.clip-label {
  padding: 2px 6px;
  font-size: 11px;
  color: white;
  background: rgba(0, 0, 0, 0.3);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.clip-resize-handle {
  position: absolute;
  top: 0;
  width: 8px;
  height: 100%;
  cursor: ew-resize;
  background: rgba(255, 255, 255, 0.1);
  opacity: 0;
  transition: opacity 0.2s;
}

.clip-resize-handle:hover {
  opacity: 1;
  background: rgba(255, 255, 255, 0.3);
}

.clip-resize-handle.left {
  left: 0;
  border-radius: 4px 0 0 4px;
}

.clip-resize-handle.right {
  right: 0;
  border-radius: 0 4px 4px 0;
}
</style>
