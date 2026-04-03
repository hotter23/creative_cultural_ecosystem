<template>
  <div class="time-ruler">
    <div
      class="time-ruler-inner"
      :style="{ width: `${duration * zoom}px`, transform: `translateX(-${scrollX}px)` }"
    >
      <div
        v-for="marker in timeMarkers"
        :key="marker.time"
        class="time-marker"
        :style="{ left: `${marker.time * zoom}px` }"
      >
        <span class="time-marker-label">{{ marker.label }}</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  zoom: number;
  duration: number;
  scrollX: number;
  containerWidth: number;
}>();

interface TimeMarker {
  time: number;
  label: string;
  isMajor: boolean;
}

const timeMarkers = computed<TimeMarker[]>(() => {
  const markers: TimeMarker[] = [];
  const startTime = Math.floor(props.scrollX / props.zoom);
  const endTime = Math.ceil((props.scrollX + props.containerWidth) / props.zoom);

  const gridInterval = getGridInterval(props.zoom);

  for (let t = startTime; t <= endTime && t <= props.duration; t += gridInterval) {
    const isMajor = t % (gridInterval * 5) === 0 || t === 0;
    markers.push({
      time: t,
      label: formatTimeLabel(t),
      isMajor
    });
  }

  return markers;
});

function getGridInterval(zoom: number): number {
  if (zoom >= 100) return 0.5;
  if (zoom >= 50) return 1;
  if (zoom >= 25) return 2;
  return 5;
}

function formatTimeLabel(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);

  if (mins > 0) {
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
  return `${secs}s`;
}
</script>

<style scoped>
.time-ruler {
  height: 30px;
  background: var(--mixer-bg-secondary);
  border-bottom: 1px solid var(--mixer-border-color);
  overflow: hidden;
  position: relative;
}

.time-ruler-inner {
  height: 100%;
  position: relative;
}

.time-marker {
  position: absolute;
  top: 0;
  height: 100%;
  border-left: 1px solid var(--mixer-border-color);
}

.time-marker-label {
  position: absolute;
  top: 4px;
  left: 4px;
  font-size: 10px;
  color: var(--mixer-text-secondary);
  white-space: nowrap;
  pointer-events: none;
}
</style>
