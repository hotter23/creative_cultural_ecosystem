<template>
  <div class="audio-page">
    <el-card shadow="never" class="novel-selector">
      <template #header>
        <span>选择小说</span>
      </template>
      <el-select v-model="selectedNovelId" placeholder="请选择小说" style="width: 100%" @change="loadChapters">
        <el-option v-for="novel in novels" :key="novel.id" :label="novel.title" :value="novel.id" />
      </el-select>
    </el-card>

    <el-card shadow="never" class="chapter-list" style="margin-top: 20px" v-if="selectedNovelId">
      <template #header>
        <span>章节列表</span>
      </template>
      <el-table :data="chapters" style="width: 100%">
        <el-table-column prop="title" label="章节标题" />
        <el-table-column prop="word_count" label="字数" width="100" />
        <el-table-column label="音频状态" width="120">
          <template #default="scope">
            <el-tag :type="scope.row.audio_status === 'completed' ? 'success' : scope.row.audio_status === 'processing' ? 'warning' : 'info'">
              {{ scope.row.audio_status === 'completed' ? '已完成' : scope.row.audio_status === 'processing' ? '生成中' : '未生成' }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column label="操作" width="200">
          <template #default="scope">
            <el-button size="small" @click="openAudioDetail(scope.row)">
              {{ scope.row.audio_status === 'not_created' ? '生成音频' : '查看音频' }}
            </el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <el-dialog v-model="audioDialogVisible" title="音频制作" width="85%" top="3vh" height="92vh" :close-on-click-modal="false">
      <div class="audio-detail">
        <el-descriptions :column="2" border>
          <el-descriptions-item label="章节">{{ currentChapter?.title }}</el-descriptions-item>
          <el-descriptions-item label="状态">
            <el-tag :type="getStatusTagType(currentAudio?.status)">
              {{ getStatusText(currentAudio?.status) }}
            </el-tag>
          </el-descriptions-item>
          <el-descriptions-item label="总段落数">{{ paragraphs.length }}</el-descriptions-item>
          <el-descriptions-item label="已完成">{{ completedCount }} / {{ paragraphs.length }}</el-descriptions-item>
        </el-descriptions>
        
        <div v-if="paragraphs.length > 0" style="margin-top: 15px; padding: 0 5px;">
          <div style="display: flex; justify-content: space-between; margin-bottom: 5px; font-size: 13px;">
            <span>制作进度</span>
            <span>{{ Math.round(completedCount / paragraphs.length * 100) }}% ({{ completedCount }}/{{ paragraphs.length }})</span>
          </div>
          <el-progress 
            :percentage="Math.round(completedCount / paragraphs.length * 100)" 
            :status="completedCount === paragraphs.length ? 'success' : undefined"
            :stroke-width="12"
          />
        </div>

        <el-card shadow="never" class="paragraph-list" style="margin-top: 20px">
          <template #header>
            <div class="card-header">
              <span>段落列表</span>
              <div>
                <el-button v-if="currentAudio" size="small" @click="refreshParagraphs">
                  <el-icon><Refresh /></el-icon>
                  刷新
                </el-button>
                <el-button v-if="!currentAudio" type="primary" size="small" @click="generateAudio">
                  <el-icon><VideoPlay /></el-icon>
                  生成音频
                </el-button>
              </div>
            </div>
          </template>
          <div class="paragraphs-container" v-loading="paragraphsLoading">
            <div v-for="para in paragraphs" :key="para.id" class="paragraph-item" :class="{ 'is-dialogue': para.type === 'dialogue' }">
              <div class="para-header">
                <span class="para-index">{{ para.paragraph_index + 1 }}.</span>
                <el-tag size="small" :type="getParagraphTypeTagType(para.type)" style="margin-right: 8px">
                  {{ getParagraphTypeLabel(para.type) }}
                </el-tag>
                <el-tag v-if="para.character_id" size="small" type="warning" style="margin-right: 8px">
                  {{ getCharacterName(para.character_id) }}
                </el-tag>
                <el-tag v-if="getCharacterVoiceName(para.character_id)" size="small" type="info" style="margin-right: 8px">
                  {{ getCharacterVoiceName(para.character_id) }}
                </el-tag>
                <el-tag size="small" :type="getSentenceStatusType(para.status)" style="margin-right: 8px">
                  {{ getSentenceStatusText(para.status) }}
                </el-tag>
              </div>
              <div class="para-content">{{ para.content }}</div>
              <div class="para-controls" v-if="currentAudio">
                <template v-if="para.type === 'environment'">
                  <el-select v-model="para.ambient_sound_id" size="small" style="width: 200px; margin-right: 10px" placeholder="选择环境音" clearable filterable @change="updateParagraphAmbient(para)">
                    <el-option v-for="sound in ambientSounds" :key="sound.id" :label="sound.name" :value="sound.id">
                      <span style="float: left">{{ sound.name }}</span>
                      <span style="float: right; color: #8492a6; font-size: 12px">{{ sound.duration.toFixed(1) }}s</span>
                    </el-option>
                  </el-select>
                  <span v-if="para.ambient_sound_id" style="color: #67c23a; margin-right: 10px">
                    已选择: {{ ambientSounds.find(s => s.id === para.ambient_sound_id)?.name }}
                  </span>
                  <el-button v-if="para.ambient_sound_id && para.audio_path" size="small" @click="playAudio(para)" style="margin-left: 10px">
                    <el-icon><Headset /></el-icon>
                    播放
                  </el-button>
                  <el-button v-else-if="para.ambient_sound_id" size="small" type="primary" @click="generateAmbientForParagraph(para)" style="margin-left: 10px">
                    <el-icon><VideoPlay /></el-icon>
                    生成
                  </el-button>
                </template>
                <template v-else>
                  <div class="voice-display">
                    <span class="voice-icon"><el-icon><Microphone /></el-icon></span>
                    <span class="voice-name" :class="{ 'no-voice': !getCharacterVoiceName(para.character_id) }">
                      {{ getCharacterVoiceName(para.character_id) || (getCharacterName(para.character_id) ? '未绑定音色' : '默认音色') }}
                    </span>
                  </div>
                 
                  <el-slider v-model="para.speed" :min="0.5" :max="2" :step="0.1" style="width: 120px; margin-left: 10px; margin-right: 10px" @change="updateParagraphVoice(para)" />
                  <span class="speed-label">{{ para.speed.toFixed(1) }}x</span>
                </template>
              </div>
              <div class="para-actions">
                <el-button size="small" type="primary" @click="regenerateParagraph(para)">
                  <el-icon><VideoPlay /></el-icon>
                  {{ para.status === 'completed' ? '重新生成' : (para.status === 'failed' ? '重试' : '生成音频') }}
                </el-button>
                <el-button v-if="para.audio_path" size="small" @click="playAudio(para)">
                  <el-icon><Headset /></el-icon>
                  播放
                </el-button>
                <el-button v-if="para.task_id && para.status !== 'completed'" size="small" @click="queryStatus(para)">
                  <el-icon><Refresh /></el-icon>
                  查询进度
                </el-button>
              </div>
              <div class="para-task-info" v-if="para.task_id || para.task_token">
                <span v-if="para.task_id">任务ID: {{ para.task_id }}</span>
                <span v-if="para.task_token" style="margin-left: 10px">Token: {{ para.task_token.substring(0, 10) }}...</span>
              </div>
              <div class="para-error" v-if="para.error_msg">
                <el-text type="danger">{{ para.error_msg }}</el-text>
              </div>
            </div>
            <el-empty v-if="paragraphs.length === 0 && !paragraphsLoading" description="该章节暂无手动标注，请先在章节编辑中进行标注" />
          </div>
        </el-card>
      </div>

      <template #footer>
        <div class="dialog-footer">
          <el-button @click="audioDialogVisible = false">关闭</el-button>
          <el-button 
            v-if="currentAudio && completedCount > 0 && currentAudio.status !== 'completed'" 
            :loading="mergingAudio" 
            @click="mergeAudio"
          >
            <el-icon><VideoPlay /></el-icon>
            合并音频 ({{ completedCount }}/{{ paragraphs.length }})
          </el-button>
          <template v-if="currentAudio?.status === 'completed'">
            <el-button type="warning" @click="mergeAudio" :loading="mergingAudio">
              <el-icon><Refresh /></el-icon>
              重新合并
            </el-button>
            <el-button type="primary" @click="exportAudio">
              <el-icon><Download /></el-icon>
              导出音频
            </el-button>
          </template>
        </div>
      </template>
    </el-dialog>

    <el-dialog v-model="playerVisible" title="音频播放" width="500px">
      <div class="audio-player">
        <div class="player-text">{{ playingParagraph?.content }}</div>
        <div class="player-path">文件路径: {{ playingParagraph?.audio_path }}</div>
        <audio ref="audioRef" :src="playingAudioUrl" controls style="width: 100%; margin-top: 20px" />
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'
import { Refresh, VideoPlay, Headset, Download, Microphone } from '@element-plus/icons-vue'

interface Novel {
  id: number
  title: string
}

interface Chapter {
  id: number
  novel_id: number
  title: string
  content?: string
  word_count: number
  audio_status: string
}

interface ChapterAudio {
  id: number
  novel_id: number
  chapter_id?: number
  status: string
  total_sentences: number
  completed_sentences: number
  merged_audio_path?: string
  created_at: string
}

interface Character {
  id: number
  novel_id: number
  name: string
  voice_id?: string
}

interface AmbientSound {
  id: number
  name: string
  description?: string
  category: string
  prompt?: string
  file_path: string
  duration: number
  volume: number
  is_loopable: boolean
  is_system: boolean
}

interface ChapterParagraph {
  id: number
  chapter_id: number
  paragraph_index: number
  content: string
  type: string
  character_id?: number
  created_at: string
  updated_at: string
  audio_id?: number
  speed: number
  pitch: number
  volume: number
  emotion: string
  audio_path?: string
  duration?: number
  status: string
  error_msg?: string
  task_id?: string
  task_token?: string
  ambient_sound_id?: number
}

const novels = ref<Novel[]>([])
const selectedNovelId = ref<number | null>(null)
const chapters = ref<Chapter[]>([])
const loading = ref(false)
const paragraphsLoading = ref(false)

const audioDialogVisible = ref(false)
const playerVisible = ref(false)
const mergingAudio = ref(false)

const currentChapter = ref<Chapter | null>(null)
const currentAudio = ref<ChapterAudio | null>(null)
const paragraphs = ref<ChapterParagraph[]>([])
const characters = ref<Character[]>([])
const characterMap = ref<Map<number, Character>>(new Map())
const ambientSounds = ref<AmbientSound[]>([])

const playingParagraph = ref<ChapterParagraph | null>(null)
const playingAudioUrl = ref('')
const audioRef = ref<HTMLAudioElement | null>(null)

const generateConfig = reactive({
  voiceId: 'female-tianmei',
  speed: 1.0
})

const completedCount = computed(() => {
  return paragraphs.value.filter(p => p.status === 'completed').length
})

const voiceMap = computed(() => {
  const map: Record<string, string> = {}
  voiceCategories.value.forEach((group: any) => {
    group.options.forEach((option: any) => {
      map[option.id] = option.name
    })
  })
  return map
})

const voiceCategories = ref<any[]>([])

const loadVoiceCategories = async () => {
  try {
    const result = await invoke<any>('get_voice_list')
    if (result.success) {
      voiceCategories.value = result.voice_categories
    }
  } catch (e) {
    console.error('加载音色列表失败:', e)
  }
}

const getStatusTagType = (status?: string) => {
  switch (status) {
    case 'completed': return 'success'
    case 'processing': return 'warning'
    case 'failed': return 'danger'
    default: return 'info'
  }
}

const getStatusText = (status?: string) => {
  switch (status) {
    case 'completed': return '已完成'
    case 'processing': return '处理中'
    case 'failed': return '失败'
    default: return '待生成'
  }
}

const getSentenceStatusType = (status: string) => {
  switch (status) {
    case 'completed': return 'success'
    case 'processing': return 'warning'
    case 'failed': return 'danger'
    default: return 'info'
  }
}

const getSentenceStatusText = (status: string) => {
  switch (status) {
    case 'completed': return '已完成'
    case 'processing': return '生成中'
    case 'failed': return '失败'
    default: return '待生成'
  }
}

const getParagraphTypeLabel = (type: string) => {
  switch (type) {
    case 'narration': return '旁白'
    case 'dialogue': return '对话'
    case 'environment': return '环境音'
    default: return type
  }
}

const getParagraphTypeTagType = (type: string) => {
  switch (type) {
    case 'narration': return 'info'
    case 'dialogue': return 'success'
    case 'environment': return 'warning'
    default: return 'info'
  }
}

const getCharacterName = (characterId?: number) => {
  if (!characterId) return ''
  const char = characterMap.value.get(characterId)
  return char ? char.name : ''
}

const getCharacterVoiceName = (characterId?: number) => {
  if (!characterId) return ''
  const char = characterMap.value.get(characterId)
  if (char && char.voice_id) {
    return voiceMap.value[char.voice_id] || char.voice_id
  }
  return ''
}

const loadNovels = async () => {
  loading.value = true
  try {
    novels.value = await invoke('get_novels')
  } catch (e) {
    ElMessage.error('加载小说列表失败')
  } finally {
    loading.value = false
  }
}

const loadChapters = async () => {
  if (!selectedNovelId.value) return
  loading.value = true
  try {
    chapters.value = await invoke('get_chapters', { novel_id: selectedNovelId.value, novelId: selectedNovelId.value })
  } catch (e) {
    ElMessage.error('加载章节列表失败')
  } finally {
    loading.value = false
  }
}

const loadCharacters = async (novelId: number) => {
  try {
    const result = await invoke<Character[]>('get_characters', {
      novel_id: novelId,
      novelId: novelId
    })
    characters.value = result
    const map = new Map<number, Character>()
    result.forEach(char => {
      map.set(char.id, char)
    })
    characterMap.value = map
  } catch (e) {
    console.error('加载角色列表失败:', e)
  }
}

const loadAmbientSounds = async () => {
  try {
    const result = await invoke<AmbientSound[]>('get_ambient_sounds', {
      category: ''
    })
    ambientSounds.value = result
  } catch (e) {
    console.error('加载环境音列表失败:', e)
  }
}

const loadAudioParagraphs = async (chapterId: number) => {
  paragraphsLoading.value = true
  try {
    const result = await invoke<ChapterParagraph[]>('get_audio_paragraphs', { chapter_id: chapterId, chapterId: chapterId })
    console.log('原始段落数据:', result)
    paragraphs.value = result
  } catch (e) {
    console.error('加载音频段落失败:', e)
  } finally {
    paragraphsLoading.value = false
  }
}

const openAudioDetail = async (chapter: Chapter) => {
  currentChapter.value = chapter
  await loadCharacters(chapter.novel_id)
  await loadAmbientSounds()
  
  try {
    await loadAudioParagraphs(chapter.id)
    if (chapter.audio_status !== 'not_created') {
      currentAudio.value = await invoke('get_chapter_audio_detail', { chapter_id: chapter.id, chapterId: chapter.id })
    } else {
      currentAudio.value = null
    }
  } catch (e) {
    console.error('加载音频详情失败:', e)
  }
  
  audioDialogVisible.value = true
}

const generateAudio = async () => {
  if (!currentChapter.value) return
  
  try {
    const audioId = await invoke<number>('generate_chapter_audio', {
      novel_id: currentChapter.value.novel_id,
      chapter_id: currentChapter.value.id,
      chapter_title: currentChapter.value.title,
      chapter_content: currentChapter.value.content || '',
      voice_id: generateConfig.voiceId,
      speed: generateConfig.speed
    })
    
    currentAudio.value = await invoke('get_chapter_audio_detail', { chapter_id: currentChapter.value.id, chapterId: currentChapter.value.id })
    await loadAudioParagraphs(currentChapter.value.id)
    
    ElMessage.success('音频任务创建成功')
    await loadChapters()
  } catch (e) {
    ElMessage.error(`创建音频任务失败: ${e}`)
  }
}

const refreshParagraphs = async () => {
  if (currentAudio.value) {
    await loadAudioParagraphs(currentAudio.value.id)
  }
}

const updateParagraphVoice = async (para: ChapterParagraph) => {
  try {
    await invoke('update_paragraph_params', {
      paragraph_id: para.id,
      speed: para.speed
    })
  } catch (e) {
    ElMessage.error('更新参数失败')
  }
}

const updateParagraphAmbient = async (para: ChapterParagraph) => {
  try {
    await invoke('update_paragraph_ambient_sound', {
      paragraph_id: para.id,
      paragraphId: para.id,
      ambient_sound_id: para.ambient_sound_id,
      ambientSoundId: para.ambient_sound_id
    })
    ElMessage.success('环境音已更新')
  } catch (e) {
    ElMessage.error('更新环境音失败')
  }
}

const generateAmbientForParagraph = async (para: ChapterParagraph) => {
  if (!para.ambient_sound_id) {
    ElMessage.warning('请先选择环境音')
    return
  }
  
  const ambientSound = ambientSounds.value.find(s => s.id === para.ambient_sound_id)
  if (!ambientSound) {
    ElMessage.error('未找到选中的环境音')
    return
  }
  
  try {
    para.status = 'processing'
    await invoke('copy_ambient_to_paragraph', {
      paragraph_id: para.id,
      paragraphId: para.id,
      ambient_sound_id: para.ambient_sound_id,
      ambientSoundId: para.ambient_sound_id
    })
    para.audio_path = ambientSound.file_path
    para.status = 'completed'
    ElMessage.success('环境音已应用')
  } catch (e) {
    para.status = 'failed'
    ElMessage.error(`应用环境音失败: ${e}`)
  }
}

const regenerateParagraph = async (para: ChapterParagraph) => {
  try {
    // 如果 currentAudio 为 null，先创建音频任务
    if (!currentAudio.value) {
      await generateAudio()
    }
    
    // 如果创建失败或仍然没有 currentAudio，退出
    if (!currentAudio.value) {
      ElMessage.error('创建音频任务失败，无法生成音频')
      return
    }
    
    let voiceId = generateConfig.voiceId
    if (para.character_id) {
      const char = characterMap.value.get(para.character_id)
      if (char && char.voice_id) {
        voiceId = char.voice_id
      }
    }
    
    const taskId = await invoke<string>('regenerate_paragraph_audio', {
      paragraph_id: para.id,
      paragraphId: para.id,
      text: para.content,
      voice_id: voiceId,
      voiceId: voiceId,
      speed: para.speed || generateConfig.speed
    })
    para.task_id = taskId
    para.status = 'processing'
    ElMessage.success(`任务创建成功，task_id: ${taskId}`)
  } catch (e) {
    ElMessage.error(`创建任务失败: ${e}`)
  }
}

const queryStatus = async (para: ChapterParagraph) => {
  if (!para.task_id) return
  
  try {
    const status = await invoke<string>('query_audio_status', { task_id: para.task_id,taskId: para.task_id })
    ElMessage.success(`任务状态: ${status}`)
    
    if (status.includes('任务已完成')) {
      await downloadParagraphAudio(para)
    }
  } catch (e) {
    ElMessage.error(`查询状态失败: ${e}`)
  }
}

const downloadParagraphAudio = async (para: ChapterParagraph) => {
  try {
    await invoke('download_paragraph_audio', { paragraph_id: para.id,paragraphId: para.id })
    if (currentChapter.value) {
      await loadAudioParagraphs(currentChapter.value.id)
    }
    await loadChapters()
    ElMessage.success('音频下载成功')
  } catch (e) {
    ElMessage.error(`下载音频失败: ${e}`)
  }
}

const playAudio = async (para: ChapterParagraph) => {
  if (!para.audio_path) {
    ElMessage.warning('该段落暂无音频文件')
    return
  }
  
  playingParagraph.value = para
  
  try {
    const audioSrc = convertFileSrc(para.audio_path)
    playingAudioUrl.value = audioSrc
    playerVisible.value = true
    
    await new Promise(resolve => setTimeout(resolve, 100))
    
    if (audioRef.value) {
      audioRef.value.load()
      try {
        await audioRef.value.play()
      } catch (playError) {
        console.warn('自动播放失败:', playError)
      }
    }
  } catch (error) {
    ElMessage.error(`播放音频失败: ${error}`)
  }
}

const exportAudio = async () => {
  try {
    const audioDir = await invoke<string>('get_audio_storage_dir')
    ElMessage.info(`音频文件存储在: ${audioDir}`)
  } catch (e) {
    console.error('获取存储目录失败:', e)
  }
}

const mergeAudio = async () => {
  if (!currentAudio.value || !currentChapter.value) {
    ElMessage.warning('请先选择章节并创建音频任务')
    return
  }
  
  const completed = paragraphs.value.filter(p => p.status === 'completed').length
  if (completed === 0) {
    ElMessage.warning('没有已完成的音频段落可以合并')
    return
  }
  
  mergingAudio.value = true
  try {
    const mergedPath = await invoke<string>('merge_chapter_audio', {
      audio_id: currentAudio.value.id,
      audioId: currentAudio.value.id,
      chapter_id: currentChapter.value.id,
      chapterId: currentChapter.value.id
    })
    
    currentAudio.value = await invoke('get_chapter_audio_detail', { 
      chapter_id: currentChapter.value.id, 
      chapterId: currentChapter.value.id 
    })
    
    await loadChapters()
    
    ElMessage.success(`音频合并成功！文件已保存到: ${mergedPath}`)
  } catch (e: any) {
    ElMessage.error(`音频合并失败: ${e}`)
  } finally {
    mergingAudio.value = false
  }
}

onMounted(async () => {
  await Promise.all([
    loadNovels(),
    loadVoiceCategories()
  ])
})
</script>

<style scoped>
.audio-page {
  padding: 20px;
}

.novel-selector {
  margin-bottom: 20px;
}

.chapter-list {
  margin-bottom: 20px;
}

.audio-detail {
  padding: 10px;
  display: flex;
  flex-direction: column;
  height: calc(92vh - 120px);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.paragraph-list {
  margin-top: 20px;
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.paragraphs-container {
  flex: 1;
  overflow-y: auto;
  padding-right: 10px;
}

.paragraph-item {
  padding: 15px;
  border-bottom: 1px solid #ebeef5;
  transition: background-color 0.2s;
}

.paragraph-item:hover {
  background-color: #f5f7fa;
}

.paragraph-item.is-dialogue:hover {
  background-color: #d9ecff;
}

.paragraph-item.is-dialogue {
  background-color: #ecf5ff;
}

.para-header {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.para-index {
  font-weight: bold;
  color: #606266;
  margin-right: 10px;
  min-width: 30px;
}

.para-content {
  color: #303133;
  line-height: 1.6;
  margin-bottom: 10px;
}

.para-controls {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
}

.voice-display {
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  background: #f0f9eb;
  border-radius: 4px;
  border: 1px solid #e1f3d8;
}

.voice-icon {
  display: flex;
  align-items: center;
  margin-right: 6px;
  color: #67c23a;
}

.voice-name {
  font-size: 13px;
  color: #606266;
}

.voice-name.no-voice {
  color: #909399;
  font-style: italic;
}

.speed-label {
  font-size: 13px;
  color: #606266;
  min-width: 36px;
}

.para-actions {
  display: flex;
  gap: 8px;
}

.para-task-info {
  margin-top: 8px;
  font-size: 12px;
  color: #909399;
}

.para-error {
  margin-top: 8px;
  font-size: 12px;
}

.audio-player {
  text-align: center;
}

.player-text {
  padding: 20px;
  background-color: #f5f7fa;
  border-radius: 4px;
  line-height: 1.6;
  margin-bottom: 10px;
}

.player-path {
  padding: 10px;
  background-color: #fdf6ec;
  border: 1px solid #f5dab1;
  border-radius: 4px;
  font-size: 12px;
  color: #e6a23c;
  word-break: break-all;
  margin-bottom: 10px;
}

.dialog-footer {
  display: flex;
  justify-content: flex-end;
  gap: 10px;
}
</style>
