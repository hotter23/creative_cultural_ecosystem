<template>
  <div class="page-container">
    <!-- 页面头部 -->
    <div v-if="showHeader" class="page-container__header">
      <div class="page-container__header-left">
        <h1 class="page-container__title">{{ title }}</h1>
        <el-breadcrumb v-if="breadcrumbs?.length" separator="/">
          <el-breadcrumb-item
            v-for="item in breadcrumbs"
            :key="item.path"
            :to="item.path"
          >
            {{ item.title }}
          </el-breadcrumb-item>
        </el-breadcrumb>
      </div>

      <div class="page-container__header-right">
        <slot name="header-actions"></slot>
      </div>
    </div>

    <!-- 页面内容 -->
    <div class="page-container__body" :class="{ 'page-container__body--no-padding': noPadding }">
      <slot></slot>
    </div>

    <!-- 页面底部 -->
    <div v-if="$slots.footer" class="page-container__footer">
      <slot name="footer"></slot>
    </div>
  </div>
</template>

<script setup lang="ts">
interface BreadcrumbItem {
  title: string;
  path?: string;
}

interface Props {
  title?: string;
  showHeader?: boolean;
  breadcrumbs?: BreadcrumbItem[];
  noPadding?: boolean;
}

withDefaults(defineProps<Props>(), {
  title: '',
  showHeader: true,
  noPadding: false
});
</script>

<style scoped>
.page-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--color-bg-base);
}

.page-container__header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: var(--space-5) var(--page-padding);
  background: var(--color-bg-container);
  border-bottom: 1px solid var(--color-border-light);
  flex-shrink: 0;
}

.page-container__header-left {
  display: flex;
  flex-direction: column;
  gap: var(--space-2);
}

.page-container__title {
  font-size: var(--text-2xl);
  font-weight: var(--font-semibold);
  color: var(--color-text-primary);
  margin: 0;
}

.page-container__header-right {
  display: flex;
  align-items: center;
  gap: var(--space-3);
}

.page-container__body {
  flex: 1;
  padding: var(--page-padding);
  overflow-y: auto;
}

.page-container__body--no-padding {
  padding: 0;
}

.page-container__footer {
  padding: var(--space-4) var(--page-padding);
  background: var(--color-bg-container);
  border-top: 1px solid var(--color-border-light);
  flex-shrink: 0;
}
</style>
