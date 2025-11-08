/**
 * Maya node types
 */
export type MayaNodeType =
  | 'transform'
  | 'mesh'
  | 'camera'
  | 'light'
  | 'group'
  | 'joint'
  | 'locator'
  | 'unknown'

/**
 * Maya scene node
 */
export interface MayaNode {
  /** Node name */
  name: string

  /** Node type */
  type: MayaNodeType

  /** Full DAG path */
  path: string

  /** Parent node name (null for root nodes) */
  parent: string | null

  /** Child nodes */
  children: MayaNode[]

  /** Visibility state */
  visible: boolean

  /** Selection state */
  selected: boolean
}

/**
 * IPC message from frontend to Maya
 */
export interface IPCMessage {
  /** Event name */
  event: string

  /** Event data */
  data: Record<string, unknown>
}

/**
 * IPC event handler
 */
export type IPCEventHandler = (data: unknown) => void

/**
 * Maya IPC interface
 */
export interface MayaIPC {
  /** Send message to Maya */
  sendToMaya: (event: string, data: Record<string, unknown>) => void

  /** Register event handler */
  onMayaEvent: (event: string, handler: IPCEventHandler) => void

  /** Unregister event handler */
  offMayaEvent: (event: string, handler: IPCEventHandler) => void
}

