<script setup lang="ts">
import { computed } from 'vue'
import type { MayaNode } from '../types'
import TreeNode from './TreeNode.vue'

interface Props {
  nodes: MayaNode[]
  selectedNode: string | null
  searchQuery: string
}

interface Emits {
  (e: 'node-select', nodeName: string): void
  (e: 'visibility-toggle', nodeName: string, visible: boolean): void
}

const props = defineProps<Props>()
const emit = defineEmits<Emits>()

// Filter nodes based on search query
const filteredNodes = computed(() => {
  if (!props.searchQuery) {
    return props.nodes
  }

  const query = props.searchQuery.toLowerCase()
  const filterNode = (node: MayaNode): MayaNode | null => {
    const matches = node.name.toLowerCase().includes(query)
    const filteredChildren = node.children
      .map(filterNode)
      .filter((n): n is MayaNode => n !== null)

    if (matches || filteredChildren.length > 0) {
      return {
        ...node,
        children: filteredChildren,
      }
    }

    return null
  }

  return props.nodes
    .map(filterNode)
    .filter((n): n is MayaNode => n !== null)
})

const handleNodeSelect = (nodeName: string) => {
  emit('node-select', nodeName)
}

const handleVisibilityToggle = (nodeName: string, visible: boolean) => {
  emit('visibility-toggle', nodeName, visible)
}
</script>

<template>
  <div class="outliner-tree">
    <div v-if="filteredNodes.length === 0" class="empty-state">
      <p v-if="searchQuery">No nodes match "{{ searchQuery }}"</p>
      <p v-else>No nodes in scene</p>
    </div>

    <div v-else class="tree-container">
      <TreeNode
        v-for="node in filteredNodes"
        :key="node.path"
        :node="node"
        :selected-node="selectedNode"
        :level="0"
        @node-select="handleNodeSelect"
        @visibility-toggle="handleVisibilityToggle"
      />
    </div>
  </div>
</template>

<style scoped>
.outliner-tree {
  width: 100%;
  height: 100%;
  overflow: auto;
}

.tree-container {
  padding: 0.5rem 0;
}

.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 200px;
  color: #666;
  font-size: 0.875rem;
}
</style>

