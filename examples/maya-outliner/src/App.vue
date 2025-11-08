<script setup lang="ts">
import { ref, onMounted } from 'vue'
import OutlinerTree from './components/OutlinerTree.vue'
import { useMayaIPC } from './composables/useMayaIPC'
import type { MayaNode } from './types'

const { sendToMaya, onMayaEvent } = useMayaIPC()
const sceneData = ref<MayaNode[]>([])
const selectedNode = ref<string | null>(null)
const searchQuery = ref('')
const isConnected = ref(false)

onMounted(() => {
  // Request initial scene data
  sendToMaya('get_scene_hierarchy', {})

  // Listen for scene updates from Maya
  onMayaEvent('scene_updated', (data: MayaNode[]) => {
    sceneData.value = data
    isConnected.value = true
  })

  onMayaEvent('selection_changed', (data: { node: string }) => {
    selectedNode.value = data.node
  })
})

const handleNodeSelect = (nodeName: string) => {
  selectedNode.value = nodeName
  sendToMaya('select_node', { node_name: nodeName })
}

const handleVisibilityToggle = (nodeName: string, visible: boolean) => {
  sendToMaya('set_visibility', { node_name: nodeName, visible })
}

</script>

<template>
  <div class="app-container">
    <header class="app-header">
      <h1>Maya Outliner</h1>
      <div class="header-actions">
        <div class="connection-status" :class="{ connected: isConnected }">
          <span class="status-dot"></span>
          {{ isConnected ? 'Connected' : 'Disconnected' }}
        </div>
      </div>
    </header>

    <div class="search-bar">
      <input
        v-model="searchQuery"
        type="text"
        placeholder="Search nodes..."
        class="search-input"
      />
    </div>

    <main class="app-main">
      <OutlinerTree
        :nodes="sceneData"
        :selected-node="selectedNode"
        :search-query="searchQuery"
        @node-select="handleNodeSelect"
        @visibility-toggle="handleVisibilityToggle"
      />
    </main>

    <footer class="app-footer">
      <p>AuroraView Maya Outliner Example</p>
      <p class="stats">
        Nodes: {{ sceneData.length }} | Selected: {{ selectedNode || 'None' }}
      </p>
    </footer>
  </div>
</template>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  background: #1e1e1e;
  color: #e0e0e0;
}

.app-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 1rem 1.5rem;
  background: #252525;
  border-bottom: 1px solid #3a3a3a;
}

.app-header h1 {
  font-size: 1.5rem;
  font-weight: 600;
  margin: 0;
  color: #4fc3f7;
}

.header-actions {
  display: flex;
  gap: 1rem;
  align-items: center;
}

.connection-status {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  padding: 0.5rem 1rem;
  border-radius: 4px;
  background: #2a2a2a;
  font-size: 0.875rem;
}

.status-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: #f44336;
}

.connection-status.connected .status-dot {
  background: #4caf50;
}

.refresh-btn {
  padding: 0.5rem 1rem;
  background: #3a3a3a;
  border: 1px solid #4a4a4a;
  color: #e0e0e0;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.refresh-btn:hover {
  background: #4a4a4a;
  border-color: #5a5a5a;
}

.search-bar {
  padding: 1rem 1.5rem;
  background: #252525;
  border-bottom: 1px solid #3a3a3a;
}

.search-input {
  width: 100%;
  padding: 0.75rem 1rem;
  background: #2a2a2a;
  border: 1px solid #3a3a3a;
  border-radius: 4px;
  color: #e0e0e0;
  font-size: 0.875rem;
}

.search-input:focus {
  outline: none;
  border-color: #4fc3f7;
}

.app-main {
  flex: 1;
  overflow: auto;
  padding: 1rem 1.5rem;
}

.app-footer {
  padding: 0.75rem 1.5rem;
  background: #252525;
  border-top: 1px solid #3a3a3a;
  font-size: 0.75rem;
  color: #888;
}

.stats {
  margin-top: 0.25rem;
  color: #666;
}
</style>

