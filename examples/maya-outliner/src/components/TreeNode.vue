<script setup lang="ts">
import { ref, computed } from 'vue'
import type { MayaNode } from '../types'

interface Props {
  node: MayaNode
  selectedNode: string | null
  level: number
}

interface Emits {
  (e: 'node-select', nodeName: string): void
  (e: 'visibility-toggle', nodeName: string, visible: boolean): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

const isExpanded = ref(true)
const hasChildren = computed(() => props.node.children.length > 0)
const isSelected = computed(() => props.node.name === props.selectedNode)

const nodeIcon = computed(() => {
  switch (props.node.type) {
    case 'mesh':
      return '🔷'
    case 'camera':
      return '📷'
    case 'light':
      return '💡'
    case 'joint':
      return '🦴'
    case 'group':
    case 'transform':
      return '📁'
    case 'locator':
      return '📍'
    default:
      return '📄'
  }
})

const toggleExpand = () => {
  if (hasChildren.value) {
    isExpanded.value = !isExpanded.value
  }
}

const handleClick = () => {
  emit('node-select', props.node.name)
}

const toggleVisibility = (event: Event) => {
  event.stopPropagation()
  emit('visibility-toggle', props.node.name, !props.node.visible)
}
</script>

<template>
  <div class="tree-node">
    <div
      class="node-row"
      :class="{ selected: isSelected }"
      :style="{ paddingLeft: `${level * 20 + 8}px` }"
      @click="handleClick"
    >
      <button
        v-if="hasChildren"
        class="expand-btn"
        :class="{ expanded: isExpanded }"
        @click.stop="toggleExpand"
      >
        ▶
      </button>
      <span v-else class="expand-spacer"></span>

      <span class="node-icon">{{ nodeIcon }}</span>
      <span class="node-name">{{ node.name }}</span>
      <span class="node-type">{{ node.type }}</span>

      <button
        class="visibility-btn"
        :class="{ hidden: !node.visible }"
        @click="toggleVisibility"
        :title="node.visible ? 'Hide' : 'Show'"
      >
        {{ node.visible ? '👁️' : '🚫' }}
      </button>
    </div>

    <div v-if="hasChildren && isExpanded" class="node-children">
      <TreeNode
        v-for="child in node.children"
        :key="child.path"
        :node="child"
        :selected-node="selectedNode"
        :level="level + 1"
        @node-select="(nodeName) => emit('node-select', nodeName)"
        @visibility-toggle="(nodeName, visible) => emit('visibility-toggle', nodeName, visible)"
      />
    </div>
  </div>
</template>

<style scoped>
.tree-node {
  user-select: none;
}

.node-row {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 0.75rem;
  cursor: pointer;
  transition: background-color 0.15s;
  border-radius: 4px;
  margin: 2px 0;
}

.node-row:hover {
  background-color: #2a2a2a;
}

.node-row.selected {
  background-color: #3a3a3a;
  border-left: 3px solid #4fc3f7;
}

.expand-btn {
  width: 16px;
  height: 16px;
  padding: 0;
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  transition: transform 0.2s;
  font-size: 10px;
}

.expand-btn.expanded {
  transform: rotate(90deg);
}

.expand-btn:hover {
  color: #e0e0e0;
}

.expand-spacer {
  width: 16px;
}

.node-icon {
  font-size: 1rem;
  flex-shrink: 0;
}

.node-name {
  flex: 1;
  font-size: 0.875rem;
  color: #e0e0e0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.node-type {
  font-size: 0.75rem;
  color: #888;
  padding: 0.125rem 0.5rem;
  background: #2a2a2a;
  border-radius: 3px;
}

.visibility-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  background: none;
  border: none;
  cursor: pointer;
  opacity: 0.6;
  transition: opacity 0.2s;
  font-size: 14px;
}

.visibility-btn:hover {
  opacity: 1;
}

.visibility-btn.hidden {
  opacity: 0.3;
}

.node-children {
  margin-left: 0;
}
</style>

