<template>
  <canvas ref="canvas" class="waveform-canvas" />
</template>

<script setup lang="ts">
import { ref, onMounted, watch, nextTick } from 'vue';

const props = defineProps<{
  data: Float32Array;
  color: string;
}>();

const canvas = ref<HTMLCanvasElement>();
let ctx: CanvasRenderingContext2D | null = null;

onMounted(() => {
  if (canvas.value) {
    ctx = canvas.value.getContext('2d');
    nextTick(() => {
      draw();
    });
  }
});

watch(() => [props.data, props.color], () => {
  nextTick(() => {
    draw();
  });
});

function draw() {
  if (!canvas.value || !ctx || !props.data || props.data.length === 0) return;

  const canvasEl = canvas.value;
  const dpr = window.devicePixelRatio || 1;

  canvasEl.width = canvasEl.offsetWidth * dpr;
  canvasEl.height = canvasEl.offsetHeight * dpr;

  ctx.scale(dpr, dpr);

  const width = canvasEl.offsetWidth;
  const height = canvasEl.offsetHeight;
  const centerY = height / 2;

  ctx.clearRect(0, 0, width, height);

  ctx.fillStyle = props.color;
  ctx.globalAlpha = 0.6;

  const step = width / props.data.length;

  for (let i = 0; i < props.data.length; i++) {
    const x = i * step;
    const amplitude = props.data[i] * centerY * 0.9;

    ctx.fillRect(x, centerY - amplitude, Math.max(step - 1, 1), amplitude * 2);
  }

  ctx.globalAlpha = 1.0;

  ctx.strokeStyle = props.color;
  ctx.lineWidth = 1;

  ctx.beginPath();
  ctx.moveTo(0, centerY);
  for (let i = 0; i < props.data.length; i++) {
    const x = i * step;
    const y = centerY - props.data[i] * centerY * 0.9;
    ctx.lineTo(x, y);
  }
  ctx.stroke();

  ctx.beginPath();
  ctx.moveTo(0, centerY);
  for (let i = 0; i < props.data.length; i++) {
    const x = i * step;
    const y = centerY + props.data[i] * centerY * 0.9;
    ctx.lineTo(x, y);
  }
  ctx.stroke();
}
</script>

<style scoped>
.waveform-canvas {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
