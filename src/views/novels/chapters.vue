<template>
  <div class="chapters-page">
    <!-- 项目信息头部 -->
    <el-card class="novel-header-card" shadow="never" v-if="novelStore.currentNovel">
      <div class="novel-header">
        <div class="novel-cover">
          <img :src="novelStore.currentNovel.cover_path || '/src/assets/vue.svg'" :alt="novelStore.currentNovel.title" v-if="novelStore.currentNovel.cover_path" />
          <div class="default-cover" v-else>
            <el-icon><Document /></el-icon>
          </div>
        </div>
        <div class="novel-info">
          <h2 class="novel-title">{{ novelStore.currentNovel.title }}</h2>
          <p class="novel-desc" v-if="novelStore.currentNovel.description">{{ novelStore.currentNovel.description }}</p>
          <div class="novel-stats">
            <span class="stat-item">章节：{{ novelStore.currentNovel.total_chapters }}</span>
            <span class="stat-item">字数：{{ (novelStore.currentNovel.total_words / 1000).toFixed(1) }}k</span>
            <el-tag :type="getStatusType(novelStore.currentNovel.status)" size="small">
              {{ getStatusLabel(novelStore.currentNovel.status) }}
            </el-tag>
          </div>
        </div>
        <div class="novel-actions">
          <el-button type="primary" @click="handleCreateChapter">
            <el-icon><Plus /></el-icon>
            新建章节
          </el-button>
          <el-button @click="handleBack">返回列表</el-button>
        </div>
      </div>
    </el-card>

    <!-- 章节列表 -->
    <el-card class="chapters-card" shadow="never" style="margin-top: 20px;">
      <template #header>
        <div class="card-header">
          <span>章节列表（{{ novelStore.chapters.length }}）</span>
        </div>
      </template>
      
      <!-- 自定义章节列表 -->
      <div class="chapter-list" v-if="!novelStore.loading">
        <div 
          v-for="chapter in novelStore.chapters" 
          :key="chapter.id" 
          class="chapter-list-item"
        >
          <div class="chapter-item-left">
            <div class="chapter-order">{{ chapter.order_num }}</div>
            <div class="chapter-info">
              <div class="chapter-title-row">
                <span class="chapter-title-text">{{ chapter.title }}</span>
                <el-tag size="small" type="info" v-if="chapter.status === 'draft'">草稿</el-tag>
                <el-tag size="small" type="success" v-if="chapter.status === 'completed'">已完成</el-tag>
                <el-tag 
                  size="small" 
                  :type="chapter.audio_status === 'completed' ? 'success' : chapter.audio_status === 'processing' ? 'warning' : 'info'"
                  class="audio-status-tag"
                  v-if="chapter.audio_status && chapter.audio_status !== 'not_created'"
                >
                  {{ chapter.audio_status === 'completed' ? '音频已完成' : chapter.audio_status === 'processing' ? '生成中' : '未生成' }}
                </el-tag>
              </div>
              <div class="chapter-meta">
                <span class="chapter-word-count">字数: {{ chapter.word_count || 0 }}</span>
                <span class="chapter-date">{{ formatDate(chapter.created_at) }}</span>
              </div>
            </div>
          </div>
          <div class="chapter-actions">
            <el-button type="primary" link @click="handleEdit(chapter)">编辑</el-button>
            <el-button type="warning" link @click="handlePreview(chapter)">预览</el-button>
            <el-button type="danger" link @click="handleDelete(chapter)">删除</el-button>
          </div>
        </div>
        <el-empty 
          v-if="novelStore.chapters.length === 0" 
          description="暂无章节，点击上方按钮创建第一个章节" 
          style="padding: 40px 0;"
        />
      </div>
      
      <!-- 加载状态 -->
      <div class="chapter-list-loading" v-if="novelStore.loading">
        <el-skeleton active :rows="5" />
      </div>
    </el-card>

    <!-- 章节预览对话框 -->
    <el-dialog
      v-model="previewDialogVisible"
      title="章节预览"
      width="800px"
      top="5vh"
      :close-on-click-modal="true"
      @closed="handlePreviewDialogClose"
    >
      <template #default>
        <div class="preview-content" v-if="previewChapter">
          <div class="preview-stats">
            <el-tag size="small">字数：{{ previewChapter.word_count || 0 }}</el-tag>
            <el-tag size="small" style="margin-left: 10px;">创建时间：{{ formatDate(previewChapter.created_at) }}</el-tag>
          </div>
          <div class="preview-text">
           
            <div class="html-preview" v-if="previewChapter.content">
              <div v-html="previewChapter.content"></div>
            </div>
            <div class="empty-preview" v-else>
              <el-empty description="该章节暂无内容" />
            </div>
          </div>
        </div>
      </template>
      <template #footer>
        <el-button @click="previewDialogVisible = false">关闭</el-button>
        <el-button 
          type="primary" 
          :disabled="!previewChapter"
          @click="handlePreviewEdit"
        >
          编辑章节
        </el-button>
      </template>
    </el-dialog>

    <!-- 删除确认对话框 -->
    <el-dialog
      v-model="deleteDialogVisible"
      title="确认删除"
      width="400px"
      :close-on-click-modal="false"
    >
      <p>确定要删除该章节吗？此操作将同时删除相关的音频数据，且无法恢复。</p>
      <template #footer>
        <el-button @click="deleteDialogVisible = false">取消</el-button>
        <el-button type="danger" :loading="deleting" @click="confirmDelete">确认删除</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, nextTick } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useNovelStore, type Chapter } from '../../stores/novel'
import dayjs from 'dayjs'
import { Plus, Document } from '@element-plus/icons-vue'
import { ElSkeleton, ElEmpty } from 'element-plus'

const router = useRouter()
const route = useRoute()
const novelStore = useNovelStore()

const deleteDialogVisible = ref(false)
const deleting = ref(false)
const currentDeleteId = ref<number | null>(null)

// 章节预览相关
const previewDialogVisible = ref(false)
const previewChapter = ref<Chapter | null>(null)

const handlePreviewDialogClose = () => {
  nextTick(() => {
    previewChapter.value = null
  })
}

const novelId = computed(() => parseInt(route.params.id as string))

const getStatusType = (status: string) => {
  const types: Record<string, string> = {
    draft: 'info',
    writing: 'primary',
    completed: 'success',
    published: 'success'
  }
  return types[status] || 'info'
}

const getStatusLabel = (status: string) => {
  const labels: Record<string, string> = {
    draft: '草稿',
    writing: '写作中',
    completed: '已完成',
    published: '已发布'
  }
  return labels[status] || status
}

const formatDate = (date: string) => {
  return dayjs(date).format('YYYY-MM-DD HH:mm')
}

const handleBack = () => {
  router.push('/novels')
}

const handleCreateChapter = () => {
  router.push(`/novels/${novelId.value}/chapters/create`)
}

const handleEdit = (chapter: Chapter) => {
  router.push(`/novels/${novelId.value}/chapters/${chapter.id}/edit`)
}

const handlePreview = (chapter: Chapter) => {
  previewChapter.value = chapter
  previewDialogVisible.value = true
}

const handlePreviewEdit = () => {
  if (previewChapter.value) {
    handleEdit(previewChapter.value)
  }
}

const handleDelete = (chapter: Chapter) => {
  currentDeleteId.value = chapter.id
  deleteDialogVisible.value = true
}

const confirmDelete = async () => {
  if (!currentDeleteId.value) return
  
  deleting.value = true
  try {
    await novelStore.deleteChapter(currentDeleteId.value)
    ElMessage.success('删除成功')
    deleteDialogVisible.value = false
  } catch (error) {
    ElMessage.error('删除失败，请重试')
  } finally {
    deleting.value = false
    currentDeleteId.value = null
  }
}

onMounted(async () => {
  if (novelId.value) {
    await novelStore.fetchNovel(novelId.value)
    await novelStore.fetchChapters(novelId.value)
  }
})
</script>

<style scoped>
.chapters-page {
  padding: 0;
}

.novel-header-card {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border: none;
}

.novel-header {
  display: flex;
  gap: 20px;
  align-items: flex-start;
}

.novel-cover {
  width: 120px;
  height: 160px;
  border-radius: 8px;
  overflow: hidden;
  background: rgba(255, 255, 255, 0.2);
  flex-shrink: 0;
}

.novel-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.default-cover {
  width: 100%;
  height: 100%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: rgba(255, 255, 255, 0.6);
  font-size: 48px;
}

.novel-info {
  flex: 1;
  min-width: 0;
}

.novel-title {
  margin: 0 0 8px 0;
  font-size: 24px;
  font-weight: 600;
  color: #fff;
}

.novel-desc {
  margin: 0 0 12px 0;
  font-size: 14px;
  color: rgba(255, 255, 255, 0.8);
  line-height: 1.6;
}

.novel-stats {
  display: flex;
  gap: 20px;
  align-items: center;
}

.stat-item {
  font-size: 14px;
  color: rgba(255, 255, 255, 0.9);
}

.novel-actions {
  display: flex;
  gap: 12px;
  flex-shrink: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.chapter-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

/* 自定义章节列表样式 */
.chapter-list {
  min-height: 400px;
}

.chapter-list-item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 16px 20px;
  border-bottom: 1px solid #ebeef5;
  transition: background-color 0.2s;
}

.chapter-list-item:hover {
  background-color: #f5f7fa;
}

.chapter-list-item:last-child {
  border-bottom: none;
}

.chapter-item-left {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  flex: 1;
  min-width: 0;
}

.chapter-order {
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #f0f2f5;
  border-radius: 6px;
  font-weight: 600;
  color: #606266;
  flex-shrink: 0;
}

.chapter-info {
  flex: 1;
  min-width: 0;
}

.chapter-title-row {
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
}

.chapter-title-text {
  font-size: 15px;
  font-weight: 500;
  color: #303133;
  max-width: 400px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.audio-status-tag {
  margin-left: 8px;
}

.chapter-meta {
  display: flex;
  gap: 20px;
  font-size: 13px;
  color: #909399;
}

.chapter-actions {
  display: flex;
  gap: 8px;
  flex-shrink: 0;
}

/* 章节预览样式 */
.preview-content {
  padding: 10px 0;
}

.preview-stats {
  margin-bottom: 20px;
  padding-bottom: 15px;
  border-bottom: 1px solid #ebeef5;
}

.preview-text {
  min-height: 400px;
  max-height: 60vh;
  overflow-y: auto;
  padding-right: 10px;
}

.plain-text-preview {
  line-height: 1.8;
  font-size: 15px;
  color: #303133;
}

.plain-text-preview p {
  margin: 0 0 12px 0;
  text-indent: 2em;
}

.html-preview {
  line-height: 1.8;
  font-size: 15px;
  color: #303133;
}

.html-preview :deep(p) {
  margin: 0 0 12px 0;
  text-indent: 2em;
}

.empty-preview {
  display: flex;
  justify-content: center;
  align-items: center;
  min-height: 300px;
}

/* 自定义滚动条 */
.preview-text::-webkit-scrollbar {
  width: 6px;
}

.preview-text::-webkit-scrollbar-thumb {
  background: #d0d0d0;
  border-radius: 3px;
}

.preview-text::-webkit-scrollbar-track {
  background: #f5f5f5;
}
</style>
