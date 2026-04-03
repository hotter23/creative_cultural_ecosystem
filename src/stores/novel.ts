import { defineStore } from 'pinia'
import { ref } from 'vue'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'

export interface Novel {
  id: number
  title: string
  description?: string
  cover_path?: string
  status: string
  current_stage: string
  total_chapters: number
  total_words: number
  created_at: string
  updated_at: string
}

export interface Chapter {
  id: number
  novel_id: number
  title: string
  content?: string
  plain_text?: string
  order_num: number
  word_count: number
  status: string
  created_at: string
  audio_status?: string
}

export const useNovelStore = defineStore('novel', () => {
  const novels = ref<Novel[]>([])
  const currentNovel = ref<Novel | null>(null)
  const chapters = ref<Chapter[]>([])
  const currentChapter = ref<Chapter | null>(null)
  const loading = ref(false)

  // 获取所有网文
  const fetchNovels = async () => {
    loading.value = true
    try {
      novels.value = await invoke<Novel[]>('get_novels')
    } catch (error) {
      console.error('获取网文列表失败:', error)
    } finally {
      loading.value = false
    }
  }

  // 获取单个网文
  const fetchNovel = async (id: number) => {
    loading.value = true
    try {
      currentNovel.value = await invoke<Novel>('get_novel', { id })
    } catch (error) {
      console.error('获取网文详情失败:', error)
    } finally {
      loading.value = false
    }
  }

  // 创建网文
  const createNovel = async (data: { title: string; description?: string }) => {
    loading.value = true
    try {
      const novel = await invoke<Novel>('create_novel', { data })
      novels.value.unshift(novel)
      return novel
    } catch (error) {
      console.error('创建网文失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  // 更新网文
  const updateNovel = async (id: number, data: any) => {
    loading.value = true
    try {
      await invoke('update_novel', { id, data })
      if (currentNovel.value?.id === id) {
        currentNovel.value = { ...currentNovel.value, ...data }
      }
      const index = novels.value.findIndex(n => n.id === id)
      if (index !== -1) {
        novels.value[index] = { ...novels.value[index], ...data }
      }
    } catch (error) {
      console.error('更新网文失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  // 删除网文
  const deleteNovel = async (id: number) => {
    loading.value = true
    try {
      await invoke('delete_novel', { id })
      novels.value = novels.value.filter(n => n.id !== id)
      if (currentNovel.value?.id === id) {
        currentNovel.value = null
      }
    } catch (error) {
      console.error('删除网文失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  // 获取章节列表
  const fetchChapters = async (novelId: number) => {
    loading.value = true
    try {
      chapters.value = await invoke<Chapter[]>('get_chapters', { novelId })
    } catch (error) {
      console.error('获取章节列表失败:', error)
    } finally {
      loading.value = false
    }
  }

  // 获取单个章节
  const fetchChapter = async (id: number) => {
    loading.value = true
    try {
      currentChapter.value = await invoke<Chapter>('get_chapter', { id })
      return currentChapter.value
    } catch (error) {
      console.error('获取章节详情失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  // 创建章节
  const createChapter = async (data: { novel_id: number; title: string; content?: string; order_num: number }) => {
    loading.value = true
    try {
      const chapter = await invoke<Chapter>('create_chapter', { data })
      chapters.value.push(chapter)
      chapters.value.sort((a, b) => a.order_num - b.order_num)
      return chapter
    } catch (error) {
      console.error('创建章节失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  // 更新章节
  const updateChapter = async (id: number, data: any) => {
    loading.value = true
    try {
      await invoke('update_chapter', { id, data })
      if (currentChapter.value?.id === id) {
        currentChapter.value = { ...currentChapter.value, ...data }
      }
      const index = chapters.value.findIndex(c => c.id === id)
      if (index !== -1) {
        chapters.value[index] = { ...chapters.value[index], ...data }
      }
    } catch (error) {
      console.error('更新章节失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  // 删除章节
  const deleteChapter = async (id: number) => {
    loading.value = true
    try {
      await invoke('delete_chapter', { id })
      chapters.value = chapters.value.filter(c => c.id !== id)
      if (currentChapter.value?.id === id) {
        currentChapter.value = null
      }
    } catch (error) {
      console.error('删除章节失败:', error)
      throw error
    } finally {
      loading.value = false
    }
  }

  return {
    novels,
    currentNovel,
    chapters,
    currentChapter,
    loading,
    fetchNovels,
    fetchNovel,
    createNovel,
    updateNovel,
    deleteNovel,
    fetchChapters,
    fetchChapter,
    createChapter,
    updateChapter,
    deleteChapter
  }
})
