export interface AudioClip {
  id: string;
  type: 'voice' | 'ambient';
  trackId: string;
  sourceId?: number;
  sourceType?: 'paragraph' | 'ambient';
  name: string;
  startTime: number;
  duration: number;
  offset: number;
  volume: number;
  fadeIn: number;
  fadeOut: number;
  isMuted: boolean;
  color: string;
  filePath: string;
  waveformData?: Float32Array;
}

export interface AudioTrack {
  id: string;
  type: 'voice' | 'ambient';
  name: string;
  color: string;
  volume: number;
  isMuted: boolean;
  isSolo: boolean;
  clips: AudioClip[];
  order: number;
}

export interface TimelineState {
  zoom: number;
  scrollX: number;
  duration: number;
  currentTime: number;
  isPlaying: boolean;
  selectedClipId: string | null;
  selectedTrackId: string | null;
}

export interface MixerProject {
  id: number;
  name: string;
  chapterId: number;
  description?: string;
  duration: number;
  zoomLevel: number;
  tracks: AudioTrack[];
  createdAt: string;
  updatedAt: string;
}

export interface MixerPreset {
  id: number;
  name: string;
  description?: string;
  category?: string;
  presetData: string;
  isSystem: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface SnapConfig {
  enabled: boolean;
  mode: 'none' | 'grid' | 'clip' | 'playhead';
  gridSize: number;
  threshold: number;
}

export interface DragData {
  type: 'audio-clip' | 'ambient-sound' | 'paragraph';
  clipId?: string;
  ambientId?: number;
  paragraphId?: number;
  operation?: 'move' | 'resize-left' | 'resize-right';
}

export interface Novel {
  id: number;
  title: string;
}

export interface Chapter {
  id: number;
  title: string;
}

export interface Paragraph {
  id: number;
  paragraphIndex: number;
  content: string;
  contentPreview: string;
  type: string;
  characterId: number | null;
  characterName: string | null;
  audioStatus: string;
  audioPath: string | null;
  duration?: number;
}

export interface AmbientSound {
  id: number;
  name: string;
  category: string;
  volume: number;
  filePath: string;
  duration?: number;
}

export function createDefaultTrack(type: 'voice' | 'ambient', order: number): AudioTrack {
  const colors = {
    voice: '#409EFF',
    ambient: '#67C23A'
  };

  return {
    id: `track-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    type,
    name: type === 'voice' ? 'TTS语音轨' : `环境音轨 ${order}`,
    color: colors[type],
    volume: 1.0,
    isMuted: false,
    isSolo: false,
    clips: [],
    order
  };
}

export function createDefaultClip(
  trackId: string,
  type: 'voice' | 'ambient',
  name: string,
  filePath: string,
  startTime: number = 0,
  duration: number = 0,
  sourceId?: number
): AudioClip {
  const colors = {
    voice: '#409EFF',
    ambient: '#67C23A'
  };

  return {
    id: `clip-${Date.now()}-${Math.random().toString(36).substr(2, 9)}`,
    type,
    trackId,
    name,
    filePath,
    startTime,
    duration,
    offset: 0,
    volume: 1.0,
    fadeIn: 0,
    fadeOut: 0,
    isMuted: false,
    color: colors[type],
    sourceId,
    sourceType: type === 'voice' ? 'paragraph' : 'ambient'
  };
}

export function formatTime(seconds: number): string {
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  const ms = Math.floor((seconds % 1) * 100);
  return `${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}.${ms.toString().padStart(2, '0')}`;
}

export function timeToPixels(time: number, zoom: number): number {
  return time * zoom;
}

export function pixelsToTime(pixels: number, zoom: number): number {
  return pixels / zoom;
}

export function clamp(value: number, min: number, max: number): number {
  return Math.min(Math.max(value, min), max);
}
