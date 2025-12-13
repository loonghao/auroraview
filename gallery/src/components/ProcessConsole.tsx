import { useEffect, useRef, useState, useCallback } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';
import { useProcessEvents, type ProcessOutput, type ProcessExit } from '../hooks/useAuroraView';

interface LogEntry {
  id: number;
  pid: number;
  type: 'stdout' | 'stderr' | 'exit' | 'info';
  data: string;
  timestamp: number;
}

interface ProcessInfo {
  pid: number;
  title: string;
  running: boolean;
}

interface ProcessConsoleProps {
  isOpen: boolean;
  onClose: () => void;
  onKillProcess?: (pid: number) => void;
}

export function ProcessConsole({ isOpen, onClose, onKillProcess }: ProcessConsoleProps) {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [processes, setProcesses] = useState<Map<number, ProcessInfo>>(new Map());
  const [autoScroll, setAutoScroll] = useState(true);
  const [filter, setFilter] = useState<'all' | 'stdout' | 'stderr'>('all');
  const logContainerRef = useRef<HTMLDivElement>(null);
  const idCounter = useRef(0);

  // Add a log entry
  const addLog = useCallback((pid: number, type: LogEntry['type'], data: string) => {
    setLogs(prev => {
      const newLog: LogEntry = {
        id: idCounter.current++,
        pid,
        type,
        data,
        timestamp: Date.now(),
      };
      // Keep last 1000 entries
      const updated = [...prev, newLog];
      if (updated.length > 1000) {
        return updated.slice(-1000);
      }
      return updated;
    });
  }, []);

  // Subscribe to process events
  useProcessEvents({
    onStdout: (data: ProcessOutput) => {
      addLog(data.pid, 'stdout', data.data);
      // Mark process as running
      setProcesses(prev => {
        const updated = new Map(prev);
        if (!updated.has(data.pid)) {
          updated.set(data.pid, { pid: data.pid, title: `Process ${data.pid}`, running: true });
        }
        return updated;
      });
    },
    onStderr: (data: ProcessOutput) => {
      addLog(data.pid, 'stderr', data.data);
      setProcesses(prev => {
        const updated = new Map(prev);
        if (!updated.has(data.pid)) {
          updated.set(data.pid, { pid: data.pid, title: `Process ${data.pid}`, running: true });
        }
        return updated;
      });
    },
    onExit: (data: ProcessExit) => {
      const exitMsg = data.code !== null ? `exited with code ${data.code}` : 'terminated';
      addLog(data.pid, 'exit', exitMsg);
      setProcesses(prev => {
        const updated = new Map(prev);
        const proc = updated.get(data.pid);
        if (proc) {
          updated.set(data.pid, { ...proc, running: false });
        }
        return updated;
      });
    },
  });

  // Auto-scroll to bottom
  useEffect(() => {
    if (autoScroll && logContainerRef.current) {
      logContainerRef.current.scrollTop = logContainerRef.current.scrollHeight;
    }
  }, [logs, autoScroll]);

  // Handle scroll to detect manual scrolling
  const handleScroll = useCallback(() => {
    if (!logContainerRef.current) return;
    const { scrollTop, scrollHeight, clientHeight } = logContainerRef.current;
    const isAtBottom = scrollHeight - scrollTop - clientHeight < 50;
    setAutoScroll(isAtBottom);
  }, []);

  // Clear logs
  const handleClear = useCallback(() => {
    setLogs([]);
  }, []);

  // Filter logs
  const filteredLogs = filter === 'all'
    ? logs
    : logs.filter(log => log.type === filter || log.type === 'exit' || log.type === 'info');

  // Format timestamp
  const formatTime = (ts: number) => {
    const date = new Date(ts);
    return date.toLocaleTimeString('en-US', { hour12: false });
  };

  // Get running process count
  const runningCount = Array.from(processes.values()).filter(p => p.running).length;

  if (!isOpen) return null;

  return (
    <div className="fixed bottom-0 left-14 right-0 h-64 bg-[#1e1e1e] border-t border-[#3c3c3c] flex flex-col z-40">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-2 bg-[#252526] border-b border-[#3c3c3c]">
        <div className="flex items-center gap-4">
          <div className="flex items-center gap-2 text-[#cccccc]">
            <Icons.Terminal className="w-4 h-4" />
            <span className="text-sm font-medium">Process Output</span>
            {runningCount > 0 && (
              <span className="px-1.5 py-0.5 text-xs bg-green-500/20 text-green-400 rounded">
                {runningCount} running
              </span>
            )}
          </div>

          {/* Filter buttons */}
          <div className="flex items-center gap-1">
            <button
              onClick={() => setFilter('all')}
              className={cn(
                "px-2 py-1 text-xs rounded transition-colors",
                filter === 'all' ? "bg-[#3c3c3c] text-white" : "text-[#858585] hover:text-[#cccccc]"
              )}
            >
              All
            </button>
            <button
              onClick={() => setFilter('stdout')}
              className={cn(
                "px-2 py-1 text-xs rounded transition-colors",
                filter === 'stdout' ? "bg-[#3c3c3c] text-white" : "text-[#858585] hover:text-[#cccccc]"
              )}
            >
              stdout
            </button>
            <button
              onClick={() => setFilter('stderr')}
              className={cn(
                "px-2 py-1 text-xs rounded transition-colors",
                filter === 'stderr' ? "bg-[#3c3c3c] text-white" : "text-[#858585] hover:text-[#cccccc]"
              )}
            >
              stderr
            </button>
          </div>
        </div>

        <div className="flex items-center gap-2">
          {/* Auto-scroll indicator */}
          <button
            onClick={() => setAutoScroll(!autoScroll)}
            className={cn(
              "p-1.5 rounded transition-colors",
              autoScroll ? "text-blue-400" : "text-[#858585] hover:text-[#cccccc]"
            )}
            title={autoScroll ? "Auto-scroll enabled" : "Auto-scroll disabled"}
          >
            <Icons.ArrowDownToLine className="w-4 h-4" />
          </button>

          {/* Clear button */}
          <button
            onClick={handleClear}
            className="p-1.5 rounded text-[#858585] hover:text-[#cccccc] hover:bg-[#3c3c3c] transition-colors"
            title="Clear console"
          >
            <Icons.Trash2 className="w-4 h-4" />
          </button>

          {/* Close button */}
          <button
            onClick={onClose}
            className="p-1.5 rounded text-[#858585] hover:text-[#cccccc] hover:bg-[#3c3c3c] transition-colors"
          >
            <Icons.X className="w-4 h-4" />
          </button>
        </div>
      </div>

      {/* Log content */}
      <div
        ref={logContainerRef}
        onScroll={handleScroll}
        className="flex-1 overflow-auto font-mono text-sm p-2"
      >
        {filteredLogs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-[#858585]">
            <div className="text-center">
              <Icons.Terminal className="w-8 h-8 mx-auto mb-2 opacity-50" />
              <p>No output yet. Run a demo to see logs here.</p>
            </div>
          </div>
        ) : (
          filteredLogs.map(log => (
            <div
              key={log.id}
              className={cn(
                "flex items-start gap-2 py-0.5 hover:bg-[#2a2a2a]",
                log.type === 'stderr' && "text-red-400",
                log.type === 'exit' && "text-yellow-400",
                log.type === 'info' && "text-blue-400",
                log.type === 'stdout' && "text-[#d4d4d4]"
              )}
            >
              <span className="text-[#858585] text-xs flex-shrink-0">
                {formatTime(log.timestamp)}
              </span>
              <span className="text-[#569cd6] text-xs flex-shrink-0">
                [{log.pid}]
              </span>
              <span className={cn(
                "text-xs flex-shrink-0 w-12",
                log.type === 'stderr' && "text-red-400",
                log.type === 'exit' && "text-yellow-400",
                log.type === 'stdout' && "text-green-400",
              )}>
                {log.type === 'exit' ? 'EXIT' : log.type.toUpperCase()}
              </span>
              <span className="flex-1 break-all whitespace-pre-wrap">{log.data}</span>
              {log.type !== 'exit' && onKillProcess && (
                <button
                  onClick={() => onKillProcess(log.pid)}
                  className="opacity-0 group-hover:opacity-100 p-1 text-red-400 hover:bg-red-400/20 rounded transition-all"
                  title="Kill process"
                >
                  <Icons.Square className="w-3 h-3" />
                </button>
              )}
            </div>
          ))
        )}
      </div>

      {/* Running processes bar */}
      {runningCount > 0 && (
        <div className="flex items-center gap-2 px-4 py-1.5 bg-[#252526] border-t border-[#3c3c3c]">
          <span className="text-xs text-[#858585]">Running:</span>
          {Array.from(processes.values())
            .filter(p => p.running)
            .map(proc => (
              <div
                key={proc.pid}
                className="flex items-center gap-1 px-2 py-0.5 bg-[#3c3c3c] rounded text-xs"
              >
                <span className="w-2 h-2 bg-green-400 rounded-full animate-pulse" />
                <span className="text-[#cccccc]">PID {proc.pid}</span>
                {onKillProcess && (
                  <button
                    onClick={() => onKillProcess(proc.pid)}
                    className="ml-1 p-0.5 text-red-400 hover:bg-red-400/20 rounded"
                    title="Kill process"
                  >
                    <Icons.X className="w-3 h-3" />
                  </button>
                )}
              </div>
            ))}
        </div>
      )}
    </div>
  );
}
