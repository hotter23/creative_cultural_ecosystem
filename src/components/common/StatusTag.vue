<template>
  <span
    class="status-tag"
    :class="`status-tag--${status}`"
  >
    <span class="status-tag__dot" />
    <span class="status-tag__label">{{ displayLabel }}</span>
  </span>
</template>

<script setup lang="ts">
import { computed } from 'vue';

type StatusType = 'pending' | 'processing' | 'done' | 'error' | 'warning';

interface Props {
  status: StatusType;
  label?: string;
}

const props = withDefaults(defineProps<Props>(), {});

const labels: Record<StatusType, string> = {
  pending: '待处理',
  processing: '进行中',
  done: '已完成',
  error: '错误',
  warning: '警告'
};

const displayLabel = computed(() => {
  return props.label || labels[props.status];
});
</script>

<style scoped>
.status-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  border-radius: var(--radius-full);
  line-height: 1;
}

.status-tag__dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  flex-shrink: 0;
}

.status-tag__label {
  white-space: nowrap;
}

/* 待处理状态 */
.status-tag--pending {
  background: rgba(217, 217, 217, 0.3);
  color: var(--color-text-secondary);
}

.status-tag--pending .status-tag__dot {
  background: var(--color-text-secondary);
}

/* 进行中状态 */
.status-tag--processing {
  background: var(--color-info-bg);
  color: var(--color-info);
}

.status-tag--processing .status-tag__dot {
  background: var(--color-info);
  animation: pulse 1.5s infinite;
}

/* 已完成状态 */
.status-tag--done {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.status-tag--done .status-tag__dot {
  background: var(--color-success);
}

/* 错误状态 */
.status-tag--error {
  background: var(--color-error-bg);
  color: var(--color-error);
}

.status-tag--error .status-tag__dot {
  background: var(--color-error);
}

/* 警告状态 */
.status-tag--warning {
  background: var(--color-warning-bg);
  color: var(--color-warning);
}

.status-tag--warning .status-tag__dot {
  background: var(--color-warning);
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}
</style>
