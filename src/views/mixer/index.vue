<template>
  <div class="mixer-page">
    <!-- 顶部工具栏 -->
    <div class="mixer-toolbar">
      <div class="toolbar-left">
        <h2 class="page-title">
          <el-icon :size="24" color="#1677FF"><Headset /></el-icon>
          音频混音
        </h2>
      </div>

      <div class="toolbar-center">
        <el-button-group>
          <el-button
            :type="activePanel === 'material' ? 'primary' : 'default'"
            @click="togglePanel('material')"
          >
            <el-icon><Document /></el-icon>
            素材选择
          </el-button>
          <el-button
            :type="activePanel === 'paragraph' ? 'primary' : 'default'"
            @click="togglePanel('paragraph')"
          >
            <el-icon><List /></el-icon>
            段落 ({{ paragraphs.length }})
          </el-button>
          <el-button
            :type="activePanel === 'ambient' ? 'primary' : 'default'"
            @click="togglePanel('ambient')"
          >
            <el-icon><Bell /></el-icon>
            环境音
          </el-button>
        </el-button-group>
      </div>

      <div class="toolbar-right">
        <el-button @click="handleSave">
          <el-icon><Folder /></el-icon>
          保存
        </el-button>
        <el-button type="primary" @click="handleExport">
          <el-icon><Upload /></el-icon>
          导出
        </el-button>
      </div>
    </div>

    <!-- 展开面板 -->
    <div v-if="activePanel" class="mixer-panels">
      <!-- 素材选择面板 -->
      <div v-if="activePanel === 'material'" class="panel-content">
        <div class="panel-section">
          <label class="panel-label">选择小说</label>
          <el-select
            v-model="selectedNovelId"
            placeholder="请选择小说"
            filterable
            class="panel-select"
            @change="onNovelChange"
          >
            <el-option
              v-for="novel in novels"
              :key="novel.id"
              :label="novel.title"
              :value="novel.id"
            />
          </el-select>
        </div>

        <div class="panel-section">
          <label class="panel-label">选择章节</label>
          <el-select
            v-model="selectedChapterId"
            placeholder="请先选择小说"
            filterable
            class="panel-select"
            :disabled="!selectedNovelId"
            @change="onChapterChange"
          >
            <el-option
              v-for="chapter in chapters"
              :key="chapter.id"
              :label="chapter.title"
              :value="chapter.id"
            />
          </el-select>
        </div>
      </div>

      <!-- 段落面板 -->
      <div v-if="activePanel === 'paragraph'" class="panel-content paragraph-panel">
        <div class="paragraph-grid">
          <div
            v-for="para in paragraphs"
            :key="para.id"
            class="paragraph-card"
            :class="{
              active: isParagraphActive(para),
              added: isParagraphAdded(para.id)
            }"
            @click="jumpToParagraph(para)"
          >
            <span class="paragraph-card__index">P{{ para.paragraph_index + 1 }}</span>
            <span
              class="paragraph-card__type"
              :class="`paragraph-card__type--${para.type}`"
            >
              {{ getTypeName(para.type) }}
            </span>
            <span class="paragraph-card__status">
              {{ para.status === 'completed' ? '✅' : '⏳' }}
            </span>
            <span class="paragraph-card__action">
              {{ isParagraphAdded(para.id) ? '✓' : '+' }}
            </span>
          </div>

          <div v-if="paragraphs.length === 0" class="paragraph-empty">
            <el-icon :size="32" color="#d9d9d9"><Document /></el-icon>
            <p>请先选择小说和章节</p>
          </div>
        </div>
      </div>

      <!-- 环境音面板 -->
      <div v-if="activePanel === 'ambient'" class="panel-content ambient-panel">
        <div class="ambient-grid">
          <div
            v-for="ambient in filteredAmbientSounds"
            :key="ambient.id"
            class="ambient-card"
            :class="{ added: isAdded(ambient.id) }"
            draggable="true"
            @click="handleAddAmbient(ambient)"
            @dragstart="handleAmbientDragStart($event, ambient)"
          >
            <span class="ambient-card__icon">🌧️</span>
            <span class="ambient-card__name">{{ ambient.name }}</span>
            <span class="ambient-card__meta">{{ ambient.category }}</span>
            <span class="ambient-card__duration">{{ formatDuration(ambient.duration) }}</span>
            <span class="ambient-card__action">
              {{ isAdded(ambient.id) ? '✓' : '+' }}
            </span>
          </div>

          <div v-if="filteredAmbientSounds.length === 0" class="ambient-empty">
            <p>暂无环境音素材</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 时间轴区域 -->
    <div class="mixer-timeline">
      <TimelineEditor />
    </div>

    <!-- 播放控制 -->
    <div class="mixer-controls">
      <PlaybackControls />
    </div>

    <!-- 属性面板 -->
    <div class="mixer-properties">
      <ClipPropertyPanel />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted } from 'vue';
import { invoke, convertFileSrc } from '@tauri-apps/api/core';
import { ElMessage } from 'element-plus';
import { Headset, Document, List, Bell, Folder, Upload } from '@element-plus/icons-vue';
import { useMixerStore } from './stores/mixer';
import { createDefaultClip } from './types';
import TimelineEditor from './components/TimelineEditor.vue';
import PlaybackControls from './components/PlaybackControls.vue';
import ClipPropertyPanel from './components/ClipPropertyPanel.vue';
import './mixer.css';

type PanelType = 'material' | 'paragraph' | 'ambient' | null;

interface Novel {
  id: number;
  title: string;
}

interface Chapter {
  id: number;
  title: string;
}

interface Paragraph {
  id: number;
  paragraph_index: number;
  content: string;
  type: string;
  audio_path: string | null;
  duration: number | null;
  status: string;
}

interface AmbientSound {
  id: number;
  name: string;
  category: string;
  file_path: string;
  duration: number;
}

const store = useMixerStore();

const activePanel = ref<PanelType>(null);
const selectedParagraphId = ref<number | null>(null);
const addedAmbientIds = ref<Set<number>>(new Set());

const novels = ref<Novel[]>([]);
const chapters = ref<Chapter[]>([]);
const paragraphs = ref<Paragraph[]>([]);
const ambientSounds = ref<AmbientSound[]>([]);

const selectedNovelId = ref<number | null>(null);
const selectedChapterId = ref<number | null>(null);

const filteredAmbientSounds = computed(() => ambientSounds.value);

// 监听 store.tracks 变化，同步清除已删除的环境音状态
watch(
  () => store.tracks,
  (newTracks) => {
    // 检查每个已添加的环境音是否还在轨道中
    const currentAmbientIds = new Set<number>();
    for (const track of newTracks) {
      if (track.type === 'ambient') {
        for (const clip of track.clips) {
          if (clip.sourceId !== undefined) {
            currentAmbientIds.add(clip.sourceId);
          }
        }
      }
    }

    // 清除不在轨道中的环境音状态
    for (const id of addedAmbientIds.value) {
      if (!currentAmbientIds.has(id)) {
        addedAmbientIds.value.delete(id);
      }
    }
  },
  { deep: true }
);

onMounted(async () => {
  store.initializeProject();
  await loadNovels();
  await loadAmbientSounds();
});

function togglePanel(panel: PanelType) {
  activePanel.value = activePanel.value === panel ? null : panel;
}

async function loadNovels() {
  try {
    novels.value = await invoke('get_novels');
  } catch (e) {
    console.error('加载小说失败:', e);
  }
}

async function loadAmbientSounds() {
  try {
    ambientSounds.value = await invoke('get_ambient_sounds');
  } catch (e) {
    console.error('加载环境音失败:', e);
  }
}

async function onNovelChange() {
  selectedChapterId.value = null;
  paragraphs.value = [];

  if (selectedNovelId.value) {
    try {
      chapters.value = await invoke('get_chapters', { novelId: selectedNovelId.value });
    } catch (e) {
      ElMessage.error(`加载章节失败: ${e}`);
    }
  }
}

async function onChapterChange() {
  if (!selectedChapterId.value) return;

  store.chapterId = selectedChapterId.value;

  try {
    const chapterParagraphs = await invoke<Paragraph[]>('get_chapter_paragraphs', {
      chapterId: selectedChapterId.value
    });

    paragraphs.value = chapterParagraphs;

    ElMessage.success(`已加载 ${chapterParagraphs.length} 个段落`);
  } catch (e) {
    console.error('加载段落失败:', e);
    ElMessage.error(`加载段落失败: ${e}`);
  }
}

function getTypeName(type: string): string {
  const map: Record<string, string> = {
    narration: '旁白',
    dialogue: '对话',
    environment: '环境',
  };
  return map[type] || type;
}

function isParagraphActive(para: Paragraph): boolean {
  return selectedParagraphId.value === para.id;
}

function isParagraphAdded(paraId: number): boolean {
  const voiceTrack = store.voiceTrack;
  if (!voiceTrack) return false;
  return voiceTrack.clips.some(c => c.sourceId === paraId);
}

function jumpToParagraph(para: Paragraph) {
  selectedParagraphId.value = para.id;

  // 如果段落已添加，跳转到对应片段
  const voiceTrack = store.voiceTrack;
  if (voiceTrack) {
    const clip = voiceTrack.clips.find(c => c.sourceId === para.id);
    if (clip) {
      // 已添加，跳转到片段位置
      store.setCurrentTime(clip.startTime);
      store.selectClip(clip.id);
    } else {
      // 未添加，添加到音轨
      addParagraphToTrack(para);
    }
  }
}

// 添加段落到音轨
async function addParagraphToTrack(para: Paragraph) {
  // 如果已添加则跳过
  if (isParagraphAdded(para.id)) {
    ElMessage.warning('该段落已在音轨中');
    return;
  }

  // 如果没有音频文件则跳过
  if (!para.audio_path) {
    ElMessage.warning('该段落没有音频文件');
    return;
  }

  const voiceTrack = store.voiceTrack;
  if (!voiceTrack) return;

  const audioSrc = convertFileSrc(para.audio_path);

  // 获取音频实际时长
  const actualDuration = await getAudioDuration(audioSrc);
  console.log(`[段落] P${para.paragraph_index + 1} 实际时长: ${actualDuration}s`);

  // 计算插入位置（追加到末尾）
  let currentEndTime = 0;
  for (const clip of voiceTrack.clips) {
    const clipEnd = clip.startTime + clip.duration;
    if (clipEnd > currentEndTime) {
      currentEndTime = clipEnd;
    }
  }

  const clip = createDefaultClip(
    voiceTrack.id,
    'voice',
    `P${para.paragraph_index + 1}`,
    audioSrc,
    currentEndTime,
    actualDuration,
    para.id
  );

  store.addClip(voiceTrack.id, clip);
  store.updateDuration();
  ElMessage.success(`已添加段落 P${para.paragraph_index + 1} (${formatDuration(actualDuration)}) 到音轨`);
}

function isAdded(id: number): boolean {
  return addedAmbientIds.value.has(id);
}

function formatDuration(seconds?: number): string {
  if (!seconds) return '0:00';
  const mins = Math.floor(seconds / 60);
  const secs = Math.floor(seconds % 60);
  return `${mins}:${secs.toString().padStart(2, '0')}`;
}

// 获取音频实际时长
function getAudioDuration(url: string): Promise<number> {
  return new Promise((resolve) => {
    const audio = new Audio();
    audio.src = url;
    audio.onloadedmetadata = () => {
      resolve(audio.duration);
    };
    audio.onerror = () => {
      resolve(10); // 默认10秒
    };
    // 超时处理
    setTimeout(() => resolve(10), 3000);
  });
}

// 环境音拖拽开始
function handleAmbientDragStart(e: DragEvent, ambient: AmbientSound) {
  const audioSrc = convertFileSrc(ambient.file_path);
  const dragData = {
    type: 'ambient-sound',
    ambientId: ambient.id,
    name: ambient.name,
    filePath: audioSrc,
    duration: ambient.duration || 10
  };

  e.dataTransfer!.setData('application/json', JSON.stringify(dragData));
  e.dataTransfer!.effectAllowed = 'copy';
  console.log('[Mixer] 开始拖拽环境音:', ambient.name);
}

async function handleAddAmbient(ambient: AmbientSound) {
  if (isAdded(ambient.id)) return;

  const audioSrc = convertFileSrc(ambient.file_path);

  let targetTrack = store.ambientTracks.find(t => !t.isMuted);

  if (!targetTrack) {
    store.addTrack('ambient');
    targetTrack = store.ambientTracks.find(t => !t.isMuted);
  }

  if (!targetTrack) {
    ElMessage.warning('没有可用的环境音轨道');
    return;
  }

  const currentTime = store.currentTime;

  // 获取音频实际时长
  const actualDuration = await getAudioDuration(audioSrc);
  console.log(`[环境音] ${ambient.name} 实际时长: ${actualDuration}s`);

  const clip = createDefaultClip(
    targetTrack.id,
    'ambient',
    ambient.name,
    audioSrc,
    currentTime,
    actualDuration,
    ambient.id
  );

  store.addClip(targetTrack.id, clip);
  addedAmbientIds.value.add(ambient.id);
  ElMessage.success(`已添加环境音: ${ambient.name} (${formatDuration(actualDuration)})`);
}

function handleSave() {
  ElMessage.info('保存功能开发中...');
}

function handleExport() {
  ElMessage.info('导出功能开发中...');
}
</script>

<style scoped>
.mixer-page {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-base);
}

/* 工具栏 */
.mixer-toolbar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-4) var(--page-padding);
  background: var(--color-bg-container);
  border-bottom: 1px solid var(--color-border-light);
  flex-shrink: 0;
}

.toolbar-left,
.toolbar-center,
.toolbar-right {
  display: flex;
  align-items: center;
  gap: var(--space-4);
}

.page-title {
  display: flex;
  align-items: center;
  gap: var(--space-3);
  font-size: var(--text-xl);
  font-weight: var(--font-semibold);
  color: var(--color-text-primary);
  margin: 0;
}

/* 面板 */
.mixer-panels {
  background: var(--color-bg-container);
  border-bottom: 1px solid var(--color-border-light);
  padding: var(--space-4) var(--page-padding);
  flex-shrink: 0;
}

.panel-content {
  display: flex;
  gap: var(--space-6);
}

.panel-section {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.panel-label {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  white-space: nowrap;
}

.panel-select {
  width: 200px;
}

/* 段落面板 */
.paragraph-panel {
  max-height: 160px;
  overflow-y: auto;
}

.paragraph-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.paragraph-card {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--color-bg-lighter);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.paragraph-card:hover {
  border-color: var(--color-primary-400);
}

.paragraph-card.active {
  background: var(--color-primary-50);
  border-color: var(--color-primary-500);
}

.paragraph-card.added {
  background: var(--color-success-bg);
  border-color: var(--color-success);
}

.paragraph-card__index {
  font-weight: var(--font-semibold);
  color: var(--color-primary-500);
  font-size: var(--text-sm);
}

.paragraph-card__status {
  font-size: var(--text-sm);
}

.paragraph-card__action {
  font-size: var(--text-base);
  color: var(--color-primary-500);
  margin-left: auto;
}

.paragraph-card.added .paragraph-card__action {
  color: var(--color-success);
}

.paragraph-empty {
  flex: 1;
  text-align: center;
  padding: var(--space-6);
  color: var(--color-text-placeholder);
}

/* 环境音面板 */
.ambient-panel {
  max-height: 200px;
  overflow-y: auto;
}

.ambient-grid {
  display: flex;
  flex-wrap: wrap;
  gap: var(--space-2);
}

.ambient-card {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-3);
  background: var(--color-bg-lighter);
  border: 1px solid var(--color-border-light);
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-fast);
  flex-shrink: 0;
}

.ambient-card:hover {
  border-color: var(--color-primary-400);
}

.ambient-card.added {
  background: var(--color-success-bg);
  border-color: var(--color-success);
}

.ambient-card__icon {
  font-size: 20px;
}

.ambient-card__name {
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  color: var(--color-text-primary);
}

.ambient-card__meta {
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
}

.ambient-card__duration {
  font-size: var(--text-xs);
  color: var(--color-text-secondary);
}

.ambient-card__action {
  font-size: var(--text-base);
  color: var(--color-primary-500);
  margin-left: auto;
}

.ambient-card.added .ambient-card__action {
  color: var(--color-success);
}

.ambient-empty {
  flex: 1;
  text-align: center;
  padding: var(--space-6);
  color: var(--color-text-placeholder);
}

/* 时间轴区域 */
.mixer-timeline {
  flex: 1;
  overflow: hidden;
}

/* 播放控制 */
.mixer-controls {
  flex-shrink: 0;
}

/* 属性面板 */
.mixer-properties {
  flex-shrink: 0;
}
</style>
