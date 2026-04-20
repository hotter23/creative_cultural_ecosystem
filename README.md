# 承接各类软件开发，微信号：hotter23 手机号:13705177381
# AI 内容创作平台

一个基于 Vue 3 + Tauri 2.0 的跨平台桌面应用，用于网文、音频、视频动画的生成与管理。

**架构特色**: 支持 MCP (Model Control Protocol) 服务端能力，可通过 OpenClaw 等 Agent 以自然语言对话方式进行内容创作。

```
┌──────────────┐   MCP 协议   ┌─────────────────────────────────┐
│   OpenClaw   │◀────────────▶│       AI 内容创作平台            │
│   (Agent)    │   调用       │  (GUI 界面 + MCP 服务端)         │
└──────────────┘              └─────────────────────────────────┘
```

## 技术栈

- **前端框架**: Vue 3 + TypeScript
- **桌面框架**: Tauri 2.0
- **UI 组件**: Element Plus
- **状态管理**: Pinia
- **路由**: Vue Router
- **数据库**: SQLite (rusqlite)
- **富文本编辑**: WangEditor
- **智能协议**: MCP (Model Control Protocol)
- **AI 集成**: MiniMax (文本生成 + TTS + 图像生成)
- **音频处理**: FFmpeg (内置打包)
- **环境音生成**: Stable Audio Open (本地 AI 模型)
- **包管理**: pnpm

## 项目结构

```
creative_cultural_ecosystem/     # 项目根目录
├── src/                          # 前端代码
│   ├── assets/                   # 静态资源
│   ├── router/                   # 路由配置
│   ├── stores/                   # Pinia 状态管理
│   ├── styles/                   # 全局样式
│   ├── views/                    # 页面组件
│   │   ├── ambient/             # 环境音管理模块
│   │   ├── audio/                # 音频制作模块
│   │   ├── characters/           # 角色管理模块
│   │   ├── dashboard/            # 工作台
│   │   ├── novels/               # 网文管理模块
│   │   │   ├── chapter-create.vue
│   │   │   ├── chapter-edit.vue
│   │   │   ├── chapters.vue
│   │   │   ├── create.vue
│   │   │   ├── detail.vue
│   │   │   └── index.vue
│   │   ├── settings/             # 系统设置
│   │   └── video/                # 视频生成模块
│   ├── App.vue                   # 主应用组件
│   ├── env.d.ts                  # 环境类型声明
│   ├── main.js                   # 主入口文件
│   └── main.ts                   # TypeScript 主入口
├── src-tauri/                    # Tauri 后端代码
│   ├── bin/                      # 内置 FFmpeg 二进制文件
│   ├── capabilities/             # Tauri 权限配置
│   ├── gen/                      # 生成的代码
│   ├── icons/                    # 应用图标
│   ├── src/
│   │   ├── clients/              # 第三方客户端
│   │   │   ├── minimax.rs        # MiniMax AI 客户端
│   │   │   └── mod.rs
│   │   ├── commands/             # Tauri 命令
│   │   │   ├── ai.rs             # AI 相关命令
│   │   │   ├── ambient.rs        # 环境音模块命令
│   │   │   ├── audio.rs          # 音频模块命令
│   │   │   ├── chapter.rs        # 章节相关命令
│   │   │   ├── character.rs      # 角色管理命令
│   │   │   ├── mcp.rs            # MCP 服务管理命令
│   │   │   ├── mod.rs
│   │   │   ├── novel.rs          # 网文相关命令
│   │   │   ├── system.rs         # 系统设置命令
│   │   │   └── video.rs          # 视频相关命令
│   │   ├── db/                   # 数据库模块
│   │   │   ├── migrations.rs     # 数据库迁移
│   │   │   ├── mod.rs            # 数据库管理
│   │   │   └── models.rs         # 数据模型
│   │   ├── mcp/                  # MCP 服务端模块
│   │   │   ├── discovery.rs      # HTTP 服务发现
│   │   │   ├── error.rs          # 错误定义
│   │   │   ├── mod.rs            # MCP 服务管理
│   │   │   ├── protocol.rs       # MCP 协议定义
│   │   │   ├── server.rs         # MCP 核心服务
│   │   │   └── tools/            # MCP 工具定义
│   │   │       ├── ai.rs
│   │   │       ├── ambient.rs
│   │   │       ├── audio.rs
│   │   │       ├── chapter.rs
│   │   │       ├── chapter_audio.rs
│   │   │       ├── character.rs
│   │   │       ├── mod.rs
│   │   │       └── novel.rs
│   │   ├── lib.rs                # Rust 库入口
│   │   └── main.rs               # Rust 主入口
│   ├── target/                   # 构建输出目录
│   ├── .gitignore
│   ├── build.rs                  # 构建脚本
│   ├── Cargo.lock                # 依赖锁定文件
│   ├── Cargo.toml                # Rust 依赖配置
│   └── tauri.conf.json           # Tauri 配置
├── scripts/                      # Python 脚本
│   ├── build/                    # 构建输出
│   ├── dist/                     # 分发文件
│   ├── model/                    # 模型文件
│   ├── stable_audio_direct.py    # 稳定音频直接生成
│   ├── stable_audio_generate.py  # 稳定音频生成主脚本
│   ├── stable_audio_optimized.py # 优化版稳定音频生成
│   ├── stable_audio_simple_final.py # 简化版稳定音频生成
│   ├── stable_audio_transformers.py # 使用 Transformers 的稳定音频生成
│   └── quantize_model.py         # 模型量化脚本
├── dist/                         # 前端构建输出
├── public/                       # 公共静态资源
├── .gitignore
├── agent.md                      # Agent 相关文档
├── index.html                    # 前端入口 HTML
├── package.json                  # npm 依赖配置
├── pnpm-lock.yaml                # pnpm 依赖锁定文件
├── README.md                     # 项目说明文档
├── SCRIPTS_BUNDLE_GUIDE.md       # 脚本打包指南
└── SCRIPTS_PATH_GUIDE.md         # 脚本路径指南
```

## 已实现功能 ✅

### 1. 基础框架

- ✅ Vue 3 + TypeScript + Vite 项目初始化
- ✅ Tauri 2.0 桌面应用集成
- ✅ Element Plus UI 组件库配置
- ✅ Pinia 状态管理
- ✅ Vue Router 路由配置
- ✅ SQLite 数据库集成（ORM 层）
- ✅ 自动数据库迁移

### 2. 网文管理模块 (Novels)

- ✅ 网文项目列表展示
- ✅ 创建/编辑/删除网文项目
- ✅ 项目详情查看
- ✅ 章节管理（列表、创建、编辑、删除）
- ✅ 富文本编辑器（WangEditor）
- ✅ 字数统计功能
- ✅ 项目状态管理（草稿/写作中/已完成）

### 3. 章节编辑高级功能

- ✅ 内容自动分段落/分割句子
- ✅ 对话识别与标记
- ✅ 角色对话标记分配
- ✅ 段落类型标记（旁白/对话/环境音）
- ✅ 批量段落标记
- ✅ 纯文本内容自动提取
- ✅ 段落音色绑定（通过角色获取）

### 4. 角色管理模块 (Characters)

- ✅ 角色列表展示（按小说分组）
- ✅ 创建/编辑/删除角色
- ✅ 角色信息配置（姓名、别名、性别、角色定位）
- ✅ 详细描述（外貌特征、性格特点、背景故事）
- ✅ 标签分类管理
- ✅ 音色绑定（支持 94 种官方音色，5种语言）
- ✅ AI 角色提取（从文本自动提取）
- ✅ AI 角色形象生成（MiniMax Image-01）
- ✅ 角色形象图片管理

### 5. 音频制作模块 (Audio)

- ✅ 智能 FFmpeg 发现（打包内置/系统PATH/用户配置）
- ✅ 章节音频项目管理
- ✅ 段落级 TTS 生成（通过 MiniMax 实现）
- ✅ 每段绑定角色和音色
- ✅ 音色分类展示（中文普通话/粤语/英文/日文/韩文）
- ✅ 角色音色自动获取（段落通过角色获取音色）
- ✅ 单段语速调节（0.5x - 2.0x）
- ✅ 批量音频生成
- ✅ 段落播放预览
- ✅ 多段音频智能合并（段落列表右侧「生成音频」已改为「合并音频」）
- ✅ 制作进度实时显示
- ✅ 重新合并功能
- ✅ 导出成品音频文件
- ✅ 生成状态持久化
- ✅ 已生成音频的段落提供「重新生成音频」按钮
- ✅ 每个段落都有独立的「生成音频」按钮（pending状态显示"生成音频"，completed状态显示"重新生成"，failed状态显示"重试"）
- ✅ 无需预先创建音频任务，点击段落"生成音频"按钮时自动创建任务记录
- ✅ 音频播放功能优化（支持多种音频格式、自动识别MIME类型、改进错误提示）
- ✅ 统一音频文件存储路径（使用 exe 安装目录下的 data/audio 目录，与日志路径保持一致）
- ✅ 日志系统（所有日志自动保存到 exe 安装目录下的 logs/ 目录，支持按日期轮换）

### 6. 环境音模块 (Ambient Sound)

- ✅ 环境音列表管理
- ✅ 类别筛选（自然、城市、室内、幻想、战斗等）
- ✅ AI 环境音生成（Stable Audio Open）
- ✅ MCP服务生成环境音自动录入系统
- ✅ 环境音时长和音量配置
- ✅ 循环播放支持
- ✅ 淡入淡出效果配置
- ✅ 段落环境音绑定（支持开头、结尾、叠加三种位置模式）
- ✅ 生成进度跟踪

### 7. 音频混音模块 (Audio Mixer) ⭐ 新增

- ✅ 独立混音页面入口
- ✅ 小说/章节/段落三级选择
- ✅ 段落级环境音配置
- ✅ 批量混音功能
- ✅ 章节音频合并（优先使用混音路径）
- ✅ 段落混音路径独立保存
- ✅ 章节混音路径保存
- ✅ 混音预设模板
- ✅ 混音状态实时显示
- ✅ 单段落混音和播放

### 8. 系统设置模块 (Settings)

- ✅ **MiniMax AI 配置**
  - API Key 安全存储
  - Group ID 配置
  - 默认模型选择
  - 服务启用/禁用开关
  - 连接测试功能
  - Token 使用统计展示

- ✅ **TTS 语音配置**
  - 默认音色选择（94 种音色）
  - 语速调节滑块（0.5x - 2.0x）
  - 配置持久化存储

- ✅ **Python 环境配置**
  - Python 路径配置（用于 Stable Audio）
  - 自动检测常见 Python 安装路径

- ✅ **MCP 服务配置**
  - 服务启用/禁用控制
  - 开机自动启动选项
  - 绑定地址配置（127.0.0.1 / 0.0.0.0）
  - 端口号配置（1024-65535）
  - 实时状态监控（运行中/已停止）
  - 服务端点信息展示
  - 可用工具计数
  - 启动/停止服务按钮
  - 一键注册到 Agent 功能

### 8. MCP 服务端 (Model Control Protocol)

- ✅ **核心服务**
  - HTTP 服务（hyper）
  - JSON-RPC 2.0 协议支持
  - CORS 跨域支持
  - 服务发现端点（.well-known/mcp）
  - 工具注册与调用框架

- ✅ **MCP 工具集**

| 分类 | 工具名称 | 功能描述 |
|------|----------|----------|
| **网文管理** | `list_novels` | 获取小说列表 |
| | `get_novel` | 获取单个小说详情 |
| | `create_novel` | 创建新小说 |
| | `update_novel` | 更新小说信息 |
| | `delete_novel` | 删除小说及关联内容 |
| **章节管理** | `list_chapters` | 获取小说章节列表 |
| | `get_chapter` | 获取章节详情（含内容） |
| | `create_chapter` | 创建新章节（支持HTML富文本） |
| | `update_chapter` | 更新章节内容 |
| | `delete_chapter` | 删除章节 |
| | `import_content_as_single_paragraph` | 导入内容为单段落 |
| | `batch_mark_paragraphs` | 批量标记段落类型 |
| | `get_chapter_paragraphs` | 获取章节段落列表 |
| **AI 内容生成** | `ai_generate_novel_content` | AI 生成小说内容 |
| | `ai_continue_novel_content` | AI 续写小说 |
| | `ai_polish_content` | AI 内容润色 |
| | `ai_summarize_content` | AI 内容摘要 |
| | `ai_suggest_plot` | AI 情节建议 |
| | `ai_extract_characters` | AI 角色提取 |
| **音频工具** | `text_to_speech` | 文本转语音（TTS） |
| | `get_voice_list` | 获取可用音色列表 |
| **章节音频管理** | `get_chapter_audio_detail` | 获取章节音频详情 |
| | `get_audio_paragraphs` | 获取音频段落列表 |
| | `generate_chapter_audio` | 生成章节音频 |
| | `regenerate_paragraph_audio` | 重新生成段落音频 |
| | `merge_chapter_audio` | 合并章节音频 |
| **环境音工具** | `list_ambient_sounds` | 获取环境音列表 |
| | `get_ambient_sounds` | 按类别获取环境音 |
| | `generate_ambient_sound` | 生成环境音 |
| | `bind_ambient_to_paragraph` | 绑定环境音到段落 |
| **角色管理** | `list_characters` | 获取小说角色列表 |
| | `get_character` | 获取角色详情 |
| | `create_character` | 创建新角色 |
| | `update_character` | 更新角色信息 |
| | `delete_character` | 删除角色 |
| | `bind_character_voice` | 绑定角色音色 |
| | `get_voice_list` | 获取可用音色列表 |
| | `list_character_images` | 获取角色形象图片列表 |

### 9. 工作台 (Dashboard)

- ✅ 统计卡片展示（网文数、章节数、音频项目数）
- ✅ 最近项目列表
- ✅ 快捷操作入口

### 10. 数据库设计

- ✅ 网文项目表 (novels)
- ✅ 章节表 (chapters)
- ✅ 段落表 (chapter_paragraphs) - 支持旁白/对话/环境音
- ✅ 音频项目表 (chapter_audios)
- ✅ 环境音表 (ambient_sounds)
- ✅ 角色表 (characters)
- ✅ 角色形象表 (character_images)
- ✅ 视频项目表 (videos)
- ✅ 视频分镜表 (video_scenes)
- ✅ 系统配置表 (system_config)

## 待开发功能 🚧

### 视频生成模块 (Video)

- 🚧 视频项目管理
- 🚧 分镜脚本编辑
- 🚧 AI 视频生成集成
- 🚧 视频预览与导出

## 安装与运行

### 环境要求

- Node.js 18+
- Rust 1.70+
- pnpm 8+
- Python 3.9+ (用于环境音生成)
- Windows 10+ / macOS 10.15+ / Linux

### 开发运行

```bash
# 安装依赖
pnpm install

# 运行 Tauri 开发模式
pnpm run tauri:dev

# 仅运行前端（不启动 Tauri）
pnpm run dev

## 数据存储位置

应用程序安装后，所有用户数据都存储在 **exe 安装目录下** 的 `data` 文件夹中，而不是用户个人目录。这种设计确保了数据与应用程序的可移植性。

### 目录结构

```
exe安装目录/
├── content_creator.exe          # 应用程序主文件
├── data/                        # 用户数据目录
│   ├── content_creator.db       # SQLite 数据库文件
│   ├── audio/                   # TTS 生成的音频文件
│   │   └── tts_*.mp3           # 语音合成音频
│   ├── mixer/                   # 混音文件目录
│   │   └── 段落混音和章节混音文件
│   └── character_images/        # 角色形象图片
└── logs/                        # 应用程序日志
    └── *.log                    # 按日期轮换的日志文件
```

### 详细说明

#### 1. 数据库文件
- **位置**: `{exe安装目录}/data/content_creator.db`
- **名称**: `content_creator.db`（内容创作者数据库）
- **类型**: SQLite 数据库
- **包含内容**:
  - 网文项目信息（novels 表）
  - 章节内容（chapters 表）
  - 段落数据（chapter_paragraphs 表）
  - 音频任务记录（chapter_audios 表）
  - 环境音配置（ambient_sounds 表）
  - 角色信息（characters 表）
  - 角色形象图片（character_images 表）
  - 视频项目（videos 表）
  - 系统配置（system_config 表）

#### 2. 音频文件
- **TTS 音频**: `{exe安装目录}/data/audio/`
- **混音文件**: `{exe安装目录}/data/mixer/`
- **角色形象**: `{exe安装目录}/data/character_images/`
- **格式**: MP3（语音）、WAV/其他（混音）

#### 3. 日志文件
- **位置**: `{exe安装目录}/logs/`
- **特性**: 按日期自动轮换（每天一个新文件）
- **命名格式**: `YYYY-MM-DD.log`
- **内容**: 包含所有应用程序操作日志，便于问题排查

#### 4. 内置资源
- **FFmpeg**: `{exe安装目录}/bin/ffmpeg.exe`
  - 用于音频处理和合并

### 数据迁移和备份

由于所有数据都存储在应用程序目录下，迁移或备份时需要：

1. 复制整个 `data` 文件夹
2. 复制 `logs` 文件夹（如需保留日志）
3. 重新安装后，将数据文件夹放回 exe 相同目录

### 注意事项

- **开发模式**: 开发时数据目录位于项目根目录的 `src-tauri/` 下
- **安装模式**: 打包安装后，数据目录位于 exe 安装位置
- **可移植性**: 数据与 exe 同目录，方便整体迁移
- **清理数据**: 删除 `data` 文件夹将清除所有用户数据（谨慎操作）

# 构建生产版本
pnpm run tauri:build
```

### 配置要求

首次运行需要在「系统设置」中配置：

1. **MiniMax API Key**（用于 AI 内容生成、TTS、图像生成）
   - 获取地址：https://platform.minimaxi.com

2. **Python 环境**（用于 Stable Audio 环境音生成）
   - 系统会自动检测，也可手动配置路径

## 开发说明

### 项目状态

当前版本：**0.2.0**

核心功能已完成：
- ✅ 网文创作与管理
- ✅ 角色管理与 AI 形象生成
- ✅ 高级音频制作（分段落、角色配音、批量生成、合并导出）
- ✅ 环境音管理与 AI 生成
- ✅ MCP 服务端（支持 Agent 调用）

### 稳定音频生成

项目包含多个稳定音频生成脚本，位于 `scripts/` 目录：

- `stable_audio_generate.py` - 主生成脚本
- `stable_audio_direct.py` - 直接生成方式
- `stable_audio_optimized.py` - 优化版生成
- `stable_audio_simple_final.py` - 简化版生成
- `stable_audio_transformers.py` - 使用 Transformers 库
- `quantize_model.py` - 模型量化工具

## 支持我
如果您觉得这个项目对您有帮助，您可以扫描以下二维码进行捐赠：

<img width="828" height="1124" alt="2757ca9078c29ccc3ced920bebd28061" src="https://github.com/user-attachments/assets/aaaa38f0-2841-4f1f-8653-c67f5f803186" />
