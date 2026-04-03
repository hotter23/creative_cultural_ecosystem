<template>
  <div class="dashboard">
    <!-- 统计卡片 -->
    <el-row :gutter="20" class="stats-row">
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon novel-icon">
              <el-icon><Document /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stats.novels }}</div>
              <div class="stat-label">网文项目</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon chapter-icon">
              <el-icon><Files /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stats.chapters }}</div>
              <div class="stat-label">章节总数</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon audio-icon">
              <el-icon><Microphone /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stats.audioProjects }}</div>
              <div class="stat-label">音频项目</div>
            </div>
          </div>
        </el-card>
      </el-col>
      <el-col :span="6">
        <el-card class="stat-card">
          <div class="stat-content">
            <div class="stat-icon video-icon">
              <el-icon><VideoCamera /></el-icon>
            </div>
            <div class="stat-info">
              <div class="stat-value">{{ stats.videos }}</div>
              <div class="stat-label">视频项目</div>
            </div>
          </div>
        </el-card>
      </el-col>
    </el-row>

    <!-- 最近项目 -->
    <el-row :gutter="20" class="content-row">
      <el-col :span="24">
        <el-card class="recent-projects">
          <template #header>
            <div class="card-header">
              <span>最近项目</span>
              <div class="header-actions">
                <el-button type="primary" @click="createNovel">
                  <el-icon><Plus /></el-icon>
                  新建网文
                </el-button>
                <el-button link @click="goToNovels">查看全部</el-button>
              </div>
            </div>
          </template>
          <el-table :data="recentNovels" v-loading="loading">
            <el-table-column prop="title" label="项目名称" min-width="200" />
            <el-table-column prop="status" label="状态" width="100">
              <template #default="{ row }">
                <el-tag :type="getStatusType(row.status)" size="small">{{ getStatusLabel(row.status) }}</el-tag>
              </template>
            </el-table-column>
            <el-table-column prop="total_chapters" label="章节数" width="100" align="center" />
            <el-table-column prop="total_words" label="字数" width="100" align="center" />
            <el-table-column prop="updated_at" label="更新时间" width="180">
              <template #default="{ row }">
                {{ formatDate(row.updated_at) }}
              </template>
            </el-table-column>
            <el-table-column label="操作" width="150" fixed="right">
              <template #default="{ row }">
                <el-button type="primary" link @click="editNovel(row)">编辑</el-button>
                <el-button type="success" link @click="goToChapters(row)">章节</el-button>
              </template>
            </el-table-column>
          </el-table>
        </el-card>
      </el-col>
    </el-row>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useNovelStore, type Novel } from '../../stores/novel'
import dayjs from 'dayjs'
import {
  Document,
  Files,
  Microphone,
  VideoCamera,
  Plus
} from '@element-plus/icons-vue'

const router = useRouter()
const novelStore = useNovelStore()

const loading = ref(false)
const recentNovels = ref<Novel[]>([])

const stats = ref({
  novels: 0,
  chapters: 0,
  audioProjects: 0,
  videos: 0
})

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

const goToNovels = () => {
  router.push('/novels')
}

const createNovel = () => {
  router.push('/novels/create')
}

const editNovel = (novel: Novel) => {
  router.push(`/novels/${novel.id}`)
}

const goToChapters = (novel: Novel) => {
  router.push(`/novels/${novel.id}/chapters`)
}

onMounted(async () => {
  loading.value = true
  await novelStore.fetchNovels()
  recentNovels.value = novelStore.novels.slice(0, 5)
  stats.value.novels = novelStore.novels.length
  stats.value.chapters = novelStore.novels.reduce((sum, n) => sum + n.total_chapters, 0)
  loading.value = false
})
</script>

<style scoped>
.dashboard {
  padding: 0;
}

.stats-row {
  margin-bottom: 20px;
}

.stat-card {
  border: none;
  border-radius: 8px;
}

.stat-content {
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-icon {
  width: 50px;
  height: 50px;
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 24px;
  color: #fff;
}

.novel-icon {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
}

.chapter-icon {
  background: linear-gradient(135deg, #f093fb 0%, #f5576c 100%);
}

.audio-icon {
  background: linear-gradient(135deg, #4facfe 0%, #00f2fe 100%);
}

.video-icon {
  background: linear-gradient(135deg, #43e97b 0%, #38f9d7 100%);
}

.stat-info {
  flex: 1;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #303133;
  line-height: 1;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-top: 4px;
}

.content-row {
  margin-bottom: 20px;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 12px;
  align-items: center;
}
</style>
