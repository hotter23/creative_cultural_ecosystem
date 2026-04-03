<template>
  <div class="ambient-page">
    <el-card shadow="never" class="page-header">
      <template #header>
        <span>环境音管理</span>
      </template>
      <p style="color: #606266; margin: 0">AI 驱动的环境音生成、管理与混音功能</p>
    </el-card>

    <el-tabs v-model="activeTab" style="margin-top: 20px;">
      <!-- 环境音生成标签页 -->
      <el-tab-pane label="AI 环境音生成" name="generate">
        <el-card shadow="never">
          <template #header>
            <span>生成自定义环境音</span>
          </template>
          
          <div class="ambient-generate-form">
            <el-row :gutter="20">
              <el-col :md="12" :sm="24">
                <el-form :model="ambientForm" label-width="100px">
                  <el-form-item label="场景描述">
                    <el-input 
                      v-model="ambientForm.prompt" 
                      type="textarea" 
                      :rows="4" 
                      placeholder="描述您想要生成的环境音场景，例如：'rainy cafe with people talking softly in background'"
                    />
                  </el-form-item>
                  <el-form-item label="快捷场景">
                    <el-tag 
                      v-for="scene in quickScenes" 
                      :key="scene" 
                      class="scene-tag" 
                      size="large"
                      @click="selectQuickScene(scene)"
                      effect="plain"
                    >
                      {{ scene }}
                    </el-tag>
                  </el-form-item>
                  <el-form-item label="持续时长(秒)">
                    <el-slider v-model="ambientForm.duration" :min="2" :max="30" :step="1" style="width: 300px" />
                    <span style="margin-left: 15px">{{ ambientForm.duration }} 秒</span>
                  </el-form-item>
                  <el-form-item label="名称(可选)">
                    <el-input v-model="ambientForm.name" placeholder="为生成的环境音命名" style="width: 300px" />
                  </el-form-item>
                  <el-form-item>
                    <el-button 
                      type="primary" 
                      :loading="generatingAmbient" 
                      @click="generateAmbientSound"
                      size="large"
                    >
                      <el-icon><VideoPlay /></el-icon>
                      {{ generatingAmbient ? '生成中...' : '生成环境音' }}
                    </el-button>
                    <el-button 
                      v-if="generatingAmbient" 
                      size="large" 
                      @click="cancelGeneration"
                      style="margin-left: 10px"
                    >
                      取消
                    </el-button>
                  </el-form-item>
                </el-form>
              </el-col>
              
              <el-col :md="12" :sm="24">
                <!-- 生成结果 -->
                <div v-if="ambientResult" class="ambient-result">
                  <el-alert 
                    :title="ambientResult.success ? '生成成功！' : '生成失败'" 
                    :type="ambientResult.success ? 'success' : 'error'" 
                    style="margin-bottom: 15px"
                  />
                  <div v-if="ambientResult.success" class="result-player">
                    <h4 style="margin-bottom: 15px">生成的环境音</h4>
                    <audio :src="getAmbientAudioUrl(ambientResult.file_path)" controls style="width: 100%" />
                    <p style="margin-top: 15px; color: #606266; font-size: 13px">
                      <el-icon style="vertical-align: middle; margin-right: 5px"><Location /></el-icon>
                      文件路径: {{ ambientResult.file_path }}
                    </p>
                    <p style="margin-top: 5px; color: #606266; font-size: 13px">
                      <el-icon style="vertical-align: middle; margin-right: 5px"><Clock /></el-icon>
                      时长: {{ ambientResult.duration }} 秒
                    </p>
                  </div>
                </div>
                
                <div v-else class="generate-tips">
                  <el-alert title="Stable Audio Open 生成模式" type="info">
                    <template #default>
                      <ul style="padding-left: 20px; margin: 0">
                        <li>🎵 使用 AI 模型生成高质量环境音</li>
                        <li>💡 输入英文描述效果更好，如 "rainy cafe with soft background music"</li>
                        <li>⚠️ 首次使用会下载模型（约 2-3GB），需要 GPU</li>
                        <li>⏱️ 生成时间约 30-60 秒</li>
                      </ul>
                    </template>
                  </el-alert>
                </div>
              </el-col>
            </el-row>
          </div>
        </el-card>
      </el-tab-pane>

      <!-- 环境音库标签页 -->
      <el-tab-pane label="环境音库" name="library">
        <el-card shadow="never">
          <template #header>
            <div class="card-header">
              <span>环境音素材库</span>
              <div>
                <el-button size="small" @click="loadAmbientSounds">
                  <el-icon><Refresh /></el-icon>
                  刷新
                </el-button>
              </div>
            </div>
          </template>
          
          <div class="ambient-filters">
            <el-select v-model="ambientFilter.category" placeholder="选择分类" size="small" clearable style="width: 150px">
              <el-option label="自然" value="nature" />
              <el-option label="城市" value="urban" />
              <el-option label="室内" value="indoor" />
              <el-option label="天气" value="weather" />
              <el-option label="自定义" value="custom" />
            </el-select>
            <el-input v-model="ambientFilter.search" placeholder="搜索环境音" size="small" style="width: 250px" clearable />
            <span style="color: #909399; font-size: 13px">
              共 {{ filteredAmbientSounds.length }} 个环境音素材
            </span>
          </div>

          <div class="ambient-library-container" v-loading="ambientLibraryLoading">
            <el-empty v-if="filteredAmbientSounds.length === 0 && !ambientLibraryLoading" description="暂无环境音素材" />
            
            <el-row :gutter="15">
              <el-col :xs="12" :sm="8" :md="6" :lg="4" v-for="sound in filteredAmbientSounds" :key="sound.id">
                <el-card class="ambient-sound-card" shadow="hover">
                  <div class="sound-icon">
                    <el-icon size="30"><Bell /></el-icon>
                  </div>
                  <div class="sound-info">
                    <div class="sound-name" :title="sound.name">{{ sound.name }}</div>
                    <div class="sound-meta">
                      <el-tag size="small" type="info">{{ sound.category }}</el-tag>
                      <span class="sound-duration">{{ formatDuration(sound.duration) }}</span>
                    </div>
                    <div class="sound-desc" v-if="sound.description" :title="sound.description">
                      {{ sound.description }}
                    </div>
                    <el-tag v-if="sound.is_system" size="small" type="success" style="margin-top: 5px">系统预置</el-tag>
                  </div>
                  <div class="sound-actions">
                    <el-button size="small" @click="playAmbientSound(sound)" title="播放">
                      <el-icon><Headset /></el-icon>
                    </el-button>
                    <el-button 
                      size="small" 
                      type="danger" 
                      :disabled="sound.is_system"
                      @click="deleteAmbientSound(sound)"
                      title="删除"
                    >
                      <el-icon><Delete /></el-icon>
                    </el-button>
                  </div>
                </el-card>
              </el-col>
            </el-row>
          </div>
        </el-card>
      </el-tab-pane>

      <!-- 混音标签页 -->
      <el-tab-pane label="环境音混音" name="mix">
        <el-card shadow="never">
          <template #header>
            <span>人声与环境音混音</span>
          </template>
          
          <div class="mix-form">
            <el-row :gutter="20">
              <el-col :md="12" :sm="24">
                <el-alert title="混音说明" type="info" style="margin-bottom: 20px">
                  <ul style="padding-left: 20px; margin: 0">
                    <li>选择已完成音频制作的章节</li>
                    <li>选择一个环境音素材</li>
                    <li>调整参数后进行混音</li>
                    <li>生成包含人声和背景环境音的完整音频</li>
                  </ul>
                </el-alert>

                <el-form :model="mixConfig" label-width="120px">
                  <el-form-item label="选择小说">
                    <el-select 
                      v-model="mixNovelId" 
                      placeholder="请选择小说" 
                      style="width: 300px"
                      @change="loadMixChapters"
                      clearable
                    >
                      <el-option v-for="novel in novels" :key="novel.id" :label="novel.title" :value="novel.id" />
                    </el-select>
                  </el-form-item>
                  <el-form-item label="选择章节">
                    <el-select 
                      v-model="mixChapterId" 
                      placeholder="请选择章节" 
                      style="width: 300px"
                      @change="loadChapterAudioInfo"
                      clearable
                      :disabled="!mixNovelId"
                    >
                      <el-option 
                        v-for="chapter in mixChapters" 
                        :key="chapter.id" 
                        :label="chapter.title + (chapter.audio_status === 'completed' ? ' (可混音)' : ' (未完成)')" 
                        :value="chapter.id"
                        :disabled="chapter.audio_status !== 'completed'"
                      />
                    </el-select>
                  </el-form-item>
                  <el-form-item label="选择环境音">
                    <el-select 
                      v-model="mixAmbientId" 
                      placeholder="请选择环境音" 
                      style="width: 300px"
                      clearable
                    >
                      <el-option 
                        v-for="sound in ambientSounds" 
                        :key="sound.id" 
                        :label="sound.name" 
                        :value="sound.id"
                      />
                    </el-select>
                    <el-button 
                      size="small" 
                      @click="loadAmbientSounds" 
                      style="margin-left: 10px"
                    >
                      刷新列表
                    </el-button>
                  </el-form-item>
                  <el-form-item label="环境音音量">
                    <el-slider v-model="mixConfig.volume" :min="0" :max="1" :step="0.05" style="width: 300px" />
                    <span style="margin-left: 15px">{{ Math.round(mixConfig.volume * 100) }}%</span>
                  </el-form-item>
                  <el-form-item label="淡入时长(秒)">
                    <el-slider v-model="mixConfig.fade_in" :min="0" :max="10" :step="0.5" style="width: 300px" />
                    <span style="margin-left: 15px">{{ mixConfig.fade_in }} 秒</span>
                  </el-form-item>
                  <el-form-item label="淡出时长(秒)">
                    <el-slider v-model="mixConfig.fade_out" :min="0" :max="10" :step="0.5" style="width: 300px" />
                    <span style="margin-left: 15px">{{ mixConfig.fade_out }} 秒</span>
                  </el-form-item>
                  <el-form-item>
                    <el-button 
                      type="primary" 
                      :loading="mixingAudio" 
                      :disabled="!canMix"
                      @click="mixAudioWithAmbient"
                      size="large"
                    >
                      <el-icon><VideoPlay /></el-icon>
                      开始混音
                    </el-button>
                  </el-form-item>
                </el-form>
              </el-col>
              
              <el-col :md="12" :sm="24">
                <!-- 混音状态 -->
                <el-descriptions :column="1" border style="margin-bottom: 20px">
                  <el-descriptions-item label="人声状态">
                    <el-tag :type="chapterAudioStatus === 'completed' ? 'success' : 'warning'">
                      {{ chapterAudioStatus === 'completed' ? '可用于混音' : '请先生成并合并人声音频' }}
                    </el-tag>
                  </el-descriptions-item>
                  <el-descriptions-item label="已选环境音">
                    <span v-if="selectedMixAmbient">{{ selectedMixAmbient.name }}</span>
                    <span v-else style="color: #909399">请先选择环境音</span>
                  </el-descriptions-item>
                  <el-descriptions-item label="预计输出">
                    <span v-if="chapterAudioStatus === 'completed' && selectedMixAmbient">
                      将生成包含人声和环境音的完整音频文件
                    </span>
                    <span v-else style="color: #909399">请完成以上配置</span>
                  </el-descriptions-item>
                </el-descriptions>

                <!-- 混音结果 -->
                <div v-if="mixResult" class="mix-result">
                  <el-alert title="混音成功！" type="success" style="margin-bottom: 15px" />
                  <div class="result-player">
                    <h4 style="margin-bottom: 15px">混音结果预览</h4>
                    <audio :src="getAmbientAudioUrl(mixResult)" controls style="width: 100%" />
                    <p style="margin-top: 15px; color: #606266; font-size: 13px">
                      <el-icon style="vertical-align: middle; margin-right: 5px"><Location /></el-icon>
                      混合音频文件: {{ mixResult }}
                    </p>
                  </div>
                </div>
              </el-col>
            </el-row>
          </div>
        </el-card>
      </el-tab-pane>
    </el-tabs>

    <!-- 音频播放器对话框 -->
    <el-dialog v-model="playerVisible" title="音频播放" width="500px">
      <div class="audio-player">
        <div class="player-text">{{ playingAmbientSound?.name || '音频播放' }}</div>
        <audio ref="audioRef" :src="playingAudioUrl" controls autoplay style="width: 100%; margin-top: 20px" />
      </div>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { convertFileSrc } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { 
  Refresh, VideoPlay, Headset, Delete, Bell, Location, Clock
} from '@element-plus/icons-vue'

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
  tags?: string
}

interface GenerateAmbientResponse {
  success: boolean
  file_path?: string
  error?: string
  duration?: number
  ambient_id?: number
}

const novels = ref<Novel[]>([])
const loading = ref(false)

// 标签页相关
const activeTab = ref('generate')

// 环境音生成相关
const ambientForm = reactive({
  prompt: '',
  duration: 10,
  name: ''
})

const generatingAmbient = ref(false)
const ambientResult = ref<GenerateAmbientResponse | null>(null)

const quickScenes = ['rainy cafe', 'forest morning birds', 'ocean waves', 'fireplace crackling', 'quiet library', 'thunderstorm', 'city street', 'countryside wind']

// 环境音库相关
const ambientSounds = ref<AmbientSound[]>([])
const ambientLibraryLoading = ref(false)
const ambientFilter = reactive({
  category: '',
  search: ''
})

const filteredAmbientSounds = computed(() => {
  return ambientSounds.value.filter(sound => {
    const matchCategory = !ambientFilter.category || sound.category === ambientFilter.category
    const matchSearch = !ambientFilter.search || 
      sound.name.toLowerCase().includes(ambientFilter.search.toLowerCase()) ||
      (sound.description && sound.description.toLowerCase().includes(ambientFilter.search.toLowerCase()))
    return matchCategory && matchSearch
  })
})

// 播放器相关
const playerVisible = ref(false)
const playingAmbientSound = ref<AmbientSound | null>(null)
const playingAudioUrl = ref('')
const audioRef = ref<HTMLAudioElement | null>(null)

// 混音相关
const mixNovelId = ref<number | null>(null)
const mixChapterId = ref<number | null>(null)
const mixAmbientId = ref<number | null>(null)
const mixChapters = ref<Chapter[]>([])
const chapterAudioStatus = ref<string>('')

const mixConfig = reactive({
  volume: 0.3,
  fade_in: 2.0,
  fade_out: 2.0
})

const mixingAudio = ref(false)
const mixResult = ref<string | null>(null)

const selectedMixAmbient = computed(() => {
  if (!mixAmbientId.value) return null
  return ambientSounds.value.find(s => s.id === mixAmbientId.value) || null
})

const canMix = computed(() => {
  return chapterAudioStatus.value === 'completed' && selectedMixAmbient.value !== null
})

// 选择快捷场景
const selectQuickScene = (scene: string) => {
  ambientForm.prompt = scene
}

// 生成环境音
const generateAmbientSound = async () => {
  if (!ambientForm.prompt.trim()) {
    ElMessage.warning('请输入场景描述')
    return
  }
  
  generatingAmbient.value = true
  ambientResult.value = null
  
  try {
    ElMessage.info('正在使用 Stable Audio Open 生成环境音...')
    const result = await invoke<GenerateAmbientResponse>('generate_ambient_sound', {
      prompt: ambientForm.prompt,
      duration: ambientForm.duration,
      name: ambientForm.name || undefined
    })
    
    ambientResult.value = result
    
    if (result.success) {
      ElMessage.success('环境音生成成功！')
      await loadAmbientSounds()
    } else {
      ElMessage.error(result.error || '生成失败')
    }
  } catch (e) {
    ElMessage.error('生成环境音失败: ' + e)
  } finally {
    generatingAmbient.value = false
  }
}

// 取消生成
const cancelGeneration = () => {
  ElMessage.info('生成任务已取消')
  generatingAmbient.value = false
}

// 格式化时长
const formatDuration = (seconds: number) => {
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

// 加载小说列表
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

// 加载章节列表（用于混音）
const loadMixChapters = async () => {
  if (!mixNovelId.value) return
  try {
    mixChapters.value = await invoke('get_chapters', { 
      novel_id: mixNovelId.value, 
      novelId: mixNovelId.value 
    })
  } catch (e) {
    ElMessage.error('加载章节列表失败')
  }
}

// 加载章节音频信息
const loadChapterAudioInfo = async () => {
  if (!mixChapterId.value) return
  try {
    const audioDetail = await invoke('get_chapter_audio_detail', { 
      chapter_id: mixChapterId.value, 
      chapterId: mixChapterId.value 
    })
    chapterAudioStatus.value = (audioDetail as any)?.status || 'not_created'
  } catch (e) {
    chapterAudioStatus.value = 'not_created'
  }
}

// 加载环境音列表
const loadAmbientSounds = async () => {
  ambientLibraryLoading.value = true
  try {
    ambientSounds.value = await invoke<AmbientSound[]>('get_ambient_sounds')
  } catch (e) {
    console.error('加载环境音列表失败:', e)
    ElMessage.error('加载环境音列表失败')
  } finally {
    ambientLibraryLoading.value = false
  }
}

// 获取音频播放 URL
const getAmbientAudioUrl = (path?: string) => {
  if (!path) return ''
  return convertFileSrc(path)
}

// 播放环境音
const playAmbientSound = async (sound: AmbientSound) => {
  playingAmbientSound.value = sound
  
  try {
    const audioData = await invoke<number[]>('get_audio_stream', { path: sound.file_path })
    const byteArray = new Uint8Array(audioData)
    const blob = new Blob([byteArray], { type: 'audio/wav' })
    playingAudioUrl.value = URL.createObjectURL(blob)
    playerVisible.value = true
  } catch (error) {
    ElMessage.error(`播放音频失败: ${error}`)
  }
}

// 删除环境音（只删除记录，保留文件）
const deleteAmbientSound = async (sound: AmbientSound) => {
  try {
    await ElMessageBox.confirm(
      `确定要从列表中移除 "${sound.name}" 吗？\n（音频文件将保留在磁盘上）`,
      '确认移除',
      {
        confirmButtonText: '移除',
        cancelButtonText: '取消',
        type: 'warning',
      }
    )
    
    await invoke('delete_ambient_sound', { ambient_id: sound.id , ambientId: sound.id})
    await loadAmbientSounds()
    
    ElMessage.success('删除成功')
  } catch {
    // 用户取消
  }
}

// 混音函数
const mixAudioWithAmbient = async () => {
  if (!mixChapterId.value || !mixAmbientId.value) {
    ElMessage.warning('请确保已选择章节和环境音')
    return
  }
  
  mixingAudio.value = true
  mixResult.value = null
  
  try {
    const resultPath = await invoke<string>('mix_voice_with_ambient', {
      chapter_id: mixChapterId.value,
      ambient_sound_id: mixAmbientId.value,
      volume: mixConfig.volume,
      fade_in: mixConfig.fade_in,
      fade_out: mixConfig.fade_out
    })
    
    mixResult.value = resultPath
    ElMessage.success('混音成功！')
  } catch (e) {
    ElMessage.error(`混音失败: ${e}`)
  } finally {
    mixingAudio.value = false
  }
}

onMounted(() => {
  loadNovels()
  loadAmbientSounds()
})
</script>

<style scoped>
.ambient-page {
  padding: 20px;
}

.page-header {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.ambient-generate-form {
  padding: 10px 0;
}

.scene-tag {
  margin-right: 10px;
  margin-bottom: 10px;
  cursor: pointer;
  transition: all 0.2s;
}

.scene-tag:hover {
  transform: scale(1.05);
}

.ambient-result {
  padding: 20px;
  background-color: #f5f7fa;
  border-radius: 8px;
}

.generate-tips {
  padding: 20px;
  background-color: #ecf5ff;
  border-radius: 8px;
}

.result-player {
  margin-top: 15px;
}

.ambient-filters {
  display: flex;
  gap: 15px;
  margin-bottom: 20px;
  align-items: center;
}

.ambient-library-container {
  min-height: 400px;
}

.ambient-sound-card {
  margin-bottom: 15px;
  transition: all 0.3s;
}

.ambient-sound-card:hover {
  transform: translateY(-2px);
}

.sound-icon {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 60px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border-radius: 4px 4px 0 0;
  margin: -15px -15px 15px -15px;
}

.sound-info {
  text-align: center;
}

.sound-name {
  font-weight: bold;
  font-size: 14px;
  margin-bottom: 8px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sound-meta {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
}

.sound-duration {
  font-size: 12px;
  color: #909399;
}

.sound-desc {
  font-size: 12px;
  color: #606266;
  margin-bottom: 10px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.sound-actions {
  display: flex;
  justify-content: center;
  gap: 8px;
  margin-top: 10px;
}

.mix-form {
  padding: 10px 0;
}

.mix-result {
  margin-top: 20px;
  padding: 20px;
  background-color: #f0f9eb;
  border-radius: 8px;
}

.audio-player {
  text-align: center;
}

.player-text {
  padding: 20px;
  background-color: #f5f7fa;
  border-radius: 4px;
  line-height: 1.6;
  margin-bottom: 20px;
}
</style>
