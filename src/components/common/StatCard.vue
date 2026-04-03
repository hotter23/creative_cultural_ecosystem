<template>
  <div
    class="stat-card"
    :class="{ 'stat-card--clickable': clickable }"
    @click="handleClick"
  >
    <div
      class="stat-card__icon"
      :style="{ background: iconBackground }"
    >
      <el-icon :size="24">
        <component :is="icon" />
      </el-icon>
    </div>

    <div class="stat-card__content">
      <div class="stat-card__value">{{ displayValue }}</div>
      <div class="stat-card__label">{{ label }}</div>
    </div>

    <div
      v-if="trend"
      class="stat-card__trend"
      :class="`stat-card__trend--${trendType}`"
    >
      {{ trendIcon }} {{ trend }}
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, type Component } from 'vue';

interface Props {
  icon: Component;
  value: string | number;
  label: string;
  iconBackground?: string;
  trend?: string;
  trendType?: 'up' | 'down';
  clickable?: boolean;
  suffix?: string;
}

const props = withDefaults(defineProps<Props>(), {
  iconBackground: '#1677FF',
  trendType: 'up',
  clickable: false,
  suffix: ''
});

const emit = defineEmits<{
  (e: 'click'): void;
}>();

const displayValue = computed(() => {
  if (typeof props.value === 'number' && props.suffix) {
    return `${props.value}${props.suffix}`;
  }
  return props.value;
});

const trendIcon = computed(() => {
  return props.trendType === 'up' ? '↑' : '↓';
});

function handleClick() {
  if (props.clickable) {
    emit('click');
  }
}
</script>

<style scoped>
.stat-card {
  display: flex;
  align-items: center;
  gap: var(--space-4);
  padding: var(--space-5);
  background: var(--color-bg-container);
  border-radius: var(--radius-lg);
  box-shadow: var(--shadow-sm);
  transition: all var(--transition-base);
}

.stat-card--clickable {
  cursor: pointer;
}

.stat-card--clickable:hover {
  box-shadow: var(--shadow-hover);
  transform: translateY(-2px);
}

.stat-card__icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 48px;
  height: 48px;
  border-radius: var(--radius-lg);
  color: white;
  flex-shrink: 0;
}

.stat-card__content {
  flex: 1;
  min-width: 0;
}

.stat-card__value {
  font-size: var(--text-2xl);
  font-weight: var(--font-bold);
  color: var(--color-text-primary);
  line-height: 1.2;
}

.stat-card__label {
  font-size: var(--text-sm);
  color: var(--color-text-secondary);
  margin-top: var(--space-1);
}

.stat-card__trend {
  display: inline-flex;
  align-items: center;
  gap: var(--space-1);
  font-size: var(--text-xs);
  font-weight: var(--font-medium);
  padding: var(--space-1) var(--space-2);
  border-radius: var(--radius-full);
  flex-shrink: 0;
}

.stat-card__trend--up {
  background: var(--color-success-bg);
  color: var(--color-success);
}

.stat-card__trend--down {
  background: var(--color-error-bg);
  color: var(--color-error);
}
</style>
