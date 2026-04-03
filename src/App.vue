<template>
  <el-config-provider :locale="zhCn">
    <div class="app-container">
      <!-- 顶部导航 -->
      <header class="app-header">
        <div class="app-header__left">
          <div class="app-logo">
            <el-icon :size="28" color="#1677FF">
              <MagicStick />
            </el-icon>
          </div>

          <!-- 顶部菜单 -->
          <nav class="app-nav">
            <router-link
              v-for="item in menuItems"
              :key="item.path"
              :to="item.path"
              class="nav-item"
              :class="{ 'nav-item--active': isActive(item.path) }"
            >
              <el-icon :size="18">
                <component :is="item.icon" />
              </el-icon>
              <span>{{ item.label }}</span>
            </router-link>
          </nav>
        </div>

        <div class="app-header__right">
          <el-button text circle>
            <el-icon :size="20"><Bell /></el-icon>
          </el-button>
          <el-button text circle>
            <el-icon :size="20"><QuestionFilled /></el-icon>
          </el-button>
          <el-dropdown trigger="click">
            <div class="user-avatar">
              <el-avatar :size="32" icon="UserFilled" />
            </div>
            <template #dropdown>
              <el-dropdown-menu>
                <el-dropdown-item>个人设置</el-dropdown-item>
                <el-dropdown-item divided>退出登录</el-dropdown-item>
              </el-dropdown-menu>
            </template>
          </el-dropdown>
        </div>
      </header>

      <!-- 主内容区 -->
      <main class="app-main">
        <router-view />
      </main>
    </div>
  </el-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import zhCn from 'element-plus/es/locale/lang/zh-cn'
import {
  House,
  Document,
  Microphone,
  Headset,
  Bell,
  User,
  VideoCamera,
  QuestionFilled,
  MagicStick
} from '@element-plus/icons-vue'
import type { Component } from 'vue'

interface MenuItem {
  path: string
  label: string
  icon: Component
}

const router = useRouter()
const route = useRoute()

const menuItems: MenuItem[] = [
  { path: '/dashboard', label: '工作台', icon: House },
  { path: '/novels', label: '网文管理', icon: Document },
  { path: '/audio', label: '音频制作', icon: Microphone },
  { path: '/mixer', label: '音频混音', icon: Headset },
  { path: '/ambient', label: '环境音', icon: Bell },
  { path: '/characters', label: '角色管理', icon: User },
  { path: '/video', label: '视频生成', icon: VideoCamera },
]

const isActive = (path: string) => {
  return route.path.startsWith(path)
}
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  width: 100vw;
  height: 100vh;
  overflow: hidden;
}

/* 顶部导航 */
.app-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: var(--header-height);
  padding: 0 var(--page-padding);
  background: var(--color-bg-container);
  border-bottom: 1px solid var(--color-border-light);
  flex-shrink: 0;
}

.app-header__left,
.app-header__center,
.app-header__right {
  display: flex;
  align-items: center;
  gap: var(--space-6);
}

.app-header__left {
  flex: 1;
}

.app-header__right {
  flex: 1;
  justify-content: flex-end;
}

/* Logo */
.app-logo {
  display: flex;
  align-items: center;
}

/* 顶部导航 */
.app-nav {
  display: flex;
  align-items: center;
  gap: var(--space-1);
  margin-left: var(--space-8);
}

.nav-item {
  display: flex;
  align-items: center;
  gap: var(--space-2);
  padding: var(--space-2) var(--space-4);
  color: var(--color-text-secondary);
  font-size: var(--text-sm);
  font-weight: var(--font-medium);
  text-decoration: none;
  border-radius: var(--radius-md);
  transition: all var(--transition-fast);
  white-space: nowrap;
}

.nav-item:hover {
  background: var(--color-bg-hover);
  color: var(--color-text-primary);
}

.nav-item--active {
  background: var(--color-primary-50);
  color: var(--color-primary-500);
}

.nav-item--active:hover {
  background: var(--color-primary-100);
}

/* 用户头像 */
.user-avatar {
  cursor: pointer;
  padding: var(--space-1);
  border-radius: var(--radius-full);
  transition: background var(--transition-fast);
}

.user-avatar:hover {
  background: var(--color-bg-hover);
}

/* 主内容区 */
.app-main {
  flex: 1;
  background: var(--color-bg-base);
  overflow-y: auto;
  overflow-x: hidden;
}

/* 响应式 */
@media (max-width: 1200px) {
  .nav-item span {
    display: none;
  }
}
</style>
