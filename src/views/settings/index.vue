<template>
  <div class="settings-page">
    <el-card shadow="never">
      <template #header>
        <span>系统设置</span>
      </template>
      
      <el-tabs v-model="activeTab" type="border-card">
        <!-- MinMax TokenPlan 配置 -->
        <el-tab-pane label="MinMax AI" name="minimax">
          <el-form :model="minimaxForm" label-width="120px" style="max-width: 600px;">
            <el-form-item label="启用服务">
              <el-switch v-model="minimaxForm.enabled" />
            </el-form-item>
            
            <el-form-item label="API Key">
              <el-input 
                v-model="minimaxForm.api_key" 
                type="password" 
                placeholder="请输入 MinMax API Key"
                show-password
                :disabled="!minimaxForm.enabled"
              />
              <div class="form-tip">
                获取地址：<a href="https://platform.minimaxi.com" target="_blank">
                  https://platform.minimaxi.com
                </a>
              </div>
            </el-form-item>
            
            <el-form-item label="Group ID">
              <el-input 
                v-model="minimaxForm.group_id" 
                placeholder="可选，企业用户需要"
                :disabled="!minimaxForm.enabled"
              />
            </el-form-item>
            
            <el-form-item label="API 地址">
              <el-input 
                v-model="minimaxForm.base_url" 
                placeholder="请输入 API 地址"
                :disabled="!minimaxForm.enabled"
              />
            </el-form-item>
            
            <el-form-item label="默认模型">
              <el-select 
                v-model="minimaxForm.default_model" 
                :disabled="!minimaxForm.enabled"
              >
                <el-option label="MiniMax M2.7" value="minimaxi-2.7" />
                <el-option label="MiniMax M2.7 Pro" value="minimaxi-2.7-pro" />
              </el-select>
            </el-form-item>
            
            <el-form-item>
              <el-button 
                type="primary" 
                @click="saveMinimaxConfig" 
                :loading="saving"
                :disabled="!minimaxForm.enabled"
              >
                保存配置
              </el-button>
              <el-button 
                @click="testMinimaxConnection" 
                :loading="testing"
                :disabled="!minimaxForm.enabled || !minimaxForm.api_key"
              >
                测试连接
              </el-button>
            </el-form-item>
          </el-form>
          
          <!-- Token 使用统计 -->
          <div v-if="tokenUsage && minimaxForm.enabled" class="token-usage">
            <el-divider content-position="left">使用统计</el-divider>
            <el-descriptions :column="2" border size="small">
              <el-descriptions-item label="今日使用">
                {{ tokenUsage.today || 0 }} tokens
              </el-descriptions-item>
              <el-descriptions-item label="本月使用">
                {{ tokenUsage.month || 0 }} tokens
              </el-descriptions-item>
            </el-descriptions>
          </div>
        </el-tab-pane>
        
        <!-- TTS 语音配置 -->
        <el-tab-pane label="语音配置" name="tts">
          <el-form :model="ttsForm" label-width="120px" style="max-width: 600px;">
            <el-form-item label="默认音色">
              <el-select v-model="ttsForm.default_voice">
                <el-option label="女声-亲和" value="voice_female_01" />
                <el-option label="男声-沉稳" value="voice_male_01" />
                <el-option label="女声-甜美" value="voice_female_02" />
                <el-option label="男声-磁性" value="voice_male_02" />
              </el-select>
            </el-form-item>
            
            <el-form-item label="语速">
              <el-slider 
                v-model="ttsForm.speed" 
                :min="0.5" 
                :max="2" 
                :step="0.1"
                show-input
              />
            </el-form-item>
            
            <el-form-item>
              <el-button type="primary" @click="saveTTSConfig" :loading="savingTTS">
                保存配置
              </el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>

        <!-- MCP 服务配置 -->
        <el-tab-pane label="MCP 服务" name="mcp">
          <el-form :model="mcpForm" label-width="140px" style="max-width: 600px;">
            <el-form-item label="启用 MCP 服务">
              <el-switch v-model="mcpForm.enabled" />
            </el-form-item>
            
            <el-form-item label="开机自动启动">
              <el-switch v-model="mcpForm.auto_start" :disabled="!mcpForm.enabled" />
            </el-form-item>
            
            <el-form-item label="绑定地址">
              <el-select 
                v-model="mcpForm.bind_address" 
                :disabled="!mcpForm.enabled"
                style="width: 200px;"
              >
                <el-option label="仅本地 (127.0.0.1)" value="127.0.0.1" />
                <el-option label="所有网卡 (0.0.0.0)" value="0.0.0.0" />
              </el-select>
              <div class="form-tip">
                选择服务监听的网络接口，0.0.0.0 允许局域网内其他机器访问
              </div>
            </el-form-item>
            
            <el-form-item label="服务端口">
              <el-input-number 
                v-model="mcpForm.port" 
                :min="1024" 
                :max="65535" 
                :disabled="!mcpForm.enabled"
                style="width: 200px;"
              />
              <div class="form-tip">
                Agent 通过此端口连接 MCP 服务
              </div>
            </el-form-item>
            
            <el-form-item label="服务状态">
              <el-tag :type="mcpStatus.http_status === 'running' ? 'success' : 'info'">
                {{ mcpStatus.http_status === 'running' ? '运行中' : '已停止' }}
              </el-tag>
              <div v-if="mcpStatus.http_status === 'running'" class="status-info">
                <p>绑定地址：<code>{{ mcpForm.bind_address }}:{{ mcpForm.port }}</code></p>
                <p>服务地址：<code>http://{{ mcpForm.bind_address === '0.0.0.0' ? 'localhost' : mcpForm.bind_address }}:{{ mcpForm.port }}/mcp</code></p>
                <p>发现端点：<code>http://{{ mcpForm.bind_address === '0.0.0.0' ? 'localhost' : mcpForm.bind_address }}:{{ mcpForm.port }}/.well-known/mcp</code></p>
              </div>
            </el-form-item>
            
            <el-form-item label="可用工具">
              <el-tag v-if="mcpStatus.tool_count" type="success">
                {{ mcpStatus.tool_count }} 个工具
              </el-tag>
              <el-tag v-else type="info">
                服务未启动
              </el-tag>
            </el-form-item>
            
            <el-form-item>
              <el-button 
                type="primary" 
                @click="saveMCPConfig" 
                :loading="savingMCP"
                :disabled="!mcpForm.enabled"
              >
                保存配置
              </el-button>
              <el-button 
                :type="mcpStatus.http_status === 'running' ? 'danger' : 'success'"
                @click="toggleMcpServer"
                :loading="togglingMcp"
                :disabled="!mcpForm.enabled"
              >
                {{ mcpStatus.http_status === 'running' ? '停止服务' : '启动服务' }}
              </el-button>
              <el-button 
                @click="registerMcpService"
                :disabled="mcpStatus.http_status !== 'running'"
              >
                注册到 Agent
              </el-button>
            </el-form-item>
          </el-form>
          
          <!-- MCP 服务说明 -->
          <div class="mcp-info" v-if="mcpForm.enabled">
            <el-divider content-position="left">服务说明</el-divider>
            <el-alert title="MCP (Model Control Protocol)" type="info" class="mcp-alert">
              <template #default>
                <p>MCP 服务允许本地 Agent 通过标准协议调用本平台的功能，包括：</p>
                <ul>
                  <li>• 网文创作与管理（创建、读取、更新、删除小说）</li>
                  <li>• 章节内容管理（支持批量操作）</li>
                  <li>• AI 内容生成（续写、润色、摘要、角色提取等）</li>
                  <li>• 文本转语音（TTS）</li>
                </ul>
                <p class="mt-2">Agent 可通过以下方式发现本服务：</p>
                <ol>
                  <li>1. 访问服务发现端点 <code>/.well-known/mcp</code></li>
                  <li>2. 或扫描本地 MCP 服务配置目录</li>
                </ol>
              </template>
            </el-alert>
            
            <!-- 可用工具列表 -->
            <div v-if="mcpTools.length > 0" class="tool-list">
              <el-divider content-position="left">可用工具列表</el-divider>
              <el-table :data="mcpTools" size="small" border>
                <el-table-column prop="name" label="工具名称" width="200" />
                <el-table-column prop="description" label="描述" />
              </el-table>
            </div>
          </div>
        </el-tab-pane>
        
        <!-- 视频配置 -->
        <el-tab-pane label="视频配置" name="video">
          <el-form :model="videoForm" label-width="120px" style="max-width: 600px;">
            <el-form-item label="默认分辨率">
              <el-select v-model="videoForm.resolution">
                <el-option label="1080p (1920×1080)" value="1080p" />
                <el-option label="2K (2560×1440)" value="2k" />
                <el-option label="4K (3840×2160)" value="4k" />
              </el-select>
            </el-form-item>
            <el-form-item label="默认帧率">
              <el-select v-model="videoForm.fps">
                <el-option label="24 FPS" :value="24" />
                <el-option label="30 FPS" :value="30" />
                <el-option label="60 FPS" :value="60" />
              </el-select>
            </el-form-item>
            <el-form-item label="FFmpeg 路径">
              <el-input v-model="videoForm.ffmpegPath" placeholder="请输入 FFmpeg 可执行文件路径">
                <template #append>
                  <el-button>浏览</el-button>
                </template>
              </el-input>
            </el-form-item>
            <el-form-item>
              <el-button type="primary" @click="saveVideoConfig" :loading="savingVideo">保存配置</el-button>
            </el-form-item>
          </el-form>
        </el-tab-pane>
        
        <!-- 关于 -->
        <el-tab-pane label="关于" name="about">
          <div class="about-content">
            <div class="logo">
              <h2>AI 内容创作平台</h2>
            </div>
            <div class="version-info">
              <p>版本号：1.0.0</p>
              <p>发布日期：2024-01-01</p>
              <p>技术栈：Vue 3 + Tauri + SQLite</p>
              <p>AI 支持：MinMax TokenPlan</p>
            </div>
            <div class="actions">
              <el-button>检查更新</el-button>
              <el-button>用户手册</el-button>
              <el-button>反馈建议</el-button>
            </div>
          </div>
        </el-tab-pane>
      </el-tabs>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { ElMessage } from 'element-plus'

const activeTab = ref('minimax')
const saving = ref(false)
const testing = ref(false)
const savingTTS = ref(false)
const savingMCP = ref(false)
const savingVideo = ref(false)
const togglingMcp = ref(false)

// MinMax 配置表单
const minimaxForm = reactive({
  enabled: false,
  api_key: '',
  base_url: 'https://api.minimaxi.com',
  default_model: 'minimaxi-2.7',
  group_id: ''
})

// TTS 配置表单
const ttsForm = reactive({
  default_voice: 'voice_female_01',
  speed: 1.0
})

const videoForm = reactive({
  resolution: '1080p',
  fps: 30,
  ffmpegPath: ''
})

// MCP 配置表单
const mcpForm = reactive({
  enabled: true,
  auto_start: false,
  port: 8787,
  bind_address: '127.0.0.1'
})

// MCP 服务状态
const mcpStatus = reactive({
  status: 'stopped',
  http_status: 'stopped',
  tool_count: 0,
  version: '',
  enabled: true,
  http_endpoint: '',
  sse_endpoint: '',
  port: null as number | null
})

// MCP 可用工具列表
const mcpTools = ref<Array<{name: string, description: string}>>([])

// Token 使用统计
const tokenUsage = ref<{
  today?: number
  month?: number
} | null>(null)

// 保存 MinMax 配置
const saveMinimaxConfig = async () => {
  if (!minimaxForm.api_key && minimaxForm.enabled) {
    ElMessage.warning('请输入 API Key')
    return
  }
  
  saving.value = true
  try {
    await invoke('save_minimax_config', {
      enabled: minimaxForm.enabled,
      apiKey: minimaxForm.api_key,
      baseUrl: minimaxForm.base_url,
      defaultModel: minimaxForm.default_model,
      groupId: minimaxForm.group_id || undefined
    })
    ElMessage.success('配置已保存')
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  } finally {
    saving.value = false
  }
}

// 测试 MinMax 连接
const testMinimaxConnection = async () => {
  if (!minimaxForm.api_key) {
    ElMessage.warning('请先输入 API Key')
    return
  }
  
  testing.value = true
  try {
    const result = await invoke<boolean>('test_minimax_connection', {
      apiKey: minimaxForm.api_key,
      baseUrl: minimaxForm.base_url || undefined,
      groupId: minimaxForm.group_id || undefined
    })
    
    if (result) {
      ElMessage.success('连接成功！MinMax TokenPlan 服务正常')
    }
  } catch (error) {
    ElMessage.error(`连接失败: ${error}`)
  } finally {
    testing.value = false
  }
}

// 保存 TTS 配置
const saveTTSConfig = async () => {
  savingTTS.value = true
  try {
    await invoke('save_tts_config', {
      defaultVoice: ttsForm.default_voice,
      speed: ttsForm.speed
    })
    ElMessage.success('配置已保存')
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  } finally {
    savingTTS.value = false
  }
}

// 保存 MCP 配置
const saveMCPConfig = async () => {
  savingMCP.value = true
  try {
    await invoke('mcp_save_config', {
      enabled: mcpForm.enabled,
      autoStart: mcpForm.auto_start,
      port: mcpForm.port,
      bindAddress: mcpForm.bind_address
    })
    ElMessage.success('配置已保存')
    // 重新加载配置和状态
    await loadMCPConfigOnly()
    await loadMCPStatus()
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  } finally {
    savingMCP.value = false
  }
}

// 仅重新加载 MCP 配置（用于保存后刷新）
const loadMCPConfigOnly = async () => {
  try {
    const mcpConfig = await invoke<Record<string, string>>('mcp_get_config')
    mcpForm.enabled = mcpConfig.mcp_enabled !== 'false'
    mcpForm.auto_start = mcpConfig.mcp_auto_start === 'true'
    mcpForm.port = parseInt(mcpConfig.mcp_port || '8787')
    mcpForm.bind_address = mcpConfig.mcp_bind_address || '127.0.0.1'
  } catch (error) {
    console.log('加载 MCP 配置失败:', error)
  }
}

// 切换 MCP 服务器状态
const toggleMcpServer = async () => {
  togglingMcp.value = true
  try {
    if (mcpStatus.http_status === 'running') {
      // 停止服务
      await invoke('mcp_http_server_stop')
      await invoke('mcp_stop')
      ElMessage.success('MCP 服务已停止')
    } else {
      // 启动服务
      await invoke('mcp_start')
      await invoke('mcp_http_server_start', { port: mcpForm.port, bindAddress: mcpForm.bind_address })
      ElMessage.success('MCP 服务已启动')
    }
    await loadMCPStatus()
    await loadMCPTools()
  } catch (error) {
    ElMessage.error(`操作失败: ${error}`)
  } finally {
    togglingMcp.value = false
  }
}

// 注册 MCP 服务到 Agent
const registerMcpService = async () => {
  try {
    await invoke('mcp_register_service', { port: mcpForm.port, bindAddress: mcpForm.bind_address })
    ElMessage.success('MCP 服务已注册到 Agent 发现目录')
  } catch (error) {
    ElMessage.error(`注册失败: ${error}`)
  }
}

// 保存视频配置
const saveVideoConfig = async () => {
  savingVideo.value = true
  try {
    await invoke('save_video_config', {
      resolution: videoForm.resolution,
      fps: videoForm.fps,
      ffmpeg_path: videoForm.ffmpegPath || undefined,
      ffmpegPath: videoForm.ffmpegPath || undefined
    })
    ElMessage.success('视频配置已保存')
  } catch (error) {
    ElMessage.error(`保存失败: ${error}`)
  } finally {
    savingVideo.value = false
  }
}

// 加载 MCP 服务状态
const loadMCPStatus = async () => {
  try {
    const status = await invoke<typeof mcpStatus>('mcp_get_status')
    Object.assign(mcpStatus, status)
    if (status.port) {
      mcpForm.port = status.port
    }
  } catch (error) {
    console.log('加载 MCP 状态失败:', error)
  }
}

// 加载 MCP 工具列表
const loadMCPTools = async () => {
  if (mcpStatus.http_status !== 'running') {
    mcpTools.value = []
    return
  }
  try {
    const tools = await invoke<Array<{name: string, description: string}>>('mcp_list_tools')
    mcpTools.value = tools
  } catch (error) {
    console.log('加载 MCP 工具列表失败:', error)
  }
}

// 加载配置
const loadConfig = async () => {
  try {
    // 加载 MinMax 配置
    const minimaxConfig = await invoke<Record<string, string>>('get_minimax_config')
    
    minimaxForm.enabled = minimaxConfig.minimax_enabled === 'true'
    minimaxForm.api_key = minimaxConfig.minimax_api_key || ''
    minimaxForm.base_url = minimaxConfig.minimax_base_url || 'https://api.minimaxi.com'
    minimaxForm.default_model = minimaxConfig.minimax_default_model || 'minimaxi-2.7'
    minimaxForm.group_id = minimaxConfig.minimax_group_id || ''
    
    // 加载 TTS 配置
    const ttsConfig = await invoke<Record<string, string>>('get_tts_config')
    ttsForm.default_voice = ttsConfig.tts_default_voice || 'voice_female_01'
    ttsForm.speed = parseFloat(ttsConfig.tts_speed || '1.0')
    
    // 加载 MCP 配置
    const mcpConfig = await invoke<Record<string, string>>('mcp_get_config')
    mcpForm.enabled = mcpConfig.mcp_enabled !== 'false'
    mcpForm.auto_start = mcpConfig.mcp_auto_start === 'true'
    mcpForm.port = parseInt(mcpConfig.mcp_port || '8787')
    mcpForm.bind_address = mcpConfig.mcp_bind_address || '127.0.0.1'
    
    // 加载视频配置
    try {
      const videoConfig = await invoke<Record<string, string>>('get_video_config')
      videoForm.resolution = videoConfig.video_resolution || '1080p'
      videoForm.fps = parseInt(videoConfig.video_fps || '30')
      videoForm.ffmpegPath = videoConfig.video_ffmpeg_path || ''
    } catch (e) {
      console.log('加载视频配置失败，使用默认值')
    }
    
  } catch (error) {
    console.log('加载配置失败:', error)
  }
}

onMounted(() => {
  loadConfig()
  loadMCPStatus()
  loadMCPTools()
})
</script>

<style scoped>
.settings-page {
  padding: 0;
}

.form-tip {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

.form-tip a {
  color: #409eff;
  text-decoration: none;
}

.form-tip a:hover {
  text-decoration: underline;
}

.token-usage {
  margin-top: 20px;
  padding: 15px;
  background: #f5f7fa;
  border-radius: 4px;
}

.about-content {
  text-align: center;
  padding: 40px;
}

.logo h2 {
  margin: 0 0 30px 0;
  color: #409eff;
  font-size: 28px;
}

.version-info {
  margin-bottom: 30px;
}

.version-info p {
  margin: 10px 0;
  color: #606266;
}

.actions {
  display: flex;
  gap: 12px;
  justify-content: center;
}

.status-info {
  margin-top: 8px;
  padding: 12px;
  background: #f0f9eb;
  border-radius: 4px;
  border: 1px solid #e1f3d8;
}

.status-info p {
  margin: 4px 0;
  font-size: 13px;
  color: #67c23a;
}

.status-info code {
  background: #e6f7ff;
  padding: 2px 6px;
  border-radius: 3px;
  color: #1890ff;
  font-family: monospace;
}

.mcp-info {
  margin-top: 20px;
}

.mcp-alert {
  margin-bottom: 20px;
}

.mcp-alert :deep(.el-alert__content) {
  text-align: left;
}

.mcp-alert p {
  margin: 8px 0;
  color: #606266;
}

.mcp-alert ul,
.mcp-alert ol {
  margin: 8px 0;
  padding-left: 20px;
}

.mcp-alert li {
  margin: 4px 0;
  color: #606266;
}

.mcp-alert code {
  background: #f5f7fa;
  padding: 2px 6px;
  border-radius: 3px;
  color: #909399;
  font-family: monospace;
  font-size: 12px;
}

.tool-list {
  margin-top: 20px;
}

.mt-2 {
  margin-top: 8px;
}
</style>
