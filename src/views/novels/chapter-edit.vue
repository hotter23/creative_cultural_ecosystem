<template>
  <div class="chapter-edit">
    <!-- 工具栏 -->
    <el-card class="toolbar-card" shadow="never">
      <div class="toolbar">
        <div class="toolbar-left">
          <el-button @click="handleBack">
            <el-icon><ArrowLeft /></el-icon>
            返回
          </el-button>
          <el-divider direction="vertical" />
          <span class="chapter-info">
            编辑章节：{{ chapterTitle }}
          </span>
        </div>
        <div class="toolbar-right">
          <el-button :loading="saving" @click="handleSave">保存草稿</el-button>
          <el-button type="primary" :loading="saving" @click="handleSaveAndComplete">保存并完成</el-button>
        </div>
      </div>
    </el-card>

    <!-- 编辑器区域 -->
    <el-row :gutter="20" style="margin-top: 20px;">
      <el-col :span="16">
        <el-card class="editor-card" shadow="never">
          <el-form :model="form" label-width="80px">
            <el-form-item label="章节标题">
              <el-input
                v-model="form.title"
                placeholder="请输入章节标题"
                maxlength="100"
                show-word-limit
              />
            </el-form-item>
            <el-form-item label="章节序号">
              <el-input-number
                v-model="form.order_num"
                :min="1"
                :max="9999"
              />
            </el-form-item>
            <el-form-item label="章节内容">
              <!-- 模式切换 -->
              <div style="margin-bottom: 15px;">
                <el-radio-group v-model="editMode" size="small">
                  <el-radio-button value="edit">编辑模式</el-radio-button>
                  <el-radio-button value="markup">标注模式</el-radio-button>
                </el-radio-group>
                <!-- 标注引导提示 -->
                <el-alert
                  v-if="editMode === 'markup'"
                  style="margin-top: 15px;"
                  title="标注指南"
                  description="1️⃣ 点击「导入内容并手动拆分」开始 → 2️⃣ 手动选择拆分位置 → 3️⃣ 为每段选择类型 → 4️⃣ 对话需绑定角色"
                  type="info"
                  :closable="false"
                  show-icon
                />
                
                <!-- 类型图例 -->
                <div v-if="editMode === 'markup'" class="type-legend">
                  <span class="legend-item">
                    <span class="legend-color narration-color"></span>
                    <span>旁白 - 叙述情节、场景描写</span>
                  </span>
                  <span class="legend-item">
                    <span class="legend-color dialogue-color"></span>
                    <span>对话 - 角色说的话，需绑定角色</span>
                  </span>
                  <span class="legend-item">
                    <span class="legend-color environment-color"></span>
                    <span>环境音 - 背景音效、环境声</span>
                  </span>
                </div>
              </div>
              
              <!-- 编辑模式 -->
              <div class="editor-container" v-if="editMode === 'edit'">
                <Toolbar
                  :editor="editorRef"
                  :defaultConfig="toolbarConfig"
                  :mode="mode"
                  style="border-bottom: 1px solid #dcdfe6;"
                />
                <Editor
                  :defaultConfig="editorConfig"
                  :mode="mode"
                  v-model="form.content"
                  @onCreated="handleCreated"
                  style="height: 500px;"
                />
              </div>
              
              <!-- 标注模式 -->
              <div class="markup-container" v-else>
                <!-- 操作工具栏 -->
                <div class="markup-toolbar">
                  <el-button size="small" @click="importAndSplit">
                    <el-icon><DocumentCopy /></el-icon>
                    导入内容并手动拆分
                  </el-button>
                  <el-button 
                    size="small" 
                    type="danger" 
                    v-if="paragraphs.length > 0" 
                    @click="clearAll"
                  >
                    <el-icon><Delete /></el-icon>
                    清空所有
                  </el-button>
                </div>
                
                <!-- 段落列表 -->
                <div v-if="paragraphs.length > 0" class="paragraph-list">
                  <div 
                    v-for="(para, index) in paragraphs" 
                    :key="index" 
                    class="paragraph-item"
                    :class="{ 'is-dialogue': para.type === 'dialogue', 'is-narration': para.type === 'narration', 'is-environment': para.type === 'environment' }"
                  >
                    <div class="para-header">
                      <span class="para-index">第 {{ index + 1 }} 段</span>
                      <el-select 
                        v-model="para.type" 
                        size="small" 
                        style="width: 120px;"
                        @change="updateParagraph(index)"
                      >
                        <el-option label="旁白" value="narration" />
                        <el-option label="角色对话" value="dialogue" />
                        <el-option label="环境音" value="environment" />
                      </el-select>
                      <el-select 
                        v-if="para.type === 'dialogue'"
                        v-model="para.characterId"
                        size="small"
                        style="width: 150px; margin-left: 10px;"
                        filterable
                        @change="updateParagraph(index)"
                      >
                        <el-option label="选择角色" value="" />
                        <el-option 
                          v-for="char in novelCharacters" 
                          :key="char.id" 
                          :label="char.name" 
                          :value="char.id" 
                        />
                      </el-select>
                      <el-tag 
                        v-if="para.type === 'dialogue' && para.character" 
                        size="small" 
                        type="success"
                        style="margin-left: 10px;"
                      >
                        {{ para.character.name }} - {{ getVoiceName(para.character.voice_id) }}
                      </el-tag>
                      <el-tag 
                        v-else-if="para.type === 'narration'" 
                        size="small" 
                        type="info"
                        style="margin-left: 10px;"
                      >
                        旁白
                      </el-tag>
                      <el-tag 
                        v-else-if="para.type === 'environment'" 
                        size="small" 
                        type="warning"
                        style="margin-left: 10px;"
                      >
                        环境音
                      </el-tag>
                      <!-- 段落操作按钮 -->
                      <div class="para-actions" style="margin-left: auto;">
                        <el-button 
                          size="small" 
                          @click="editParagraph(index)"
                          style="margin-right: 5px;"
                        >
                          <el-icon><EditPen /></el-icon>
                        </el-button>
                        <el-button 
                          size="small" 
                          type="danger" 
                          @click="deleteParagraph(index)"
                        >
                          <el-icon><Delete /></el-icon>
                        </el-button>
                      </div>
                    </div>
                    <div class="para-content">
                      <el-input 
                        v-if="editingIndex === index"
                        v-model="para.content" 
                        type="textarea" 
                        :rows="3" 
                        @blur="editingIndex = -1"
                        @keyup.enter="editingIndex = -1"
                      />
                      <span v-else>{{ para.content }}</span>
                    </div>
                  </div>
                  
                  <!-- 添加新段落 -->
                  <div style="text-align: center; margin-top: 20px;">
                    <el-button 
                      type="primary" 
                      size="small" 
                      @click="addParagraph"
                      icon="Plus"
                    >
                      添加新段落
                    </el-button>
                  </div>
                </div>
                
                <!-- 空状态 -->
                <div v-else class="empty-paragraphs">
                  <el-empty description="点击上方按钮「导入内容并手动拆分」开始" :image-size="80" />
                </div>
              </div>
            </el-form-item>
          </el-form>
        </el-card>
      </el-col>
      
      <el-col :span="8">
        <!-- 字数统计 -->
        <el-card class="stats-card" shadow="never">
          <template #header>
            <span>字数统计</span>
          </template>
          <div class="stats-content">
            <div class="stat-item">
              <span class="stat-label">当前字数</span>
              <span class="stat-value">{{ wordCount }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">章节字数</span>
              <span class="stat-value">{{ chapterWordCount }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">小说总字数</span>
              <span class="stat-value">{{ totalWordCount }}</span>
            </div>
          </div>
        </el-card>

        <!-- 标注统计（标注模式下显示） -->
        <el-card class="markup-stats-card" shadow="never" style="margin-top: 20px;" v-if="editMode === 'markup'">
          <template #header>
            <span>标注统计</span>
          </template>
          <div class="stats-content">
            <div class="stat-item">
              <span class="stat-label">总段落数</span>
              <span class="stat-value">{{ paragraphs.length }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">已标注</span>
              <span class="stat-value">{{ markedCount }} / {{ paragraphs.length }}</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">旁白</span>
              <span class="stat-value" style="color: #909399;">{{ narrationCount }} 段</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">对话</span>
              <span class="stat-value" style="color: #67c23a;">{{ dialogueCount }} 段</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">环境音</span>
              <span class="stat-value" style="color: #e6a23c;">{{ environmentCount }} 段</span>
            </div>
            <el-progress :percentage="markupProgress" :stroke-width="8" style="margin-top: 10px;" />
          </div>
        </el-card>

        <!-- AI 创作辅助 -->
        

        <!-- 快捷操作 -->
        
      </el-col>
    </el-row>

    <!-- AI 辅助功能面板 -->
    <el-drawer
      v-model="aiPanelVisible"
      title="AI 创作助手"
      direction="rtl"
      size="500px"
      :close-on-click-modal="false"
    >
      <div class="ai-panel-content">
        <!-- 操作类型选择 -->
        <el-radio-group v-model="currentAIAction" style="margin-bottom: 20px;" @change="onAIActionChange">
          <el-radio-button value="continue">续写</el-radio-button>
          <el-radio-button value="polish">润色</el-radio-button>
          <el-radio-button value="summarize">摘要</el-radio-button>
          <el-radio-button value="suggest">建议</el-radio-button>
          <el-radio-button value="custom">自定义</el-radio-button>
        </el-radio-group>

        <!-- 生成参数 -->
        <el-form v-if="currentAIAction === 'continue' || currentAIAction === 'custom'">
          <el-form-item label="生成长度">
            <el-select v-model="aiForm.length" style="width: 100%;">
              <el-option label="短篇 (约500字)" :value="500" />
              <el-option label="中篇 (约1000字)" :value="1000" />
              <el-option label="长篇 (约2000字)" :value="2000" />
            </el-select>
          </el-form-item>
        </el-form>

        <el-form v-if="currentAIAction === 'polish'">
          <el-form-item label="润色风格">
            <el-select v-model="aiForm.style" style="width: 100%;">
              <el-option label="更有张力" value="更有张力，增强戏剧性" />
              <el-option label="更加细腻" value="增加细节描写，让内容更细腻" />
              <el-option label="简洁明快" value="精简文字，让表达更简洁" />
              <el-option label="幽默风趣" value="增加幽默感" />
              <el-option label="文艺抒情" value="增强文艺气息和抒情色彩" />
            </el-select>
          </el-form-item>
        </el-form>

        <!-- 自定义指令 -->
        <el-form v-if="currentAIAction === 'custom'">
          <el-form-item label="自定义指令">
            <el-input
              v-model="aiForm.customPrompt"
              type="textarea"
              :rows="3"
              placeholder="请描述你想要 AI 做什么..."
            />
          </el-form-item>
        </el-form>

        <!-- 生成按钮 -->
        <el-button 
          type="primary" 
          style="width: 100%; margin-bottom: 20px;"
          :loading="aiLoading"
          @click="executeAIAction"
        >
          {{ aiLoading ? 'AI 正在思考...' : '开始生成' }}
        </el-button>

        <!-- AI 结果展示 -->
        <div v-if="aiResult" class="ai-result">
          <el-divider content-position="left">AI 生成结果</el-divider>
          
          <div class="ai-result-content">
            <pre>{{ aiResult }}</pre>
          </div>

          <!-- 操作按钮 -->
          <div class="ai-result-actions" style="margin-top: 15px;">
            <el-button size="small" @click="copyResult">
              <el-icon><DocumentCopy /></el-icon>
              复制
            </el-button>
            <el-button size="small" type="primary" @click="appendResult">
              <el-icon><Plus /></el-icon>
              追加到末尾
            </el-button>
            <el-button size="small" type="success" @click="replaceSelection">
              <el-icon><Refresh /></el-icon>
              替换选中内容
            </el-button>
          </div>
        </div>

        <!-- 历史记录 -->
        <div v-if="aiHistory.length > 0" class="ai-history" style="margin-top: 20px;">
          <el-divider content-position="left">历史记录</el-divider>
          <div 
            v-for="(item, index) in aiHistory.slice(-5).reverse()" 
            :key="index"
            class="history-item"
            style="padding: 10px; background: #f5f7fa; margin-bottom: 10px; border-radius: 4px; cursor: pointer;"
            @click="useHistoryResult(item)"
          >
            <div style="font-size: 12px; color: #909399; margin-bottom: 5px;">
              {{ item.action }} - {{ item.time }}
            </div>
            <div style="font-size: 13px; overflow: hidden; text-overflow: ellipsis; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical;">
              {{ item.content }}
            </div>
          </div>
        </div>
      </div>
    </el-drawer>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, shallowRef, watch, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useNovelStore } from '../../stores/novel'
import { Editor, Toolbar } from '@wangeditor/editor-for-vue'
import type { IDomEditor } from '@wangeditor/editor'
import { invoke } from '@tauri-apps/api/core'
import {
  ArrowLeft,
  MagicStick,
  EditPen,
  DocumentChecked,
  Delete,
  Document,
  Warning,
  ChatDotRound,
  DocumentCopy,
  Plus,
  Refresh
} from '@element-plus/icons-vue'

const router = useRouter()
const route = useRoute()
const novelStore = useNovelStore()

const saving = ref(false)
const aiPanelVisible = ref(false)
const aiLoading = ref(false)
const aiEnabled = ref(false)

const novelId = computed(() => parseInt(route.params.novelId as string))
const chapterId = computed(() => parseInt(route.params.id as string))

const chapterTitle = ref('')
const chapterWordCount = ref(0)
const totalWordCount = ref(0)

// 编辑器实例
const editorRef = shallowRef<IDomEditor | null>(null)
const mode = ref<'default' | 'simple'>('default')

const form = reactive({
  title: '',
  order_num: 1,
  content: ''
})

// AI 表单
const aiForm = reactive({
  prompt: '',
  length: 1000,
  style: '更有张力，增强戏剧性',
  customPrompt: ''
})

// AI 状态
const currentAIAction = ref('continue')
const aiResult = ref('')
const aiHistory = ref<Array<{
  action: string
  content: string
  time: string
}>>([])

const wordCount = computed(() => {
  if (!editorRef.value) return 0
  return editorRef.value.getText().replace(/\s/g, '').length
})

// 标注模式
const editMode = ref<'edit' | 'markup'>('edit')

// 段落类型定义
interface Paragraph {
  content: string
  type: 'narration' | 'dialogue' | 'environment'
  characterId?: number | null
  character?: any
}

// 段落列表
const paragraphs = ref<Paragraph[]>([])

// 角色列表
const novelCharacters = ref<any[]>([])

// 按分类的音色列表（用于显示角色绑定的音色）
const voiceCategories = [
  {
    label: '中文（普通话）',
    options: [
      { id: 'male-qn-qingse', name: '青涩青年' },
      { id: 'male-qn-jingying', name: '精英青年' },
      { id: 'male-qn-badao', name: '霸道青年' },
      { id: 'male-qn-daxuesheng', name: '青年大学生' },
      { id: 'female-shaonv', name: '少女' },
      { id: 'female-yujie', name: '御姐' },
      { id: 'female-chengshu', name: '成熟女性' },
      { id: 'female-tianmei', name: '甜美女性' },
      { id: 'male-qn-qingse-jingpin', name: '青涩青年(精)' },
      { id: 'male-qn-jingying-jingpin', name: '精英青年(精)' },
      { id: 'male-qn-badao-jingpin', name: '霸道青年(精)' },
      { id: 'male-qn-daxuesheng-jingpin', name: '大学生(精)' },
      { id: 'female-shaonv-jingpin', name: '少女(精)' },
      { id: 'female-yujie-jingpin', name: '御姐(精)' },
      { id: 'female-chengshu-jingpin', name: '成熟女性(精)' },
      { id: 'female-tianmei-jingpin', name: '甜美女性(精)' },
      { id: 'clever_boy', name: '聪明男童' },
      { id: 'cute_boy', name: '可爱男童' },
      { id: 'lovely_girl', name: '萌萌女童' },
      { id: 'bingjiao_didi', name: '病娇弟弟' },
      { id: 'junlang_nanyou', name: '俊朗男友' },
      { id: 'chunzhen_xuedi', name: '纯真学弟' },
      { id: 'lengdan_xiongzhang', name: '冷淡学长' },
      { id: 'badao_shaoye', name: '霸道少爷' },
      { id: 'tianxin_xiaoling', name: '甜心小玲' },
      { id: 'qiaopi_mengmei', name: '俏皮萌妹' },
      { id: 'wumei_yujie', name: '妩媚御姐' },
      { id: 'diadia_xuemei', name: '嗲嗲学妹' },
      { id: 'danya_xuejie', name: '淡雅学姐' },
      { id: 'Chinese (Mandarin)_Reliable_Executive', name: '沉稳高管' },
      { id: 'Chinese (Mandarin)_News_Anchor', name: '新闻女声' },
      { id: 'Chinese (Mandarin)_Mature_Woman', name: '傲娇御姐' },
      { id: 'Chinese (Mandarin)_Unrestrained_Young_Man', name: '不羁青年' },
      { id: 'Chinese (Mandarin)_Southern_Young_Man', name: '南方小哥' },
      { id: 'Chinese (Mandarin)_Gentle_Youth', name: '温润青年' },
    ]
  },
  {
    label: '中文（粤语）',
    options: [
      { id: 'Cantonese_ProfessionalHost（F)', name: '专业女主持' },
      { id: 'Cantonese_GentleLady', name: '温柔女声' },
      { id: 'Cantonese_ProfessionalHost（M)', name: '专业男主持' },
      { id: 'Cantonese_PlayfulMan', name: '活泼男声' },
      { id: 'Cantonese_CuteGirl', name: '可爱女孩' },
      { id: 'Cantonese_KindWoman', name: '善良女声' },
    ]
  }
]

// 快速查找音色名称
const getVoiceName = (voiceId?: string) => {
  if (!voiceId) return '未绑定音色'
  for (const cat of voiceCategories) {
    const found = cat.options.find(opt => opt.id === voiceId)
    if (found) return found.name
  }
  return voiceId
}

// 加载角色列表
const loadCharacters = async () => {
  try {
    console.log('loadCharacters: novelId.value =', novelId.value)
    console.log('loadCharacters: typeof novelId.value =', typeof novelId.value)
    const params = { 
      novel_id: novelId.value,
      novelId: novelId.value 
    }
    console.log('loadCharacters: params =', params)
    const chars = await invoke('get_characters', params)
    console.log('loadCharacters: chars =', chars)
    novelCharacters.value = chars as any[]
  } catch (error) {
    console.error('加载角色列表失败:', error)
  }
}

// 编辑段落索引
const editingIndex = ref(-1)

// 导入内容并手动拆分
const importAndSplit = () => {
  if (!form.content) {
    ElMessage.warning('请先在编辑模式下输入内容')
    return
  }
  
  // 使用 HTML 解析获取纯文本
  const tempDiv = document.createElement('div')
  tempDiv.innerHTML = form.content
  const text = tempDiv.textContent || tempDiv.innerText || ''
  
  // 简单清理换行
  const processedText = text
    .replace(/\r\n/g, '\n')
    .replace(/\r/g, '\n')
    .replace(/\n\s*\n/g, '\n')
    .trim()
  
  // 将整个内容作为一个段落导入
  paragraphs.value = [{
    content: processedText,
    type: 'narration' as Paragraph['type'],
    characterId: null,
    character: undefined
  }]
  
  ElMessage.success(`内容已导入，当前 1 个段落。请手动编辑调整。`)
}

// 清空所有段落
const clearAll = () => {
  ElMessageBox.confirm('确定要清空所有段落吗？这将删除所有标注信息。', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    paragraphs.value = []
    ElMessage.success('已清空')
  }).catch(() => {})
}

// 编辑段落
const editParagraph = (index: number) => {
  editingIndex.value = index
  nextTick(() => {
    const input = document.querySelector('.paragraph-item:nth-child(' + (index + 1) + ') textarea') as HTMLTextAreaElement | null
    if (input) {
      input.focus()
    }
  })
}

// 删除段落
const deleteParagraph = (index: number) => {
  ElMessageBox.confirm('确定要删除该段落吗？', '警告', {
    confirmButtonText: '确定',
    cancelButtonText: '取消',
    type: 'warning'
  }).then(() => {
    paragraphs.value.splice(index, 1)
    ElMessage.success('删除成功')
  }).catch(() => {})
}

// 添加新段落
const addParagraph = () => {
  paragraphs.value.push({
    content: '',
    type: 'narration' as Paragraph['type'],
    characterId: null,
    character: undefined
  })
  // 自动进入编辑状态
  editingIndex.value = paragraphs.value.length - 1
  nextTick(() => {
    const input = document.querySelector('.paragraph-item:last-child textarea')
    if (input) {
      input.focus()
    }
  })
}

// 更新段落类型或角色
const updateParagraph = (index: number) => {
  const para = paragraphs.value[index]
  if (para.type === 'dialogue' && para.characterId && para.characterId !== '') {
    // 查找角色信息
    para.character = novelCharacters.value.find(c => c.id === para.characterId)
  } else {
    para.character = undefined
    if (para.type !== 'dialogue') {
      para.characterId = null
    }
  }
}

// 保存章节段落标注
const saveChapterParagraphs = async () => {
  if (!chapterId.value || paragraphs.value.length === 0) {
    return
  }
  
  try {
    await invoke('save_chapter_paragraphs', {
      chapter_id: chapterId.value,
      chapterId: chapterId.value,
      paragraphs: paragraphs.value.map((para, index) => ({
        paragraph_index: index,
        content: para.content,
        type: para.type,
        character_id: para.characterId
      }))
    })
  } catch (error) {
    console.error('保存段落标注失败:', error)
  }
}

// 加载章节段落标注
const loadChapterParagraphs = async () => {
  if (!chapterId.value) {
    return
  }
  
  try {
    const paragraphsData = await invoke('get_chapter_paragraphs', {
      chapter_id: chapterId.value,
      chapterId: chapterId.value
    }) as any[]
    
    if (paragraphsData && paragraphsData.length > 0) {
      paragraphs.value = paragraphsData.map(para => ({
        content: para.content,
        type: para.type as any,
        characterId: para.character_id,
        character: para.character_id ? novelCharacters.value.find(c => c.id === para.character_id) : undefined
      }))
    }
  } catch (error) {
    console.error('加载段落标注失败:', error)
  }
}

// 监听模式切换（保持旧的引用，但不自动触发）
watch(editMode, (newMode) => {
  if (newMode === 'markup') {
    // 标注模式不自动拆分，等待用户手动操作
  }
}, { flush: 'post' })

// 标注统计计算
const markedCount = computed(() => {
  return paragraphs.value.filter(p => p.type !== 'narration' || p.characterId).length
})

const narrationCount = computed(() => {
  return paragraphs.value.filter(p => p.type === 'narration').length
})

const dialogueCount = computed(() => {
  return paragraphs.value.filter(p => p.type === 'dialogue').length
})

const environmentCount = computed(() => {
  return paragraphs.value.filter(p => p.type === 'environment').length
})

const markupProgress = computed(() => {
  if (paragraphs.value.length === 0) return 0
  // 对话需要绑定角色才算完成标注
  const completed = paragraphs.value.filter(p => {
    if (p.type === 'dialogue') {
      return p.characterId !== null && p.characterId !== undefined
    }
    return true // 旁白和环境音只要选择了类型就算完成
  }).length
  return Math.round((completed / paragraphs.value.length) * 100)
})

// 工具栏配置
const toolbarConfig = {
  excludeKeys: []
}

// 编辑器配置
const editorConfig = {
  placeholder: '请输入章节内容...',
  MENU_CONF: {}
}

const handleCreated = (editor: IDomEditor) => {
  editorRef.value = editor
}

const handleBack = () => {
  if (form.title || form.content) {
    ElMessageBox.confirm(
      '您有未保存的内容，确定要离开吗？',
      '提示',
      {
        confirmButtonText: '确定',
        cancelButtonText: '取消',
        type: 'warning'
      }
    ).then(() => {
      router.push(`/novels/${novelId.value}/chapters`)
    }).catch(() => {})
  } else {
    router.push(`/novels/${novelId.value}/chapters`)
  }
}

const handleSave = async () => {
  if (!form.title) {
    ElMessage.warning('请输入章节标题')
    return
  }
  
  saving.value = true
  try {
    if (chapterId.value) {
      await novelStore.updateChapter(chapterId.value, {
        title: form.title,
        content: form.content,
        order_num: form.order_num
      })
      
      // 保存段落标注
      await saveChapterParagraphs()
    }
    
    ElMessage.success('保存成功')
  } catch (error) {
    ElMessage.error('保存失败，请重试')
  } finally {
    saving.value = false
  }
}

const handleSaveAndComplete = async () => {
  if (!form.title) {
    ElMessage.warning('请输入章节标题')
    return
  }
  
  saving.value = true
  try {
    if (chapterId.value) {
      await novelStore.updateChapter(chapterId.value, {
        title: form.title,
        content: form.content,
        order_num: form.order_num,
        status: 'completed'
      })
      
      // 保存段落标注
      await saveChapterParagraphs()
    }
    
    ElMessage.success('章节已完成！')
    router.push(`/novels/${novelId.value}/chapters`)
  } catch (error) {
    ElMessage.error('保存失败，请重试')
  } finally {
    saving.value = false
  }
}



// 检查 AI 配置
const checkAIConfig = async () => {
  try {
    const config = await invoke<Record<string, string>>('get_minimax_config')
    aiEnabled.value = config.minimax_enabled === 'true' && !!config.minimax_api_key
  } catch {
    aiEnabled.value = false
  }
}

// 跳转到设置页面
const goToSettings = () => {
  router.push('/settings')
}

// 打开 AI 面板
const openAIPanel = (action: string) => {
  currentAIAction.value = action
  aiResult.value = ''
  aiPanelVisible.value = true
}

// AI 操作类型变更
const onAIActionChange = () => {
  aiResult.value = ''
}

// 执行 AI 操作
const executeAIAction = async () => {
  if (!aiEnabled.value) {
    ElMessage.warning('请先配置并启用 AI 服务')
    return
  }

  aiLoading.value = true
  aiResult.value = ''

  try {
    let result = ''
    const currentContent = form.content || ''

    switch (currentAIAction.value) {
      case 'continue':
        // 续写
        result = await invoke<string>('ai_continue_novel_content', {
          prefixContent: currentContent,
          genre: '通用',
          wordCount: aiForm.length
        })
        break

      case 'polish':
        // 润色
        result = await invoke<string>('ai_polish_content', {
          content: currentContent,
          style: aiForm.style
        })
        break

      case 'summarize':
        // 摘要
        result = await invoke<string>('ai_summarize_content', {
          content: currentContent
        })
        break

      case 'suggest':
        // 情节建议
        result = await invoke<string>('ai_suggest_plot', {
          content: currentContent
        })
        break

      case 'custom':
        // 自定义
        if (!aiForm.customPrompt) {
          ElMessage.warning('请输入自定义指令')
          aiLoading.value = false
          return
        }
        result = await invoke<string>('ai_chat', {
          prompt: aiForm.customPrompt,
          context: currentContent
        })
        break
    }

    aiResult.value = result

    // 保存到历史记录
    const actionNames: Record<string, string> = {
      continue: '智能续写',
      polish: '内容润色',
      summarize: '内容摘要',
      suggest: '情节建议',
      custom: '自由提问'
    }

    aiHistory.value.push({
      action: actionNames[currentAIAction.value] || currentAIAction.value,
      content: result,
      time: new Date().toLocaleTimeString()
    })

  } catch (error) {
    ElMessage.error(`AI 生成失败: ${error}`)
  } finally {
    aiLoading.value = false
  }
}

// 复制结果
const copyResult = async () => {
  try {
    await navigator.clipboard.writeText(aiResult.value)
    ElMessage.success('已复制到剪贴板')
  } catch {
    ElMessage.error('复制失败')
  }
}

// 追加结果到末尾
const appendResult = () => {
  if (!aiResult.value) return
  
  if (editorRef.value) {
    editorRef.value.dangerouslyInsertHtml(`<p>${aiResult.value.replace(/\n/g, '</p><p>')}</p>`)
  } else {
    form.content += '\n' + aiResult.value
  }
  
  ElMessage.success('已追加到内容末尾')
}

// 替换选中内容
const replaceSelection = () => {
  if (!aiResult.value) return
  
  if (editorRef.value) {
    const selection = editorRef.value.getSelectionText()
    if (selection) {
      // 有选中文本，替换
      editorRef.value.deleteBackward('character')
      editorRef.value.dangerouslyInsertHtml(aiResult.value)
      ElMessage.success('已替换选中内容')
    } else {
      // 无选中，在光标处插入
      editorRef.value.dangerouslyInsertHtml(aiResult.value)
      ElMessage.success('已插入内容')
    }
  } else {
    form.content += aiResult.value
    ElMessage.success('已插入内容')
  }
}

// 使用历史结果
const useHistoryResult = (item: { content: string }) => {
  aiResult.value = item.content
}

const handleCheck = () => {
  if (!form.content) {
    ElMessage.warning('请先输入内容')
    return
  }
  ElMessage.info('内容检测功能开发中...')
}

const handleClear = () => {
  ElMessageBox.confirm(
    '确定要清空所有内容吗？',
    '提示',
    {
      confirmButtonText: '确定',
      cancelButtonText: '取消',
      type: 'warning'
    }
  ).then(() => {
      form.content = ''
      ElMessage.success('已清空')
    }).catch(() => {})
}

onMounted(async () => {
  // 检查 AI 配置
  await checkAIConfig()
  
  // 如果是编辑模式，加载章节数据
  if (chapterId.value) {
    const chapter = await novelStore.fetchChapter(chapterId.value)
    if (chapter) {
      form.title = chapter.title
      form.order_num = chapter.order_num
      form.content = chapter.content || ''
      chapterTitle.value = chapter.title
      chapterWordCount.value = chapter.word_count
    }
  }
  
  // 加载小说统计
  if (novelStore.currentNovel) {
    totalWordCount.value = novelStore.currentNovel.total_words
  }
  
  // 加载角色列表
  await loadCharacters()
  
  // 加载章节段落标注
  await loadChapterParagraphs()
})

onUnmounted(() => {
  if (editorRef.value == null) return
  editorRef.value.destroy()
  editorRef.value = null
})
</script>

<style scoped>
.chapter-edit {
  padding: 0;
}

.toolbar-card {
  position: sticky;
  top: 0;
  z-index: 100;
}

.toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.toolbar-left {
  display: flex;
  align-items: center;
  gap: 12px;
}

.chapter-info {
  font-size: 14px;
  color: #606266;
}

.editor-container {
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  overflow: hidden;
}

/* 标注模式样式 */
.markup-container {
  border: 1px solid #dcdfe6;
  border-radius: 4px;
  padding: 15px;
  min-height: 500px;
  max-height: 600px;
  overflow-y: auto;
  background: #fafafa;
}

/* 工具栏 */
.markup-toolbar {
  margin-bottom: 20px;
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
}

.paragraph-list {
  margin-top: 20px;
}

.paragraph-item {
  background: #fff;
  border: 1px solid #e4e7ed;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
  transition: all 0.3s;
}

.paragraph-item:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.paragraph-item.is-narration {
  border-left: 4px solid #909399;
}

.paragraph-item.is-dialogue {
  border-left: 4px solid #67c23a;
}

.paragraph-item.is-environment {
  border-left: 4px solid #e6a23c;
}

.para-header {
  display: flex;
  align-items: center;
  margin-bottom: 10px;
  padding-bottom: 10px;
  border-bottom: 1px dashed #ebeef5;
  flex-wrap: wrap;
  gap: 10px;
}

.para-index {
  font-size: 12px;
  color: #909399;
  margin-right: 15px;
  font-weight: 500;
}

.para-content {
  font-size: 14px;
  line-height: 1.8;
  color: #303133;
  white-space: pre-wrap;
  word-break: break-all;
  margin-top: 10px;
}

.para-content textarea {
  width: 100%;
  font-size: 14px;
  line-height: 1.8;
}

.para-actions {
  margin-left: auto;
  display: flex;
  gap: 5px;
}

.empty-paragraphs {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 300px;
}

/* 类型图例样式 */
.type-legend {
  display: flex;
  gap: 20px;
  margin-top: 15px;
  padding: 10px 15px;
  background: #f0f9ff;
  border-radius: 6px;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  color: #606266;
}

.legend-color {
  display: inline-block;
  width: 16px;
  height: 16px;
  border-radius: 3px;
}

.narration-color {
  background: #909399;
}

.dialogue-color {
  background: #67c23a;
}

.environment-color {
  background: #e6a23c;
}

.stats-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.stat-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.stat-label {
  font-size: 14px;
  color: #606266;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}

.action-buttons {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.action-btn {
  justify-content: flex-start;
}
</style>

<style>
@import '@wangeditor/editor/dist/css/style.css';
</style>
