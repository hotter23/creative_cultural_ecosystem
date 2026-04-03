<template>
  <div class="novel-detail">
    <el-card shadow="never">
      <template #header>
        <div class="card-header">
          <span>网文详情</span>
          <el-button @click="handleBack">返回列表</el-button>
        </div>
      </template>
      
      <div v-if="novelStore.currentNovel" class="detail-content">
        <el-form :model="form" label-width="100px" style="max-width: 600px;">
          <el-form-item label="项目名称">
            <el-input v-model="form.title" />
          </el-form-item>
          <el-form-item label="项目描述">
            <el-input v-model="form.description" type="textarea" :rows="4" />
          </el-form-item>
          <el-form-item label="项目状态">
            <el-select v-model="form.status">
              <el-option label="草稿" value="draft" />
              <el-option label="写作中" value="writing" />
              <el-option label="已完成" value="completed" />
              <el-option label="已发布" value="published" />
            </el-select>
          </el-form-item>
          <el-form-item label="当前阶段">
            <el-select v-model="form.current_stage">
              <el-option label="网文创作" value="novel" />
              <el-option label="音频制作" value="audio" />
              <el-option label="角色设计" value="character" />
              <el-option label="视频生成" value="video" />
            </el-select>
          </el-form-item>
          <el-form-item>
            <el-button type="primary" :loading="saving" @click="handleSave">保存修改</el-button>
            <el-button @click="handleChapters">管理章节</el-button>
          </el-form-item>
        </el-form>
        
        <el-divider />
        
        <div class="stats-info">
          <h4>项目统计</h4>
          <el-row :gutter="20">
            <el-col :span="6">
              <div class="stat-item">
                <div class="stat-label">章节数</div>
                <div class="stat-value">{{ novelStore.currentNovel.total_chapters }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="stat-item">
                <div class="stat-label">总字数</div>
                <div class="stat-value">{{ novelStore.currentNovel.total_words }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="stat-item">
                <div class="stat-label">创建时间</div>
                <div class="stat-value">{{ formatDate(novelStore.currentNovel.created_at) }}</div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="stat-item">
                <div class="stat-label">更新时间</div>
                <div class="stat-value">{{ formatDate(novelStore.currentNovel.updated_at) }}</div>
              </div>
            </el-col>
          </el-row>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted, computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { ElMessage } from 'element-plus'
import { useNovelStore } from '../../stores/novel'
import dayjs from 'dayjs'

const router = useRouter()
const route = useRoute()
const novelStore = useNovelStore()

const saving = ref(false)
const novelId = computed(() => parseInt(route.params.id as string))

const form = reactive({
  title: '',
  description: '',
  status: '',
  current_stage: ''
})

const formatDate = (date: string) => {
  return dayjs(date).format('YYYY-MM-DD HH:mm')
}

const handleBack = () => {
  router.push('/novels')
}

const handleChapters = () => {
  router.push(`/novels/${novelId.value}/chapters`)
}

const handleSave = async () => {
  saving.value = true
  try {
    await novelStore.updateNovel(novelId.value, {
      title: form.title,
      description: form.description,
      status: form.status,
      current_stage: form.current_stage
    })
    ElMessage.success('保存成功')
  } catch (error) {
    ElMessage.error('保存失败，请重试')
  } finally {
    saving.value = false
  }
}

onMounted(async () => {
  if (novelId.value) {
    await novelStore.fetchNovel(novelId.value)
    if (novelStore.currentNovel) {
      form.title = novelStore.currentNovel.title
      form.description = novelStore.currentNovel.description || ''
      form.status = novelStore.currentNovel.status
      form.current_stage = novelStore.currentNovel.current_stage
    }
  }
})
</script>

<style scoped>
.novel-detail {
  padding: 0;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.detail-content {
  padding: 20px 0;
}

.stats-info {
  margin-top: 20px;
}

.stats-info h4 {
  margin: 0 0 20px 0;
  color: #303133;
}

.stat-item {
  text-align: center;
  padding: 20px;
  background: #f5f7fa;
  border-radius: 8px;
}

.stat-label {
  font-size: 14px;
  color: #909399;
  margin-bottom: 8px;
}

.stat-value {
  font-size: 24px;
  font-weight: 600;
  color: #303133;
}
</style>
