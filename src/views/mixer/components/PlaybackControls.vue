<template>
  <div class="playback-controls">
    <button @click="togglePlay" class="playback-btn">
      {{ isPlaying ? '⏸️' : '▶️' }}
    </button>
    <button @click="stop" class="playback-btn" :disabled="!canStop">
      ⏹️
    </button>

    <div class="time-display">
      {{ formatTime(currentTime) }} / {{ formatTime(duration) }}
    </div>

    <input
      type="range"
      class="progress-slider"
      :value="currentTime"
      :max="duration"
      step="0.01"
      @input="handleSeek"
    />

    <div class="volume-control">
      <span>🔊</span>
      <input
        type="range"
        class="volume-slider"
        :value="masterVolume * 100"
        min="0"
        max="100"
        @input="handleVolumeChange"
      />
      <span class="volume-value">{{ Math.round(masterVolume * 100) }}%</span>
    </div>

    <label class="loop-toggle">
      <input type="checkbox" v-model="isLoop" />
      🔁 循环
    </label>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue';
import { useMixerStore } from '../stores/mixer';
import { formatTime } from '../types';
import { getMixerEngine } from '../AudioMixer';

const store = useMixerStore();

const isPlaying = computed(() => store.isPlaying);
const currentTime = computed(() => store.currentTime);
const duration = computed(() => store.duration);
const tracks = computed(() => store.tracks);
const isLoop = ref(false);
const masterVolume = ref(1.0);

let playbackTimer: number | null = null;
let playbackStartTime = 0;
let playbackStartPosition = 0;

const canStop = computed(() => currentTime.value > 0 || isPlaying.value);

watch(isPlaying, (newVal) => {
  if (newVal) {
    startPlayback();
  } else {
    pausePlayback();
  }
});

watch(masterVolume, (newVal) => {
  const mixer = getMixerEngine();
  mixer.setMasterVolume(newVal);
});

// 【P1修复】监听 tracks 变化，触发音频缓冲重新加载
watch(
  () => store.tracks,
  async (newTracks) => {
    console.log('[PlaybackControls] Tracks 发生变化:', newTracks.length, '个轨道');
    await loadAllAudioBuffers();
    updateTrackGains();
  },
  { deep: true }
);

onMounted(() => {
  loadAllAudioBuffers();
});

onUnmounted(() => {
  stopPlaybackTimer();
});

// 【P1修复】完善的音频缓冲加载函数
async function loadAllAudioBuffers() {
  const mixer = getMixerEngine();

  console.log('[PlaybackControls] 开始加载音频缓冲...');

  for (const track of tracks.value) {
    // 确保轨道 GainNode 已创建
    if (!mixer['trackGains'].has(track.id)) {
      mixer.createTrackGain(track.id, track.volume);
    }

    console.log(`[PlaybackControls] 轨道: ${track.name}, ${track.clips.length} 个片段`);

    for (const clip of track.clips) {
      const bufferKey = clip.filePath || clip.id;

      if (clip.filePath && !mixer['audioBuffers'].has(bufferKey)) {
        try {
          console.log(`[PlaybackControls] 加载音频缓冲: ${clip.name} - ${clip.filePath}`);
          await mixer.loadAudio(bufferKey, clip.filePath);
          console.log(`[PlaybackControls] ✅ 音频加载成功: ${clip.name}`);
        } catch (error) {
          console.error(`[PlaybackControls] ❌ 加载音频失败: ${clip.name}`, error);
        }
      } else {
        console.log(`[PlaybackControls] 跳过已加载的音频: ${clip.name}`);
      }
    }
  }

  console.log('[PlaybackControls] 音频缓冲加载完成');
}

// 【P1修复】更新轨道 GainNode
function updateTrackGains() {
  const mixer = getMixerEngine();

  for (const track of tracks.value) {
    mixer.setTrackVolume(track.id, track.volume);
    console.log(`[PlaybackControls] 更新轨道音量: ${track.name} = ${track.volume * 100}%`);
  }
}

async function togglePlay() {
  await getMixerEngine().resume();
  store.togglePlay();
}

function stop() {
  pausePlayback();
  store.stop();
  const mixer = getMixerEngine();
  mixer.stopAll();
}

function handleSeek(e: Event) {
  const value = parseFloat((e.target as HTMLInputElement).value);
  const wasPlaying = isPlaying.value;

  if (wasPlaying) {
    pausePlayback();
  }

  store.setCurrentTime(value);

  if (wasPlaying) {
    setTimeout(() => {
      startPlayback();
    }, 50);
  }
}

function handleVolumeChange(e: Event) {
  const value = parseInt((e.target as HTMLInputElement).value) / 100;
  masterVolume.value = value;
}

async function startPlayback() {
  if (currentTime.value >= duration.value) {
    store.setCurrentTime(0);
  }

  playbackStartTime = Date.now();
  playbackStartPosition = currentTime.value;

  const mixer = getMixerEngine();

  const allClips = tracks.value.flatMap(t => t.clips);
  mixer.playClips(allClips, currentTime.value);

  startPlaybackTimer();
}

function pausePlayback() {
  const mixer = getMixerEngine();
  mixer.stopAll();
  stopPlaybackTimer();
}

function startPlaybackTimer() {
  stopPlaybackTimer();

  playbackTimer = window.setInterval(() => {
    const elapsed = (Date.now() - playbackStartTime) / 1000;
    const newTime = playbackStartPosition + elapsed;

    if (newTime >= duration.value) {
      if (isLoop.value) {
        store.setCurrentTime(0);
        playbackStartTime = Date.now();
        playbackStartPosition = 0;

        const mixer = getMixerEngine();
        const allClips = tracks.value.flatMap(t => t.clips);
        mixer.playClips(allClips, 0);
      } else {
        store.stop();
      }
    } else {
      store.setCurrentTime(newTime);
    }
  }, 50);
}

function stopPlaybackTimer() {
  if (playbackTimer !== null) {
    clearInterval(playbackTimer);
    playbackTimer = null;
  }
}
</script>

<style scoped>
.playback-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 16px;
  background: var(--mixer-bg-secondary);
  border-top: 1px solid var(--mixer-border-color);
}

.playback-btn {
  padding: 8px 16px;
  border: none;
  background: var(--mixer-primary-color);
  color: white;
  border-radius: 4px;
  cursor: pointer;
  font-size: 16px;
  transition: background 0.2s;
}

.playback-btn:hover {
  background: #66b1ff;
}

.playback-btn:disabled {
  background: var(--mixer-border-color);
  cursor: not-allowed;
}

.time-display {
  font-size: 14px;
  font-weight: 500;
  color: var(--mixer-text-primary);
  font-family: 'Courier New', monospace;
  min-width: 120px;
}

.progress-slider {
  flex: 1;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--mixer-bg-tertiary);
  border-radius: 2px;
  outline: none;
  cursor: pointer;
}

.progress-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border-radius: 50%;
  background: var(--mixer-primary-color);
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(0, 0, 0, 0.2);
}

.volume-control {
  display: flex;
  align-items: center;
  gap: 8px;
}

.volume-slider {
  width: 80px;
  height: 4px;
  -webkit-appearance: none;
  appearance: none;
  background: var(--mixer-bg-tertiary);
  border-radius: 2px;
  outline: none;
}

.volume-slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  border-radius: 50%;
  background: var(--mixer-primary-color);
  cursor: pointer;
}

.volume-value {
  font-size: 12px;
  color: var(--mixer-text-secondary);
  min-width: 35px;
}

.loop-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 13px;
  color: var(--mixer-text-secondary);
  cursor: pointer;
}

.loop-toggle input {
  cursor: pointer;
}
</style>
