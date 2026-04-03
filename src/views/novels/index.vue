<template>
  <div class="novels-page">
    <!-- 搜索和筛选 -->
    <el-card class="filter-card" shadow="never">
      <el-form :inline="true" :model="filterForm" class="filter-form">
        <el-form-item label="搜索">
          <el-input
            v-model="filterForm.keyword"
            placeholder="输入项目名称搜索"
            clearable
            style="width: 240px"
            @input="handleSearch"
          />
        </el-form-item>
        <el-form-item label="状态">
          <el-select v-model="filterForm.status" clearable placeholder="全部状态" @change="handleFilter">
            <el-option label="草稿" value="draft" />
            <el-option label="写作中" value="writing" />
            <el-option label="已完成" value="completed" />
            <el-option label="已发布" value="published" />
          </el-select>
        </el-form-item>
        <el-form-item>
          <el-button type="primary" @click="handleCreate">
            <el-icon><Plus /></el-icon>
            新建项目
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 项目列表 -->
    <el-card class="list-card" shadow="never" style="margin-top: 20px;">
      <el-table
        :data="filteredNovels"
        v-loading="novelStore.loading"
        empty-text="暂无项目，点击右上角创建第一个网文项目"
      >
        <el-table-column prop="title" label="项目名称" min-width="200">
          <template #default="{ row }">
            <div class="novel-title">
              <div class="novel-cover" v-if="row.cover_path">
                <img :src="row.cover_path" :alt="row.title" />
              </div>
              <div class="novel-cover default-cover" v-else>
                <el-icon><Document /></el-icon>
              </div>
              <div class="novel-info">
                <div class="title-text">{{ row.title }}</div>
                <div class="title-desc" v-if="row.description">{{ row.description }}</div>
              </div>
            </div>
          </template>
        </el-table-column>
        <el-table-column prop="status" label="状态" width="100">
          <template #default="{ row }">
            <el-tag :type="getStatusType(row.status)" size="small">
              {{ getStatusLabel(row.status) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="current_stage" label="当前阶段" width="100">
          <template #default="{ row }">
            <el-tag type="info" size="small">
              {{ getStageLabel(row.current_stage) }}
            </el-tag>
          </template>
        </el-table-column>
        <el-table-column prop="total_chapters" label="章节数" width="80" align="center" />
        <el-table-column prop="total_words" label="字数" width="100" align="center">
          <template #default="{ row }">
            {{ (row.total_words / 1000).toFixed(1) }}k
          </template>
        </el-table-column>
        <el-table-column prop="updated_at" label="更新时间" width="160">
          <template #default="{ row }">
            {{ formatDate(row.updated_at) }}
          </template>
        </el-table-column>
        <el-table-column label="操作" width="220" fixed="right">
          <template #default="{ row }">
            <el-button type="primary" link @click="handleEdit(row)">编辑</el-button>
            <el-button type="success" link @click="handleChapters(row)">章节</el-button>
            <!-- <el-button type="warning" link @click="handleAudio(row)">音频</el-button> -->
            <el-button type="danger" link @click="handleDelete(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </el-card>

    <!-- 删除确认对话框 -->
    <el-dialog
      v-model="deleteDialogVisible"
      title="确认删除"
      width="400px"
      :close-on-click-modal="false"
    >
      <p>确定要删除该网文项目吗？此操作将同时删除所有相关的章节、音频和视频数据，且无法恢复。</p>
      <template #footer>
        <el-button @click="deleteDialogVisible = false">取消</el-button>
        <el-button type="danger" :loading="deleting" @click="confirmDelete">确认删除</el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useNovelStore, type Novel } from '../../stores/novel'
import dayjs from 'dayjs'
import { Plus, Document } from '@element-plus/icons-vue'

const router = useRouter()
const novelStore = useNovelStore()

const filterForm = reactive({
  keyword: '',
  status: ''
})

const deleteDialogVisible = ref(false)
const deleting = ref(false)
const currentDeleteId = ref<number | null>(null)

const filteredNovels = computed(() => {
  let list = novelStore.novels
  if (filterForm.keyword) {
    const keyword = filterForm.keyword.toLowerCase()
    list = list.filter(n => n.title.toLowerCase().includes(keyword))
  }
  if (filterForm.status) {
    list = list.filter(n => n.status === filterForm.status)
  }
  return list
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

const getStageLabel = (stage: string) => {
  const labels: Record<string, string> = {
    novel: '网文创作',
    audio: '音频制作',
    character: '角色设计',
    video: '视频生成'
  }
  return labels[stage] || stage
}

const formatDate = (date: string) => {
  return dayjs(date).format('YYYY-MM-DD HH:mm')
}

const handleSearch = () => {
  // 搜索由 computed 自动处理
}

const handleFilter = () => {
  // 筛选由 computed 自动处理
}

const handleCreate = () => {
  router.push('/novels/create')
}

const handleEdit = (novel: Novel) => {
  router.push(`/novels/${novel.id}`)
}

const handleChapters = (novel: Novel) => {
  router.push(`/novels/${novel.id}/chapters`)
}

const handleAudio = (novel: Novel) => {
  // TODO: 跳转到音频制作页面
  ElMessage.info('音频制作功能开发中...')
}

const handleDelete = (novel: Novel) => {
  currentDeleteId.value = novel.id
  deleteDialogVisible.value = true
}

const confirmDelete = async () => {
  if (!currentDeleteId.value) return
  
  deleting.value = true
  try {
    await novelStore.deleteNovel(currentDeleteId.value)
    ElMessage.success('删除成功')
    deleteDialogVisible.value = false
  } catch (error) {
    ElMessage.error('删除失败，请重试')
  } finally {
    deleting.value = false
    currentDeleteId.value = null
  }
}

onMounted(() => {
  novelStore.fetchNovels()
})
</script>

<style scoped>
.novels-page {
  padding: 0;
}

.filter-card {
  margin-bottom: 0;
}

.filter-form {
  margin-bottom: 0;
}

.novel-title {
  display: flex;
  align-items: center;
  gap: 12px;
}

.novel-cover {
  width: 48px;
  height: 64px;
  border-radius: 4px;
  overflow: hidden;
  background: #f5f7fa;
  flex-shrink: 0;
}

.novel-cover img {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.default-cover {
  display: flex;
  align-items: center;
  justify-content: center;
  color: #c0c4cc;
  font-size: 24px;
}

.novel-info {
  flex: 1;
  min-width: 0;
}

.title-text {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.title-desc {
  font-size: 12px;
  color: #909399;
  margin-top: 2px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
