<template>
  <div class="chapter-create">
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
            新建章节
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
              <div class="editor-container">
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
              <span class="stat-label">预计字数</span>
              <span class="stat-value">--</span>
            </div>
            <div class="stat-item">
              <span class="stat-label">小说总字数</span>
              <span class="stat-value">{{ totalWordCount }}</span>
            </div>
          </div>
        </el-card>

        <!-- 快捷操作 -->
        <el-card class="quick-actions-card" shadow="never" style="margin-top: 20px;">
          <template #header>
            <span>快捷操作</span>
          </template>
          <div class="action-buttons">
            <el-button class="action-btn" block @click="handleAIGenerate">
              <el-icon><MagicStick /></el-icon>
              AI 生成
            </el-button>
            <el-button class="action-btn" block type="success" @click="handleAIOptimize">
              <el-icon><Edit /></el-icon>
              AI 润色
            </el-button>
            <el-button class="action-btn" block type="warning" @click="handleCheck">
              <el-icon><DocumentChecked /></el-icon>
              内容检测
            </el-button>
            <el-button class="action-btn" block type="danger" @click="handleClear">
              <el-icon><Delete /></el-icon>
              清空内容
            </el-button>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- AI 对话框 -->
    <el-dialog
      v-model="aiDialogVisible"
      title="AI 内容生成"
      width="600px"
      :close-on-click-modal="false"
    >
      <el-form :model="aiForm">
        <el-form-item label="生成要求">
          <el-input
            v-model="aiForm.prompt"
            type="textarea"
            :rows="4"
            placeholder="请描述你想要生成的内容..."
          />
        </el-form-item>
        <el-form-item label="生成长度">
          <el-select v-model="aiForm.length">
            <el-option label="短篇 (约500字)" :value="500" />
            <el-option label="中篇 (约1000字)" :value="1000" />
            <el-option label="长篇 (约2000字)" :value="2000" />
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="aiDialogVisible = false">取消</el-button>
        <el-button type="primary" :loading="aiGenerating" @click="confirmAIGenerate">生成</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted, onUnmounted, shallowRef } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useNovelStore } from '../../stores/novel'
import { Editor, Toolbar } from '@wangeditor/editor-for-vue'
import type { IDomEditor } from '@wangeditor/editor'
import {
  ArrowLeft,
  MagicStick,
  Edit,
  DocumentChecked,
  Delete
} from '@element-plus/icons-vue'

const router = useRouter()
const route = useRoute()
const novelStore = useNovelStore()

const saving = ref(false)
const aiDialogVisible = ref(false)
const aiGenerating = ref(false)

const novelId = computed(() => parseInt(route.params.novelId as string))

const totalWordCount = ref(0)

// 编辑器实例
const editorRef = shallowRef<IDomEditor | null>(null)
const mode = ref<'default' | 'simple'>('default')

const form = reactive({
  title: '',
  order_num: 1,
  content: ''
})

const aiForm = reactive({
  prompt: '',
  length: 1000
})

const wordCount = computed(() => {
  if (!editorRef.value) return 0
  return editorRef.value.getText().replace(/\s/g, '').length
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
    await novelStore.createChapter({
      novel_id: novelId.value,
      title: form.title,
      content: form.content,
      order_num: form.order_num
    })
    
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
    await novelStore.createChapter({
      novel_id: novelId.value,
      title: form.title,
      content: form.content,
      order_num: form.order_num
    })
    
    ElMessage.success('章节创建成功！')
    router.push(`/novels/${novelId.value}/chapters`)
  } catch (error) {
    ElMessage.error('保存失败，请重试')
  } finally {
    saving.value = false
  }
}

const handleAIGenerate = () => {
  aiForm.prompt = ''
  aiDialogVisible.value = true
}

const handleAIOptimize = () => {
  if (!form.content) {
    ElMessage.warning('请先输入内容')
    return
  }
  ElMessage.info('AI 润色功能开发中...')
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

const confirmAIGenerate = () => {
  if (!aiForm.prompt) {
    ElMessage.warning('请输入生成要求')
    return
  }
  aiGenerating.value = true
  
  // 模拟 AI 生成
  setTimeout(() => {
    const generatedText = `这是 AI 根据您的要求生成的内容：\n\n${aiForm.prompt}\n\n（此处为 AI 生成的实际内容，开发中...）`
    form.content += `<p>${generatedText.replace(/\n/g, '</p><p>')}</p>`
    aiDialogVisible.value = false
    aiGenerating.value = false
    ElMessage.success('生成完成')
  }, 2000)
}

onMounted(() => {
  // 设置默认序号
  form.order_num = novelStore.chapters.length + 1
  
  // 加载小说统计
  if (novelStore.currentNovel) {
    totalWordCount.value = novelStore.currentNovel.total_words
  }
})

onUnmounted(() => {
  if (editorRef.value == null) return
  editorRef.value.destroy()
  editorRef.value = null
})
</script>

<style scoped>
.chapter-create {
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
