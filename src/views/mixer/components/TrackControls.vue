<template>
  <div class="track-control">
    <div class="track-name">{{ track.name }}</div>
    <div class="track-controls">
      <button
        class="track-btn"
        :class="{ active: track.isMuted }"
        @click="toggleMute"
        title="静音"
      >
        {{ track.isMuted ? '🔇' : '🔊' }}
      </button>
      <button
        class="track-btn"
        :class="{ active: track.isSolo }"
        @click="toggleSolo"
        title="独奏"
      >
        {{ track.isSolo ? '🎧' : '🎵' }}
      </button>
      <input
        type="range"
        class="track-volume-slider"
        :value="track.volume * 100"
        min="0"
        max="100"
        @input="updateVolume"
        title="音量"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { getMixerEngine } from '../AudioMixer';
import type { AudioTrack } from '../types';

const props = defineProps<{
  track: AudioTrack;
}>();

const emit = defineEmits<{
  (e: 'update', trackId: string, updates: Partial<AudioTrack>): void;
}>();

// 【P3修复】导入 AudioMixer
const mixer = getMixerEngine();

function toggleMute() {
  const newMuted = !props.track.isMuted;
  emit('update', props.track.id, { isMuted: newMuted });

  // 【P3修复】如果静音，同步到 AudioMixer
  if (newMuted) {
    mixer.setTrackVolume(props.track.id, 0);
  } else {
    mixer.setTrackVolume(props.track.id, props.track.volume);
  }
}

function toggleSolo() {
  emit('update', props.track.id, { isSolo: !props.track.isSolo });
}

function updateVolume(e: Event) {
  const value = parseInt((e.target as HTMLInputElement).value) / 100;
  emit('update', props.track.id, { volume: value });

  // 【P3修复】同步到 AudioMixer
  mixer.setTrackVolume(props.track.id, value);
}
</script>

<style scoped>
.track-control {
  width: 150px;
  min-width: 150px;
  height: 80px;
  display: flex;
  flex-direction: column;
  justify-content: center;
  padding: 8px;
  border-right: 1px solid var(--mixer-border-color);
  background: var(--mixer-bg-primary);
}

.track-name {
  font-size: 13px;
  font-weight: 500;
  color: var(--mixer-text-primary);
  margin-bottom: 4px;
}

.track-controls {
  display: flex;
  align-items: center;
  gap: 4px;
}

.track-btn {
  padding: 2px 6px;
  border: none;
  background: var(--mixer-bg-secondary);
  color: var(--mixer-text-secondary);
  border-radius: 3px;
  cursor: pointer;
  font-size: 12px;
  transition: all 0.2s;
}

.track-btn:hover {
  background: var(--mixer-primary-color);
  color: white;
}

.track-btn.active {
  background: var(--mixer-primary-color);
  color: white;
}

.track-volume-slider {
  width: 50px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--mixer-bg-tertiary);
  border-radius: 2px;
  outline: none;
}

.track-volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 10px;
  height: 10px;
  border-radius: 50%;
  background: var(--mixer-primary-color);
  cursor: pointer;
}
</style>
