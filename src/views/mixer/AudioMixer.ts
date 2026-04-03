import type { AudioClip } from './types';

export class AudioMixerEngine {
  private audioContext: AudioContext | null = null;
  private masterGain: GainNode | null = null;
  private trackGains: Map<string, GainNode> = new Map();
  private trackVolumes: Map<string, number> = new Map();
  private audioBuffers: Map<string, AudioBuffer> = new Map();
  private activeSources: Map<string, AudioBufferSourceNode> = new Map();
  private masterVolume: number = 1.0;

  constructor() {
    this.initAudioContext();
  }

  private initAudioContext() {
    if (typeof window !== 'undefined' && 'AudioContext' in window) {
      this.audioContext = new AudioContext();
      this.masterGain = this.audioContext.createGain();
      this.masterGain.connect(this.audioContext.destination);
      this.masterGain.gain.value = this.masterVolume;
      console.log('[AudioMixer] ✅ AudioContext 初始化成功');
    } else {
      console.error('[AudioMixer] ❌ AudioContext 初始化失败');
    }
  }

  async resume(): Promise<void> {
    if (this.audioContext && this.audioContext.state === 'suspended') {
      await this.audioContext.resume();
      console.log('[AudioMixer] ✅ AudioContext 已恢复');
    }
  }

  async loadAudio(id: string, url: string): Promise<void> {
    if (!this.audioContext) {
      throw new Error('AudioContext not initialized');
    }

    console.log(`[AudioMixer] 加载音频: ${id} -> ${url}`);

    try {
      const response = await fetch(url);
      if (!response.ok) {
        throw new Error(`HTTP error! status: ${response.status}`);
      }

      const arrayBuffer = await response.arrayBuffer();
      const audioBuffer = await this.audioContext.decodeAudioData(arrayBuffer);
      this.audioBuffers.set(id, audioBuffer);

      console.log(`[AudioMixer] ✅ 音频加载成功: ${id}, 时长: ${audioBuffer.duration.toFixed(2)}s`);
    } catch (error) {
      console.error(`[AudioMixer] ❌ 加载音频失败 ${id}:`, error);
      throw error;
    }
  }

  // 【P2修复】完善 createTrackGain 方法
  createTrackGain(trackId: string, volume: number = 1.0): GainNode | null {
    if (!this.audioContext || !this.masterGain) {
      console.error('[AudioMixer] createTrackGain: AudioContext 未初始化');
      return null;
    }

    if (!this.trackGains.has(trackId)) {
      const gain = this.audioContext.createGain();
      gain.gain.value = volume; // 【P2修复】设置初始音量
      gain.connect(this.masterGain);
      this.trackGains.set(trackId, gain);
      this.trackVolumes.set(trackId, volume);
      console.log(`[AudioMixer] ✅ 创建轨道 GainNode: ${trackId}, 音量: ${volume * 100}%`);
    }

    return this.trackGains.get(trackId) || null;
  }

  // 【P2修复】完善 setTrackVolume 方法
  setTrackVolume(trackId: string, volume: number) {
    const clampedVolume = Math.max(0, Math.min(1, volume));
    this.trackVolumes.set(trackId, clampedVolume);

    if (!this.trackGains.has(trackId)) {
      // 【P2修复】如果 GainNode 不存在，先创建
      this.createTrackGain(trackId, clampedVolume);
    } else {
      const gain = this.trackGains.get(trackId)!;
      gain.gain.value = clampedVolume;
    }

    console.log(`[AudioMixer] 设置轨道音量: ${trackId} = ${clampedVolume * 100}%`);
  }

  getTrackVolume(trackId: string): number {
    return this.trackVolumes.get(trackId) || 1.0;
  }

  setMasterVolume(volume: number) {
    this.masterVolume = Math.max(0, Math.min(1, volume));
    if (this.masterGain) {
      this.masterGain.gain.value = this.masterVolume;
    }
  }

  // 【P5修复】修正 playClip 方法的播放位置计算
  playClip(
    clip: AudioClip,
    playbackPosition: number = 0,
    trackVolume: number = 1.0
  ): void {
    if (!this.audioContext || !clip.filePath) {
      console.warn(`[AudioMixer] playClip: 缺少 audioContext 或 filePath`);
      return;
    }

    const bufferKey = clip.filePath;
    const buffer = this.audioBuffers.get(bufferKey);
    if (!buffer) {
      console.warn(`[AudioMixer] playClip: 找不到音频缓冲 ${clip.id}: ${bufferKey}`);
      return;
    }

    const source = this.audioContext.createBufferSource();
    source.buffer = buffer;

    const gainNode = this.audioContext.createGain();
    const effectiveVolume = clip.isMuted ? 0 : clip.volume * trackVolume;
    gainNode.gain.value = effectiveVolume;

    source.connect(gainNode);
    gainNode.connect(this.audioContext.destination);

    const clipStartTime = clip.startTime;
    const clipOffset = clip.offset || 0;
    const clipDuration = clip.duration;

    // 【P5修复】计算播放开始时间、buffer 偏移量和持续时间
    let when = this.audioContext.currentTime;
    let bufferOffset = clipOffset;
    let bufferDuration = clipDuration;

    if (clipStartTime > playbackPosition) {
      // clip 还没到播放时间，需要延迟
      when = this.audioContext.currentTime + (clipStartTime - playbackPosition);
      console.log(`[AudioMixer] 延迟播放 ${clip.name}: ${clipStartTime - playbackPosition}s 后开始`);
    } else {
      // clip 已经开始了，需要从中间开始播放
      bufferOffset = clipOffset + (playbackPosition - clipStartTime);
      bufferDuration = clipDuration - (playbackPosition - clipStartTime);

      if (bufferDuration <= 0) {
        console.log(`[AudioMixer] 跳过已播放完的片段: ${clip.name}`);
        return;
      }

      console.log(`[AudioMixer] 从中间播放 ${clip.name}: 偏移 ${bufferOffset.toFixed(2)}s, 剩余 ${bufferDuration.toFixed(2)}s`);
    }

    // 淡入效果
    if (clip.fadeIn > 0 && playbackPosition === 0) {
      gainNode.gain.setValueAtTime(0, when);
      gainNode.gain.linearRampToValueAtTime(effectiveVolume, when + clip.fadeIn);
    }

    // 淡出效果
    if (clip.fadeOut > 0 && bufferDuration > clip.fadeOut) {
      const fadeOutStart = when + bufferDuration - clip.fadeOut;
      gainNode.gain.setValueAtTime(effectiveVolume, fadeOutStart);
      gainNode.gain.linearRampToValueAtTime(0, when + bufferDuration);
    }

    try {
      source.start(when, bufferOffset, bufferDuration);
      this.activeSources.set(clip.id, source);

      console.log(`[AudioMixer] ✅ 开始播放: ${clip.name} (when: ${when.toFixed(2)}, offset: ${bufferOffset.toFixed(2)}, duration: ${bufferDuration.toFixed(2)})`);

      source.onended = () => {
        this.activeSources.delete(clip.id);
      };
    } catch (error) {
      console.error(`[AudioMixer] ❌ 播放失败 ${clip.name}:`, error);
    }
  }

  stopClip(clipId: string): void {
    const source = this.activeSources.get(clipId);
    if (source) {
      try {
        source.stop();
        this.activeSources.delete(clipId);
        console.log(`[AudioMixer] 停止片段: ${clipId}`);
      } catch (error) {
        console.error(`[AudioMixer] 停止片段失败 ${clipId}:`, error);
      }
    }
  }

  stopAll(): void {
    console.log(`[AudioMixer] 停止所有片段 (${this.activeSources.size} 个)`);
    this.activeSources.forEach((source, id) => {
      try {
        source.stop();
      } catch (error) {
        console.error(`[AudioMixer] 停止失败 ${id}:`, error);
      }
    });
    this.activeSources.clear();
  }

  // 【P2修复】确保轨道 GainNode 已创建
  playClips(clips: AudioClip[], playbackPosition: number = 0): void {
    this.stopAll();

    console.log(`[AudioMixer] 播放所有片段: ${clips.length} 个, 起始位置: ${playbackPosition}s`);

    for (const clip of clips) {
      // 【P2修复】确保轨道 GainNode 已创建
      if (!this.trackGains.has(clip.trackId)) {
        this.createTrackGain(clip.trackId, 1.0);
      }

      const trackVolume = this.trackVolumes.get(clip.trackId) || 1.0;
      this.playClip(clip, playbackPosition, trackVolume);
    }
  }

  getActiveSources(): string[] {
    return Array.from(this.activeSources.keys());
  }

  isPlaying(): boolean {
    return this.activeSources.size > 0;
  }

  getAudioBuffer(id: string): AudioBuffer | undefined {
    return this.audioBuffers.get(id);
  }

  hasAudioBuffer(id: string): boolean {
    return this.audioBuffers.has(id);
  }

  cleanup(): void {
    this.stopAll();
    this.audioBuffers.clear();
    this.trackGains.clear();
    this.trackVolumes.clear();

    if (this.audioContext) {
      this.audioContext.close();
      this.audioContext = null;
    }

    console.log('[AudioMixer] 清理完成');
  }
}

let mixerInstance: AudioMixerEngine | null = null;

export function getMixerEngine(): AudioMixerEngine {
  if (!mixerInstance) {
    console.log('[AudioMixer] 创建新的 AudioMixerEngine 实例');
    mixerInstance = new AudioMixerEngine();
  }
  return mixerInstance;
}

export function destroyMixerEngine(): void {
  if (mixerInstance) {
    mixerInstance.cleanup();
    mixerInstance = null;
    console.log('[AudioMixer] 实例已销毁');
  }
}
