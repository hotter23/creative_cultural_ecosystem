<template>
  <div
    class="audio-clip"
    :class="{
      'is-selected': isSelected,
      'is-dragging': isDragging
    }"
    :style="clipStyle"
    @click.stop="handleClick"
    @mousedown="handleMouseDown"
  >
    <div class="clip-content">
      <WaveformCanvas
        v-if="clip.waveformData"
        :data="clip.waveformData"
        :color="clip.color"
      />
      <div v-else class="clip-placeholder" :style="{ backgroundColor: clip.color }" />
    </div>

    <div class="clip-label">{{ clip.name }}</div>
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
  (e: 'dragStart', clipId: string, event: DragEvent): void;
  (e: 'dragEnd'): void;
}>();

const isDragging = ref(false);
const clipStyle = computed(() => ({
  left: `${props.clip.startTime * props.zoom}px`,
  width: `${props.clip.duration * props.zoom}px`,
  backgroundColor: props.clip.color
}));

let dragStartTime = 0;
let clickStartPos = { x: 0, y: 0 };
let hasMoved = false;

function handleClick(e: MouseEvent) {
  // 只有在没有移动过的情况下才触发选中
  if (!hasMoved) {
    emit('select', props.clip.id);
  }
}

function handleMouseDown(e: MouseEvent) {
  if (e.button !== 0) return;

  // 开始监听拖拽
  isDragging.value = true;
  hasMoved = false;
  clickStartPos = { x: e.clientX, y: e.clientY };
  dragStartTime = props.clip.startTime;

  document.addEventListener('mousemove', handleMouseMove);
  document.addEventListener('mouseup', handleMouseUp);
}

function handleMouseMove(e: MouseEvent) {
  if (!isDragging.value) return;

  const deltaX = e.clientX - clickStartPos.x;
  const deltaY = e.clientY - clickStartPos.y;

  // 超过阈值才算移动
  if (!hasMoved && (Math.abs(deltaX) > 5 || Math.abs(deltaY) > 5)) {
    hasMoved = true;
    emit('dragStart', props.clip.id, e);
  }

  if (hasMoved) {
    const deltaTime = deltaX / props.zoom;
    const newStartTime = clamp(dragStartTime + deltaTime, 0, Infinity);

    emit('move', {
      clipId: props.clip.id,
      newStartTime
    });
  }
}

function handleMouseUp() {
  isDragging.value = false;
  document.removeEventListener('mousemove', handleMouseMove);
  document.removeEventListener('mouseup', handleMouseUp);

  if (hasMoved) {
    emit('dragEnd');
  }

  setTimeout(() => {
    hasMoved = false;
  }, 50);
}

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
</style>
