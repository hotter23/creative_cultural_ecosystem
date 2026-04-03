<template>
  <div
    class="playhead"
    :style="playheadStyle"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue';

const props = defineProps<{
  currentTime: number;
  zoom: number;
  scrollX: number;
}>();

const playheadStyle = computed(() => ({
  left: `${props.currentTime * props.zoom - props.scrollX}px`
}));
</script>

<style scoped>
.playhead {
  position: absolute;
  top: 0;
  width: 2px;
  height: 100%;
  background: var(--mixer-playhead-color);
  pointer-events: none;
  z-index: 100;
  opacity: 0.8;
}

.playhead::before {
  content: '';
  position: absolute;
  top: 0;
  left: -6px;
  width: 0;
  height: 0;
  border-left: 6px solid transparent;
  border-right: 6px solid transparent;
  border-top: 10px solid var(--mixer-playhead-color);
}
</style>
