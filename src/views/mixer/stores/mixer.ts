import { defineStore } from 'pinia';
import { ref, computed } from 'vue';
import type { AudioTrack, AudioClip, SnapConfig } from '../types';
import { createDefaultTrack, createDefaultClip, clamp } from '../types';

export const useMixerStore = defineStore('mixer', () => {
  const projectId = ref<number | null>(null);
  const projectName = ref<string>('');
  const chapterId = ref<number | null>(null);

  const tracks = ref<AudioTrack[]>([]);
  const duration = ref<number>(60);

  const zoom = ref<number>(50);
  const scrollX = ref<number>(0);
  const currentTime = ref<number>(0);
  const isPlaying = ref<boolean>(false);

  const selectedClipId = ref<string | null>(null);
  const selectedTrackId = ref<string | null>(null);

  const snapConfig = ref<SnapConfig>({
    enabled: true,
    mode: 'grid',
    gridSize: 0.5,
    threshold: 10
  });

  const history = ref<string[]>([]);
  const historyIndex = ref<number>(-1);
  const maxHistorySize = 50;

  const selectedClip = computed(() => {
    if (!selectedClipId.value) return null;
    for (const track of tracks.value) {
      const clip = track.clips.find(c => c.id === selectedClipId.value);
      if (clip) return clip;
    }
    return null;
  });

  const selectedTrack = computed(() => {
    if (!selectedTrackId.value) return null;
    return tracks.value.find(t => t.id === selectedTrackId.value) || null;
  });

  const voiceTrack = computed(() => {
    return tracks.value.find(t => t.type === 'voice');
  });

  const ambientTracks = computed(() => {
    return tracks.value.filter(t => t.type === 'ambient');
  });

  function initializeProject() {
    const voiceTrackObj = createDefaultTrack('voice', 0);
    const ambientTrack = createDefaultTrack('ambient', 1);

    tracks.value = [voiceTrackObj, ambientTrack];
    duration.value = 60;
    currentTime.value = 0;
    selectedClipId.value = null;
    selectedTrackId.value = null;
    isPlaying.value = false;

    saveHistory();
  }

  function saveHistory() {
    const state = JSON.stringify({
      tracks: tracks.value,
      duration: duration.value
    });

    if (historyIndex.value < history.value.length - 1) {
      history.value = history.value.slice(0, historyIndex.value + 1);
    }

    history.value.push(state);
    historyIndex.value = history.value.length - 1;

    if (history.value.length > maxHistorySize) {
      history.value.shift();
      historyIndex.value--;
    }
  }

  function undo() {
    if (historyIndex.value > 0) {
      historyIndex.value--;
      const state = JSON.parse(history.value[historyIndex.value]);
      tracks.value = state.tracks;
      duration.value = state.duration;
    }
  }

  function redo() {
    if (historyIndex.value < history.value.length - 1) {
      historyIndex.value++;
      const state = JSON.parse(history.value[historyIndex.value]);
      tracks.value = state.tracks;
      duration.value = state.duration;
    }
  }

  function canUndo(): boolean {
    return historyIndex.value > 0;
  }

  function canRedo(): boolean {
    return historyIndex.value < history.value.length - 1;
  }

  function addTrack(type: 'voice' | 'ambient') {
    saveHistory();

    const order = tracks.value.length;
    const newTrack = createDefaultTrack(type, order);
    tracks.value.push(newTrack);
  }

  function removeTrack(trackId: string) {
    saveHistory();

    const index = tracks.value.findIndex(t => t.id === trackId);
    if (index !== -1 && tracks.value.length > 1) {
      tracks.value.splice(index, 1);

      if (selectedTrackId.value === trackId) {
        selectedTrackId.value = null;
      }

      if (selectedClipId.value) {
        const clip = findClipById(selectedClipId.value);
        if (!clip || clip.trackId === trackId) {
          selectedClipId.value = null;
        }
      }

      updateDuration();
    }
  }

  function updateTrack(trackId: string, updates: Partial<AudioTrack>) {
    const track = tracks.value.find(t => t.id === trackId);
    if (track) {
      Object.assign(track, updates);
    }
  }

  function addClip(trackId: string, clip: AudioClip) {
    saveHistory();

    const track = tracks.value.find(t => t.id === trackId);
    if (track) {
      track.clips.push(clip);
      updateDuration();
    }
  }

  function removeClip(clipId: string) {
    saveHistory();

    for (const track of tracks.value) {
      const index = track.clips.findIndex(c => c.id === clipId);
      if (index !== -1) {
        track.clips.splice(index, 1);
        break;
      }
    }

    if (selectedClipId.value === clipId) {
      selectedClipId.value = null;
    }

    updateDuration();
  }

  function updateClip(clipId: string, updates: Partial<AudioClip>) {
    for (const track of tracks.value) {
      const clip = track.clips.find(c => c.id === clipId);
      if (clip) {
        Object.assign(clip, updates);
        updateDuration();
        return;
      }
    }
  }

  function moveClip(clipId: string, newStartTime: number, newTrackId?: string) {
    saveHistory();

    let clip: AudioClip | undefined;
    let sourceTrack: AudioTrack | undefined;

    for (const track of tracks.value) {
      const found = track.clips.find(c => c.id === clipId);
      if (found) {
        clip = found;
        sourceTrack = track;
        break;
      }
    }

    if (!clip || !sourceTrack) return;

    if (newTrackId && newTrackId !== sourceTrack.id) {
      sourceTrack.clips = sourceTrack.clips.filter(c => c.id !== clipId);
      const targetTrack = tracks.value.find(t => t.id === newTrackId);
      if (targetTrack) {
        clip.trackId = newTrackId;
        targetTrack.clips.push(clip);
      }
    }

    let finalTime = Math.max(0, newStartTime);

    if (snapConfig.value.enabled && snapConfig.value.mode === 'grid') {
      finalTime = Math.round(finalTime / snapConfig.value.gridSize) * snapConfig.value.gridSize;
    }

    clip.startTime = finalTime;
    updateDuration();
  }

  function resizeClip(clipId: string, newDuration: number, edge: 'left' | 'right') {
    saveHistory();

    for (const track of tracks.value) {
      const clip = track.clips.find(c => c.id === clipId);
      if (clip) {
        const minDuration = 0.1;
        const maxDuration = clip.duration + (clip.offset || 0);

        if (edge === 'right') {
          clip.duration = clamp(newDuration, minDuration, maxDuration);
        } else {
          const delta = clip.duration - newDuration;
          if (delta >= -clip.offset) {
            clip.startTime += delta;
            clip.duration = newDuration;
            clip.offset = Math.max(0, clip.offset + delta);
          }
        }

        updateDuration();
        break;
      }
    }
  }

  function selectClip(clipId: string | null) {
    selectedClipId.value = clipId;
  }

  function selectTrack(trackId: string | null) {
    selectedTrackId.value = trackId;
  }

  function findClipById(clipId: string): AudioClip | undefined {
    for (const track of tracks.value) {
      const clip = track.clips.find(c => c.id === clipId);
      if (clip) return clip;
    }
    return undefined;
  }

  function getClipById(clipId: string): AudioClip | undefined {
    return findClipById(clipId);
  }

  function updateDuration() {
    let maxEnd = 0;

    for (const track of tracks.value) {
      for (const clip of track.clips) {
        const end = clip.startTime + clip.duration;
        if (end > maxEnd) {
          maxEnd = end;
        }
      }
    }

    duration.value = Math.max(maxEnd + 10, 60);
  }

  function setZoom(newZoom: number) {
    zoom.value = clamp(newZoom, 10, 200);
  }

  function zoomIn() {
    setZoom(zoom.value * 1.2);
  }

  function zoomOut() {
    setZoom(zoom.value / 1.2);
  }

  function setScrollX(newScrollX: number) {
    scrollX.value = Math.max(0, newScrollX);
  }

  function setCurrentTime(time: number) {
    currentTime.value = clamp(time, 0, duration.value);
  }

  function play() {
    isPlaying.value = true;
  }

  function pause() {
    isPlaying.value = false;
  }

  function stop() {
    isPlaying.value = false;
    currentTime.value = 0;
  }

  function togglePlay() {
    if (isPlaying.value) {
      pause();
    } else {
      play();
    }
  }

  function setSnapConfig(config: Partial<SnapConfig>) {
    Object.assign(snapConfig.value, config);
  }

  function clearProject() {
    projectId.value = null;
    projectName.value = '';
    chapterId.value = null;
    tracks.value = [];
    duration.value = 60;
    currentTime.value = 0;
    selectedClipId.value = null;
    selectedTrackId.value = null;
    isPlaying.value = false;
    history.value = [];
    historyIndex.value = -1;
  }

  return {
    projectId,
    projectName,
    chapterId,
    tracks,
    duration,
    zoom,
    scrollX,
    currentTime,
    isPlaying,
    selectedClipId,
    selectedTrackId,
    snapConfig,
    history,
    historyIndex,

    selectedClip,
    selectedTrack,
    voiceTrack,
    ambientTracks,

    initializeProject,
    saveHistory,
    undo,
    redo,
    canUndo,
    canRedo,

    addTrack,
    removeTrack,
    updateTrack,

    addClip,
    removeClip,
    updateClip,
    moveClip,
    resizeClip,

    selectClip,
    selectTrack,
    findClipById,
    getClipById,

    updateDuration,
    setZoom,
    zoomIn,
    zoomOut,
    setScrollX,

    setCurrentTime,
    play,
    pause,
    stop,
    togglePlay,

    setSnapConfig,
    clearProject
  };
});
