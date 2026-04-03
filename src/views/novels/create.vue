<template>
  <div class="novel-create">
    <el-card shadow="never">
      <template #header>
        <span>创建网文项目</span>
      </template>
      
      <el-form
        ref="formRef"
        :model="form"
        :rules="rules"
        label-width="100px"
        style="max-width: 600px;"
      >
        <el-form-item label="项目名称" prop="title">
          <el-input
            v-model="form.title"
            placeholder="请输入网文项目名称"
            maxlength="100"
            show-word-limit
          />
        </el-form-item>
        
        <el-form-item label="项目描述" prop="description">
          <el-input
            v-model="form.description"
            type="textarea"
            :rows="4"
            placeholder="请输入网文项目简介"
            maxlength="500"
            show-word-limit
          />
        </el-form-item>
        
        <el-form-item label="封面图片">
          <div class="cover-upload">
            <el-upload
              class="avatar-uploader"
              :show-file-list="false"
              :before-upload="beforeUpload"
              :on-success="handleUploadSuccess"
              action="#"
            >
              <div v-if="form.cover_path" class="cover-preview">
                <img :src="form.cover_path" class="cover-image" />
              </div>
              <div v-else class="cover-placeholder">
                <el-icon class="cover-icon"><Plus /></el-icon>
                <div class="cover-text">点击上传封面</div>
              </div>
            </el-upload>
            <div class="cover-tips">
              <p>建议尺寸：600 × 800 像素</p>
              <p>支持格式：JPG、PNG、WEBP</p>
              <p>文件大小不超过 5MB</p>
            </div>
          </div>
        </el-form-item>
        
        <el-form-item>
          <el-button type="primary" :loading="submitting" @click="handleSubmit">
            创建项目
          </el-button>
          <el-button @click="handleCancel">取消</el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue'
import { useRouter } from 'vue-router'
import { ElMessage, type FormInstance, type UploadProps } from 'element-plus'
import { useNovelStore } from '../../stores/novel'
import { Plus } from '@element-plus/icons-vue'

const router = useRouter()
const novelStore = useNovelStore()

const formRef = ref<FormInstance>()
const submitting = ref(false)

const form = reactive({
  title: '',
  description: '',
  cover_path: ''
})

const rules = {
  title: [
    { required: true, message: '请输入项目名称', trigger: 'blur' },
    { min: 2, max: 100, message: '长度在 2 到 100 个字符', trigger: 'blur' }
  ]
}

const beforeUpload: UploadProps['beforeUpload'] = (file) => {
  const isImage = file.type.startsWith('image/')
  const isLt5M = file.size / 1024 / 1024 < 5
  
  if (!isImage) {
    ElMessage.error('只能上传图片文件！')
    return false
  }
  if (!isLt5M) {
    ElMessage.error('图片大小不能超过 5MB！')
    return false
  }
  
  // 转换为base64预览
  const reader = new FileReader()
  reader.onload = (e) => {
    form.cover_path = e.target?.result as string
  }
  reader.readAsDataURL(file)
  
  return false // 阻止自动上传，后续可以实现保存到本地文件
}

const handleUploadSuccess = () => {
  // 上传成功处理
}

const handleSubmit = async () => {
  if (!formRef.value) return
  
  try {
    await formRef.value.validate()
    submitting.value = true
    
    const novel = await novelStore.createNovel({
      title: form.title,
      description: form.description
    })
    
    ElMessage.success('创建成功！')
    router.push(`/novels/${novel.id}/chapters`)
  } catch (error) {
    if (error !== false) {
      ElMessage.error('创建失败，请重试')
    }
  } finally {
    submitting.value = false
  }
}

const handleCancel = () => {
  router.back()
}
</script>

<style scoped>
.novel-create {
  padding: 0;
}

.cover-upload {
  display: flex;
  gap: 20px;
  align-items: flex-start;
}

.avatar-uploader {
  display: block;
}

.cover-preview,
.cover-placeholder {
  width: 150px;
  height: 200px;
  border: 2px dashed #dcdfe6;
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.3s;
  overflow: hidden;
}

.cover-preview:hover,
.cover-placeholder:hover {
  border-color: #409eff;
}

.cover-image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.cover-icon {
  font-size: 36px;
  color: #8c939d;
}

.cover-text {
  font-size: 12px;
  color: #8c939d;
  margin-top: 8px;
}

.cover-tips {
  padding-top: 8px;
}

.cover-tips p {
  margin: 0;
  font-size: 12px;
  color: #909399;
  line-height: 1.8;
}
</style>
