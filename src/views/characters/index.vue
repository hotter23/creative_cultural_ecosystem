<template>
  <div class="characters-page">
    <!-- 搜索和筛选 -->
    <el-card class="filter-card" shadow="never">
      <el-form :inline="true" :model="filterForm" class="filter-form">
        <el-form-item label="所属小说">
          <el-select
            v-model="filterForm.novelId"
            clearable
            placeholder="选择小说"
            style="width: 240px"
            @change="loadCharacters"
          >
            <el-option
              v-for="novel in novels"
              :key="novel.id"
              :label="novel.title"
              :value="novel.id"
            />
          </el-select>
        </el-form-item>
        <el-form-item label="搜索">
          <el-input
            v-model="filterForm.keyword"
            placeholder="输入角色名称搜索"
            clearable
            style="width: 200px"
            @input="handleSearch"
          />
        </el-form-item>
        <el-form-item>
          <el-button
            type="primary"
            @click="handleCreate"
            :disabled="!filterForm.novelId"
          >
            <el-icon><Plus /></el-icon>
            添加角色
          </el-button>
          <!-- <el-button
            type="success"
            @click="handleExtractCharacters"
            :loading="extracting"
            :disabled="!filterForm.novelId"
          >
            <el-icon><MagicStick /></el-icon>
            AI 提取角色
          </el-button> -->
        </el-form-item>
      </el-form>
    </el-card>

    <!-- 角色列表 -->
    <el-card class="list-card" shadow="never" style="margin-top: 20px;">
      <el-row :gutter="20">
        <el-col
          v-for="character in filteredCharacters"
          :key="character.id"
          :xs="24"
          :sm="12"
          :md="8"
          :lg="6"
          :xl="4"
        >
          <el-card class="character-card" shadow="hover">
            <div class="character-avatar">
              <div
                class="avatar-image"
                :style="{
                  background: getAvatarColor(character),
                }"
              >
                <el-image
                  v-if="character.defaultImage"
                  :src="character.defaultImage"
                  class="image"
                  fit="cover"
                  :alt="character.name"
                  :preview-src-list="[character.defaultImage]"
                  preview-teleported
                  style="cursor: pointer;"
                />
                <span v-else>{{ character.name.charAt(0) }}</span>
              </div>
            </div>
            <div class="character-info">
              <h3 class="character-name">{{ character.name }}</h3>
              <p class="character-meta" v-if="character.gender || character.role">
                <span v-if="character.gender" class="meta-tag">{{ getGenderLabel(character.gender) }}</span>
                <span v-if="character.role" class="role-tag">{{ character.role }}</span>
              </p>
              <div class="voice-info" :class="{ 'voice-info-empty': !character.voice_id }">
                <span class="voice-icon"><el-icon><Microphone /></el-icon></span>
                <span class="voice-name">{{ getVoiceName(character.voice_id) || '未绑定音色' }}</span>
              </div>
              <p class="character-desc" v-if="character.description">
                {{ character.description.substring(0, 50) }}{{ character.description.length > 50 ? '...' : '' }}
              </p>
            </div>
            <div class="character-actions">
              <el-button size="small" @click="handleViewCharacter(character)">
                详情
              </el-button>
              <el-button size="small" @click="handleEditCharacter(character)">
                编辑
              </el-button>
              <el-button size="small" type="primary" @click="handleGenerateImage(character)">
                生成形象
              </el-button>
              <el-button
                size="small"
                type="danger"
                @click="handleDeleteCharacter(character)"
              >
                删除
              </el-button>
            </div>
          </el-card>
        </el-col>

        <el-col
          v-if="characters.length === 0 && !loading"
          :xs="24"
          style="text-align: center; padding: 60px 0"
        >
          <el-empty
            description="暂无角色，先选择小说后添加或提取角色"
          />
        </el-col>
      </el-row>
    </el-card>

    <!-- 创建/编辑角色对话框 -->
    <el-dialog
      v-model="characterDialogVisible"
      :title="editingCharacter ? '编辑角色' : '创建角色'"
      width="600px"
    >
      <el-form
        ref="characterFormRef"
        :model="characterForm"
        label-width="100px"
        class="character-form"
      >
        <el-form-item label="角色名称" prop="name" required>
          <el-input v-model="characterForm.name" placeholder="输入角色名称" />
        </el-form-item>
        <el-form-item label="别名">
          <el-input v-model="characterForm.aliases" placeholder="输入别名，多个用逗号分隔" />
        </el-form-item>
        <el-form-item label="性别">
          <el-select v-model="characterForm.gender" clearable placeholder="选择性别">
            <el-option label="男" value="male" />
            <el-option label="女" value="female" />
            <el-option label="其他" value="other" />
          </el-select>
        </el-form-item>
        <el-form-item label="角色定位">
          <el-select v-model="characterForm.role" clearable placeholder="选择角色定位">
            <el-option label="主角" value="protagonist" />
            <el-option label="配角" value="supporting" />
            <el-option label="反派" value="antagonist" />
            <el-option label="客串" value="guest" />
          </el-select>
        </el-form-item>
        <el-form-item label="外貌特征">
          <el-input
            v-model="characterForm.appearance"
            type="textarea"
            :rows="2"
            placeholder="描述角色的外貌特征"
          />
        </el-form-item>
        <el-form-item label="性格特点">
          <el-input
            v-model="characterForm.personality"
            type="textarea"
            :rows="2"
            placeholder="描述角色的性格特点"
          />
        </el-form-item>
        <el-form-item label="人物描述">
          <el-input
            v-model="characterForm.description"
            type="textarea"
            :rows="3"
            placeholder="角色的详细描述"
          />
        </el-form-item>
        <el-form-item label="标签">
          <el-input
            v-model="characterForm.tags"
            placeholder="输入标签，多个用逗号分隔"
          />
        </el-form-item>
        <el-form-item label="绑定音色">
          <el-select
            v-model="characterForm.voice_id"
            clearable
            placeholder="选择绑定的音色（用于音频生成）"
            filterable
            style="width: 100%;"
          >
            <el-option-group
              v-for="category in voiceCategories"
              :key="category.label"
              :label="category.label"
            >
              <el-option
                v-for="voice in category.options"
                :key="voice.id"
                :label="voice.name"
                :value="voice.id"
              />
            </el-option-group>
          </el-select>
        </el-form-item>
      </el-form>
      <template #footer>
        <el-button @click="characterDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleSaveCharacter" :loading="saving">
          保存
        </el-button>
      </template>
    </el-dialog>

    <!-- 角色详情对话框 -->
    <el-dialog
      v-model="detailDialogVisible"
      title="角色详情"
      width="700px"
    >
      <div v-if="selectedCharacter" class="character-detail">
        <div class="detail-header">
          <div
            class="detail-avatar"
            :style="{
              background: getAvatarColor(selectedCharacter),
            }"
          >
            <el-image
              v-if="selectedCharacter.defaultImage"
              :src="selectedCharacter.defaultImage"
              class="image"
              fit="cover"
              :preview-src-list="[selectedCharacter.defaultImage]"
              preview-teleported
              style="cursor: pointer;"
            />
            <span v-else>{{ selectedCharacter.name.charAt(0) }}</span>
          </div>
          <div class="detail-info">
            <h2>{{ selectedCharacter.name }}</h2>
            <p v-if="selectedCharacter.aliases">别名：{{ selectedCharacter.aliases }}</p>
            <p>
              <el-tag v-if="selectedCharacter.gender" size="small" style="margin-right: 8px;">
                {{ getGenderLabel(selectedCharacter.gender) }}
              </el-tag>
              <el-tag v-if="selectedCharacter.role" size="small" type="success">
                {{ getRoleLabel(selectedCharacter.role) }}
              </el-tag>
            </p>
          </div>
        </div>

        <el-descriptions :column="2" border class="detail-desc">
          <el-descriptions-item label="外貌特征" v-if="selectedCharacter.appearance">
            {{ selectedCharacter.appearance }}
          </el-descriptions-item>
          <el-descriptions-item label="性格特点" v-if="selectedCharacter.personality">
            {{ selectedCharacter.personality }}
          </el-descriptions-item>
          <el-descriptions-item label="人物描述" :span="2" v-if="selectedCharacter.description">
            {{ selectedCharacter.description }}
          </el-descriptions-item>
          <el-descriptions-item label="标签" v-if="selectedCharacter.tags">
            <el-tag v-for="tag in selectedCharacter.tags.split(',')" :key="tag" size="small" style="margin-right: 4px;">
              {{ tag.trim() }}
            </el-tag>
          </el-descriptions-item>
        </el-descriptions>

        <!-- 角色形象画廊 -->
        <div v-if="characterImages.length > 0" class="image-gallery-section">
          <h3>角色形象画廊</h3>
          <el-row :gutter="10">
            <el-col
              v-for="image in characterImages"
              :key="image.id"
              :xs="8"
              :sm="6"
              :md="4"
            >
              <el-card class="gallery-image-card" shadow="hover">
                <div class="gallery-image-wrapper">
                  <el-image
                    :src="image.image_path"
                    class="gallery-image"
                    fit="cover"
                    :preview-src-list="characterImages.map(i => i.image_path)"
                    :initial-index="characterImages.indexOf(image)"
                    preview-teleported
                  />
                  <div class="image-actions">
                    <el-button
                      size="small"
                      :type="image.is_default ? 'success' : 'default'"
                      @click="handleSetDefaultImage(selectedCharacter.id, image)"
                      style="margin-bottom: 4px;"
                    >
                      {{ image.is_default ? '默认' : '设为默认' }}
                    </el-button>
                    <el-popconfirm
                      title="确定要删除这张图片吗？"
                      @confirm="handleDeleteImage(image.id)"
                    >
                      <template #reference>
                        <el-button size="small" type="danger" style="width: 100%">删除</el-button>
                      </template>
                    </el-popconfirm>
                  </div>
                </div>
                <div class="image-meta" v-if="image.pose || image.expression">
                  <span v-if="image.pose">姿态: {{ image.pose }}</span>
                  <span v-if="image.expression">表情: {{ image.expression }}</span>
                </div>
              </el-card>
            </el-col>
          </el-row>
        </div>
      </div>
      <template #footer>
        <el-button @click="detailDialogVisible = false">关闭</el-button>
        <el-button type="primary" @click="handleGenerateImage(selectedCharacter)">
          生成新形象
        </el-button>
      </template>
    </el-dialog>

    <!-- 生成角色形象对话框 -->
    <el-dialog
      v-model="generateImageDialogVisible"
      title="AI 生成角色形象"
      width="600px"
    >
      <div class="generate-image-form">
        <el-alert
          title="提示"
          description="AI 将根据角色的描述信息自动生成形象，您也可以自定义提示词进行调整。"
          type="info"
          :closable="false"
          style="margin-bottom: 20px;"
        />
        <el-form label-width="100px">
          <el-form-item label="生成风格">
            <el-select v-model="generateForm.style" style="width: 100%;">
              <el-option label="通用" value="general" />
              <el-option label="动漫风格" value="anime" />
              <el-option label="写实照片" value="photorealistic" />
              <el-option label="油画风格" value="painting" />
            </el-select>
          </el-form-item>
          <el-form-item label="图像尺寸">
            <el-select v-model="generateForm.size" style="width: 100%;">
              <el-option label="1024x1024 (方形)" value="1024x1024" />
              <el-option label="768x1024 (竖版)" value="768x1024" />
              <el-option label="1024x768 (横版)" value="1024x768" />
            </el-select>
          </el-form-item>
          <el-form-item label="姿态">
            <el-select v-model="generateForm.pose" clearable placeholder="选择姿态（可选）" style="width: 100%;">
              <el-option label="正面半身像" value="正面半身像" />
              <el-option label="全身像" value="全身像" />
              <el-option label="侧面像" value="侧面像" />
              <el-option label="战斗姿态" value="战斗姿态" />
              <el-option label="优雅坐姿" value="优雅坐姿" />
            </el-select>
          </el-form-item>
          <el-form-item label="表情">
            <el-select v-model="generateForm.expression" clearable placeholder="选择表情（可选）" style="width: 100%;">
              <el-option label="微笑" value="微笑" />
              <el-option label="严肃" value="严肃" />
              <el-option label="愤怒" value="愤怒" />
              <el-option label="忧郁" value="忧郁" />
              <el-option label="自信" value="自信" />
            </el-select>
          </el-form-item>
          <el-form-item label="自定义提示词">
            <el-input
              v-model="generateForm.prompt"
              type="textarea"
              :rows="3"
              placeholder="输入自定义提示词，将与角色信息合并用于生成"
            />
            <div class="form-tip">
              角色信息将自动用于生成，此处可输入额外的风格或细节要求
            </div>
          </el-form-item>
        </el-form>
      </div>
      <template #footer>
        <el-button @click="generateImageDialogVisible = false">取消</el-button>
        <el-button type="primary" @click="handleConfirmGenerateImage" :loading="generating">
          开始生成
        </el-button>
      </template>
    </el-dialog>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, computed, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage, ElMessageBox } from 'element-plus'
import { Plus, MagicStick, VideoPlay, Microphone } from '@element-plus/icons-vue'

interface Character {
  id: number
  novel_id: number
  name: string
  aliases?: string
  gender?: string
  role?: string
  description?: string
  appearance?: string
  personality?: string
  voice_id?: string
  tags?: string
  created_at: string
  defaultImage?: string
}

interface CharacterImage {
  id: string
  character_id: number
  image_type: string
  pose?: string
  expression?: string
  image_path: string
  prompt?: string
  seed?: number
  is_default: boolean
  created_at: string
}

// 数据
const novels = ref<Array<{ id: number; title: string }>>([])
const characters = ref<Character[]>([])
const loading = ref(false)
const saving = ref(false)
const extracting = ref(false)
const generating = ref(false)

// 筛选
const filterForm = reactive({
  novelId: null as number | null,
  keyword: '',
})

const filteredCharacters = computed(() => {
  if (!filterForm.keyword) {
    return characters.value
  }
  const keyword = filterForm.keyword.toLowerCase()
  return characters.value.filter(
    (c) =>
      c.name.toLowerCase().includes(keyword) ||
      (c.aliases && c.aliases.toLowerCase().includes(keyword))
  )
})

// 对话框
const characterDialogVisible = ref(false)
const detailDialogVisible = ref(false)
const generateImageDialogVisible = ref(false)
const editingCharacter = ref<Character | null>(null)
const selectedCharacter = ref<Character | null>(null)
const characterImages = ref<CharacterImage[]>([])

// 表单
const characterForm = reactive({
  name: '',
  aliases: '',
  gender: '',
  role: '',
  description: '',
  appearance: '',
  personality: '',
  tags: '',
  voice_id: '',
})

// 音色映射 - MiniMax 官方音色列表
// 按分类的音色列表（含中文普通话、粤语等）
const voiceCategories = ref<any[]>([])

// 加载音色列表
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

// 快速查找音色名称
const getVoiceName = (voiceId?: string) => {
  if (!voiceId) return ''
  for (const cat of voiceCategories.value) {
    const found = cat.options.find((opt: any) => opt.id === voiceId)
    if (found) return found.name
  }
  return voiceId
}

const generateForm = reactive({
  style: 'anime',
  size: '768x1024',
  pose: '',
  expression: '',
  prompt: '',
})

// 方法
const getAvatarColor = (character: Character) => {
  const colors = [
    'linear-gradient(135deg, #667eea 0%, #764ba2 100%)',
    'linear-gradient(135deg, #f093fb 0%, #f5576c 100%)',
    'linear-gradient(135deg, #4facfe 0%, #00f2fe 100%)',
    'linear-gradient(135deg, #43e97b 0%, #38f9d7 100%)',
    'linear-gradient(135deg, #fa709a 0%, #fee140 100%)',
  ]
  const index = character.id % colors.length
  return colors[index]
}

const getGenderLabel = (gender: string) => {
  const map: Record<string, string> = {
    male: '男',
    female: '女',
    other: '其他',
  }
  return map[gender] || gender
}

const getRoleLabel = (role: string) => {
  const map: Record<string, string> = {
    protagonist: '主角',
    supporting: '配角',
    antagonist: '反派',
    guest: '客串',
  }
  return map[role] || role
}

const loadNovels = async () => {
  try {
    const result = await invoke('get_novels')
    novels.value = result as Array<{ id: number; title: string }>
  } catch (e) {
    console.error('加载小说列表失败', e)
  }
}

const loadCharacters = async () => {
  if (!filterForm.novelId) {
    characters.value = []
    return
  }

  loading.value = true
  try {
    const chars = (await invoke('get_characters', {
      novel_id: filterForm.novelId,
      novelId: filterForm.novelId,
    })) as Character[]

    // 加载每个角色的默认图片
    for (const char of chars) {
      try {
        const images = (await invoke('get_character_images', {
          character_id: char.id,
          characterId: char.id,
        })) as CharacterImage[]
        const defaultImage = images.find((img) => img.is_default)
        if (defaultImage) {
          char.defaultImage = defaultImage.image_path
        }
      } catch (e) {
        console.error(`加载角色 ${char.id} 图片失败`, e)
      }
    }

    characters.value = chars
  } catch (e) {
    ElMessage.error(`加载角色列表失败: ${e}`)
  } finally {
    loading.value = false
  }
}

const handleSearch = () => {
  // 搜索由 computed 属性处理
}

const handleCreate = () => {
  editingCharacter.value = null
  Object.assign(characterForm, {
    name: '',
    aliases: '',
    gender: '',
    role: '',
    description: '',
    appearance: '',
    personality: '',
    tags: '',
  })
  characterDialogVisible.value = true
}

const handleEditCharacter = (character: Character) => {
  editingCharacter.value = character
  Object.assign(characterForm, {
    name: character.name,
    aliases: character.aliases || '',
    gender: character.gender || '',
    role: character.role || '',
    description: character.description || '',
    appearance: character.appearance || '',
    personality: character.personality || '',
    tags: character.tags || '',
    voice_id: character.voice_id || '',
  })
  characterDialogVisible.value = true
}

const handleSaveCharacter = async () => {
  if (!characterForm.name.trim()) {
    ElMessage.warning('请输入角色名称')
    return
  }

  saving.value = true
  try {
    if (editingCharacter.value) {
      // 更新
      await invoke('update_character', {
        request: {
          id: editingCharacter.value.id,
          voiceId: characterForm.voice_id,
          ...characterForm,
        },
      })
      ElMessage.success('角色更新成功')
    } else {
      // 创建
      await invoke('create_character', {
        request: {
          novelId: filterForm.novelId,
          ...characterForm,
        },
      })
      ElMessage.success('角色创建成功')
    }
    characterDialogVisible.value = false
    await loadCharacters()
  } catch (e) {
    ElMessage.error(`保存失败: ${e}`)
  } finally {
    saving.value = false
  }
}

const handleDeleteCharacter = async (character: Character) => {
  try {
    await ElMessageBox.confirm(
      `确定要删除角色「${character.name}」吗？此操作不可恢复！`,
      '确认删除',
      {
        type: 'warning',
        confirmButtonText: '删除',
        cancelButtonText: '取消',
      }
    )

    await invoke('delete_character', {
      characterId: character.id,
    })
    ElMessage.success('删除成功')
    await loadCharacters()
  } catch (e) {
    if (e !== 'cancel') {
      ElMessage.error(`删除失败: ${e}`)
    }
  }
}

const handleViewCharacter = async (character: Character) => {
  selectedCharacter.value = character
  try {
    characterImages.value = (await invoke('get_character_images', {
      character_id: character.id,
      characterId: character.id,
    })) as CharacterImage[]
  } catch (e) {
    console.error('加载角色图片失败', e)
    characterImages.value = []
  }
  detailDialogVisible.value = true
}

const handleGenerateImage = (character: Character | null) => {
  if (!character) return
  
  selectedCharacter.value = character
  Object.assign(generateForm, {
    style: 'anime',
    size: '768x1024',
    pose: '',
    expression: '',
    prompt: '',
  })
  generateImageDialogVisible.value = true
}

const handleConfirmGenerateImage = async () => {
  if (!selectedCharacter.value) return

  generating.value = true
  try {
    const result = (await invoke('generate_character_image', {
      request: {
        character_id: selectedCharacter.value.id,
        characterId: selectedCharacter.value.id,
        prompt: generateForm.prompt || null,
        style: generateForm.style,
        size: generateForm.size,
        pose: generateForm.pose || null,
        expression: generateForm.expression || null,
      },
    })) as { id: string; url: string; prompt: string }

    ElMessage.success('形象生成成功！')
    generateImageDialogVisible.value = false

    // 如果是在详情页打开的，刷新图片列表
    if (detailDialogVisible.value) {
      await handleViewCharacter(selectedCharacter.value)
    } else {
      // 刷新列表以显示新头像
      await loadCharacters()
    }
  } catch (e) {
    ElMessage.error(`生成失败: ${e}`)
  } finally {
    generating.value = false
  }
}

const handleSetDefaultImage = async (characterId: number, image: CharacterImage) => {
  if (image.is_default) return

  try {
    await invoke('set_default_character_image', {
      character_id: characterId,
      characterId: characterId,
      image_id: image.id,
      imageId: image.id,
    })
    ElMessage.success('已设为默认形象')
    // 刷新
    // 刷新
    await handleViewCharacter(selectedCharacter.value!)
    await loadCharacters()
  } catch (e) {
    ElMessage.error(`操作失败: ${e}`)
  }
}

const handleDeleteImage = async (imageId: string) => {
  try {
    await invoke('delete_character_image', {
      image_id: imageId,
    })
    ElMessage.success('删除成功')
    // 刷新
    await handleViewCharacter(selectedCharacter.value!)
    await loadCharacters()
  } catch (e) {
    ElMessage.error(`删除失败: ${e}`)
  }
}

const handleExtractCharacters = async () => {
  if (!filterForm.novelId) {
    ElMessage.warning('请先选择小说')
    return
  }

  extracting.value = true
  try {
    const extracted = (await invoke('extract_characters_from_content', {
      novelId: filterForm.novelId,
    })) as Character[]

    if (extracted.length === 0) {
      ElMessage.info('未能从小说内容中提取到角色，请确保小说已有章节内容')
    } else {
      ElMessage.success(`成功提取 ${extracted.length} 个角色`)
      await loadCharacters()
    }
  } catch (e) {
    ElMessage.error(`提取失败: ${e}`)
  } finally {
    extracting.value = false
  }
}

onMounted(async () => {
  await Promise.all([
    loadNovels(),
    loadVoiceCategories()
  ])
  // 如果小说列表只有一个，自动选中
  if (novels.value.length === 1) {
    filterForm.novelId = novels.value[0].id
    await loadCharacters()
  }
})
</script>

<style scoped>
.characters-page {
  padding: 0;
}

.character-card {
  text-align: center;
  margin-bottom: 20px;
}

.character-avatar {
  margin-bottom: 16px;
}

.avatar-image {
  width: 80px;
  height: 80px;
  margin: 0 auto;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 32px;
  font-weight: bold;
  color: white;
  overflow: hidden;
}

.avatar-image .image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.character-info {
  margin-bottom: 12px;
}

.character-name {
  margin: 0 0 8px 0;
  font-size: 16px;
  color: #303133;
}

.character-meta {
  margin: 0 0 8px 0;
  font-size: 12px;
  color: #909399;
}

.character-meta span + span::before {
  content: '·';
  margin: 0 4px;
}

.role-tag {
  background: #e1f3d8;
  color: #67c23a;
  padding: 0 6px;
  border-radius: 3px;
  font-size: 12px;
}

.meta-tag {
  background: #f0f9eb;
  color: #95d475;
  padding: 0 6px;
  border-radius: 3px;
  font-size: 12px;
}

.voice-info {
  margin-bottom: 8px;
  display: inline-flex;
  align-items: center;
  padding: 4px 10px;
  background: #f0f9eb;
  border-radius: 4px;
  border: 1px solid #e1f3d8;
}

.voice-info.voice-info-empty {
  background: #f4f4f5;
  border-color: #e4e7ed;
  opacity: 0.7;
}

.voice-info .voice-icon {
  display: flex;
  align-items: center;
  margin-right: 6px;
  color: #67c23a;
}

.voice-info.voice-info-empty .voice-icon {
  color: #909399;
}

.voice-info .voice-name {
  font-size: 13px;
  color: #606266;
}

.voice-info.voice-info-empty .voice-name {
  color: #909399;
  font-style: italic;
}

.character-desc {
  margin: 0;
  font-size: 12px;
  color: #606266;
  line-height: 1.4;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.character-actions {
  display: flex;
  justify-content: center;
  gap: 8px;
  flex-wrap: wrap;
}

/* 详情对话框 */
.character-detail .detail-header {
  display: flex;
  align-items: center;
  margin-bottom: 24px;
}

.detail-avatar {
  width: 100px;
  height: 100px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 40px;
  font-weight: bold;
  color: white;
  margin-right: 24px;
  overflow: hidden;
}

.detail-avatar .image {
  width: 100%;
  height: 100%;
  object-fit: cover;
}

.detail-info h2 {
  margin: 0 0 8px 0;
  font-size: 24px;
  color: #303133;
}

.detail-info p {
  margin: 4px 0;
  color: #606266;
}

.detail-desc {
  margin-bottom: 24px;
}

.image-gallery-section h3 {
  margin: 0 0 16px 0;
  font-size: 16px;
  color: #303133;
}

.gallery-image-card {
  margin-bottom: 10px;
}

.gallery-image-wrapper {
  position: relative;
  width: 100%;
  padding-top: 150%; /* 调整图片比例，更高一些 */
  overflow: hidden;
}

.gallery-image {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  object-fit: cover;
  cursor: pointer;
}

.image-actions {
  position: absolute;
  left: 0;
  right: 0;
  bottom: 0;
  height: auto;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: row;
  align-items: center;
  justify-content: center;
  padding: 10px 8px;
  gap: 8px;
  opacity: 0;
  transition: opacity 0.2s;
}

.gallery-image-wrapper:hover .image-actions {
  opacity: 1;
}

.image-meta {
  padding: 8px;
  font-size: 12px;
  color: #909399;
  display: flex;
  justify-content: space-between;
}

.generate-image-form .form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}
</style>
