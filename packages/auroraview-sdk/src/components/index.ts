/**
 * AuroraView SDK Components
 *
 * This module exports React components for the AuroraView SDK.
 *
 * @example
 * ```tsx
 * import { AgentSidebar } from '@auroraview/sdk/components';
 *
 * function App() {
 *   return (
 *     <div className="app">
 *       <MainContent />
 *       <AgentSidebar position="right" width={400} />
 *     </div>
 *   );
 * }
 * ```
 */

// AI Agent Sidebar
export {
  AgentSidebar,
  type AgentSidebarProps,
  type Message,
  type ToolCallInfo,
} from './AgentSidebar';

// Import CSS (consumers should import this separately or use bundler)
// import './AgentSidebar.css';

