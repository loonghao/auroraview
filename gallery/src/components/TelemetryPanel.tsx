/**
 * TelemetryPanel - Real-time performance monitoring and log viewer.
 *
 * Two tabs:
 *   - Metrics: counters, latency histograms, navigation state, errors
 *   - Logs: streaming auroraview.* Python log entries with level filters
 *
 * Polls the Python backend every 2s (configurable). Supports pause/resume.
 */

import { useEffect, useState, useCallback, useRef } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

// ---------- types ----------

interface TelemetryCounters {
  emit_count: number;
  eval_js_count: number;
  navigation_count: number;
  ipc_call_count: number;
  error_count: number;
}

interface TelemetryHistograms {
  load_time_avg_ms: number | null;
  load_time_p95_ms: number | null;
  eval_js_avg_ms: number | null;
  eval_js_p95_ms: number | null;
  ipc_latency_avg_ms: number | null;
  ipc_latency_p95_ms: number | null;
}

interface LogEntry {
  seq: number;
  ts: number;
  level: string;
  logger: string;
  msg: string;
}

interface TelemetrySnapshot {
  webview_id: string;
  uptime_s: number;
  counters: TelemetryCounters;
  histograms: TelemetryHistograms;
  last_url: string | null;
  last_error: string | null;
  otel_available: boolean;
  log_seq: number;
  logs?: LogEntry[];
}

interface TelemetryPanelProps {
  isOpen: boolean;
  onClose: () => void;
  getTelemetry: (opts?: {
    include_logs?: boolean;
    log_since?: number;
  }) => Promise<{
    ok: boolean;
    instances?: TelemetrySnapshot[];
    count?: number;
  }>;
}

type TabId = 'metrics' | 'logs';
type LogLevel = 'DEBUG' | 'INFO' | 'WARNING' | 'ERROR';

// ---------- helpers ----------

function formatMs(v: number | null): string {
  if (v === null || v === undefined) return '-';
  return v < 1 ? `${(v * 1000).toFixed(0)}us` : `${v.toFixed(1)}ms`;
}

function formatUptime(seconds: number): string {
  if (seconds < 60) return `${seconds.toFixed(0)}s`;
  if (seconds < 3600) return `${Math.floor(seconds / 60)}m ${Math.floor(seconds % 60)}s`;
  const h = Math.floor(seconds / 3600);
  const m = Math.floor((seconds % 3600) / 60);
  return `${h}h ${m}m`;
}

function formatTimestamp(ts: number): string {
  return new Date(ts * 1000).toLocaleTimeString('en-US', { hour12: false, fractionalSecondDigits: 3 } as Intl.DateTimeFormatOptions);
}

const LEVEL_STYLES: Record<string, { bg: string; text: string }> = {
  DEBUG: { bg: 'bg-[#333]', text: 'text-[#888]' },
  INFO: { bg: 'bg-blue-500/15', text: 'text-blue-400' },
  WARNING: { bg: 'bg-yellow-500/15', text: 'text-yellow-400' },
  ERROR: { bg: 'bg-red-500/15', text: 'text-red-400' },
  CRITICAL: { bg: 'bg-red-600/20', text: 'text-red-300' },
};

// ---------- sub-components ----------

function MetricCard({ label, value, sub, icon: Icon, color }: {
  label: string;
  value: string | number;
  sub?: string;
  icon: React.ComponentType<{ className?: string }>;
  color: string;
}) {
  return (
    <div className="flex items-center gap-2 px-3 py-2 bg-[#2d2d2d] rounded-lg border border-[#3c3c3c]">
      <div className={cn("p-1.5 rounded", color)}>
        <Icon className="w-3.5 h-3.5" />
      </div>
      <div className="min-w-0">
        <div className="text-xs text-[#858585] truncate">{label}</div>
        <div className="text-sm font-mono text-[#cccccc] leading-tight">{value}</div>
        {sub && <div className="text-[10px] text-[#666] leading-tight">{sub}</div>}
      </div>
    </div>
  );
}

function LatencyBar({ label, avg, p95 }: { label: string; avg: number | null; p95: number | null }) {
  const maxWidth = 100;
  const maxMs = 50;
  const avgPct = avg !== null ? Math.min((avg / maxMs) * maxWidth, maxWidth) : 0;
  const p95Pct = p95 !== null ? Math.min((p95 / maxMs) * maxWidth, maxWidth) : 0;

  const barColor = (v: number | null, warnMs: number, critMs: number) => {
    if (v === null) return 'bg-[#444]';
    if (v > critMs) return 'bg-red-500/80';
    if (v > warnMs) return 'bg-yellow-500/80';
    return 'bg-emerald-500/80';
  };

  return (
    <div className="flex items-center gap-2 text-xs">
      <span className="w-20 text-[#858585] text-right flex-shrink-0">{label}</span>
      <div className="flex-1 flex items-center gap-1.5">
        <div className="flex-1 h-3 bg-[#1e1e1e] rounded overflow-hidden flex items-center relative">
          <div
            className={cn("absolute h-full rounded opacity-40", barColor(p95, 20, 50))}
            style={{ width: `${p95Pct}%` }}
          />
          <div
            className={cn("absolute h-full rounded", barColor(avg, 10, 30))}
            style={{ width: `${avgPct}%` }}
          />
        </div>
        <span className="w-16 text-[#cccccc] font-mono text-right flex-shrink-0">{formatMs(avg)}</span>
        <span className="w-16 text-[#666] font-mono text-right flex-shrink-0">p95 {formatMs(p95)}</span>
      </div>
    </div>
  );
}

// ---------- main component ----------

export function TelemetryPanel({ isOpen, onClose, getTelemetry }: TelemetryPanelProps) {
  const [snapshots, setSnapshots] = useState<TelemetrySnapshot[]>([]);
  const [polling, setPolling] = useState(true);
  const [lastUpdate, setLastUpdate] = useState<number | null>(null);
  const [activeTab, setActiveTab] = useState<TabId>('metrics');
  const intervalRef = useRef<ReturnType<typeof setInterval> | null>(null);

  // Log state
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [logCursor, setLogCursor] = useState(0);
  const [levelFilter, setLevelFilter] = useState<Set<LogLevel>>(new Set(['INFO', 'WARNING', 'ERROR']));
  const [autoScroll, setAutoScroll] = useState(true);
  const logEndRef = useRef<HTMLDivElement | null>(null);

  const fetchData = useCallback(async () => {
    try {
      const includeLogsTab = activeTab === 'logs';
      const result = await getTelemetry({
        include_logs: includeLogsTab,
        log_since: includeLogsTab ? logCursor : 0,
      });
      if (result.ok && result.instances) {
        setSnapshots(result.instances);
        setLastUpdate(Date.now());

        // Merge new log entries (incremental)
        if (includeLogsTab && result.instances.length > 0) {
          const inst = result.instances[0];
          if (inst.logs && inst.logs.length > 0) {
            setLogs(prev => {
              const merged = [...prev, ...inst.logs!];
              // Keep last 2000 entries max
              return merged.length > 2000 ? merged.slice(-2000) : merged;
            });
            const maxSeq = inst.logs[inst.logs.length - 1].seq;
            setLogCursor(maxSeq);
          }
        }
      }
    } catch {
      // Silently ignore fetch errors
    }
  }, [getTelemetry, activeTab, logCursor]);

  // Auto-scroll logs
  useEffect(() => {
    if (autoScroll && activeTab === 'logs' && logEndRef.current) {
      logEndRef.current.scrollIntoView({ behavior: 'auto' });
    }
  }, [logs, autoScroll, activeTab]);

  // Poll when open
  useEffect(() => {
    if (!isOpen || !polling) {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
      return;
    }

    fetchData();
    intervalRef.current = setInterval(fetchData, 2000);
    return () => {
      if (intervalRef.current) {
        clearInterval(intervalRef.current);
        intervalRef.current = null;
      }
    };
  }, [isOpen, polling, fetchData]);

  if (!isOpen) return null;

  const instance = snapshots.length > 0 ? snapshots[0] : null;
  const counters = instance?.counters;
  const histograms = instance?.histograms;

  const toggleLevel = (level: LogLevel) => {
    setLevelFilter(prev => {
      const next = new Set(prev);
      if (next.has(level)) next.delete(level);
      else next.add(level);
      return next;
    });
  };

  const filteredLogs = logs.filter(l => levelFilter.has(l.level as LogLevel));

  return (
    <div className="fixed bottom-0 left-14 right-0 h-72 bg-[#1e1e1e] border-t border-[#3c3c3c] flex flex-col z-40">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-1.5 bg-[#252526] border-b border-[#3c3c3c] flex-shrink-0">
        <div className="flex items-center gap-3">
          {/* Tabs */}
          <button
            onClick={() => setActiveTab('metrics')}
            className={cn(
              "flex items-center gap-1.5 px-2 py-1 text-xs rounded transition-colors",
              activeTab === 'metrics'
                ? "bg-[#3c3c3c] text-[#cccccc]"
                : "text-[#858585] hover:text-[#cccccc]"
            )}
          >
            <Icons.Activity className="w-3.5 h-3.5" />
            Metrics
          </button>
          <button
            onClick={() => setActiveTab('logs')}
            className={cn(
              "flex items-center gap-1.5 px-2 py-1 text-xs rounded transition-colors",
              activeTab === 'logs'
                ? "bg-[#3c3c3c] text-[#cccccc]"
                : "text-[#858585] hover:text-[#cccccc]"
            )}
          >
            <Icons.ScrollText className="w-3.5 h-3.5" />
            Logs
            {logs.length > 0 && (
              <span className="px-1 text-[10px] bg-[#555] text-[#ccc] rounded">{logs.length}</span>
            )}
          </button>

          {/* Instance badge */}
          {instance && (
            <>
              <span className="px-1.5 py-0.5 text-xs bg-emerald-500/20 text-emerald-400 rounded">
                {instance.webview_id}
              </span>
              {instance.otel_available && (
                <span className="px-1.5 py-0.5 text-xs bg-blue-500/20 text-blue-400 rounded">
                  OTel
                </span>
              )}
              <span className="text-xs text-[#666]">
                uptime {formatUptime(instance.uptime_s)}
              </span>
            </>
          )}
        </div>

        <div className="flex items-center gap-2">
          {lastUpdate && (
            <span className="text-[10px] text-[#555]">
              {new Date(lastUpdate).toLocaleTimeString('en-US', { hour12: false })}
            </span>
          )}
          <button
            onClick={() => setPolling(!polling)}
            className={cn(
              "p-1 rounded transition-colors",
              polling ? "text-emerald-400" : "text-[#858585] hover:text-[#cccccc]"
            )}
            title={polling ? 'Live (click to pause)' : 'Paused (click to resume)'}
          >
            {polling ? <Icons.Radio className="w-3.5 h-3.5" /> : <Icons.Pause className="w-3.5 h-3.5" />}
          </button>
          <button
            onClick={fetchData}
            className="p-1 rounded text-[#858585] hover:text-[#cccccc] hover:bg-[#3c3c3c] transition-colors"
            title="Refresh now"
          >
            <Icons.RefreshCw className="w-3.5 h-3.5" />
          </button>
          <button
            onClick={onClose}
            className="p-1 rounded text-[#858585] hover:text-[#cccccc] hover:bg-[#3c3c3c] transition-colors"
          >
            <Icons.X className="w-3.5 h-3.5" />
          </button>
        </div>
      </div>

      {/* Body */}
      {activeTab === 'metrics' ? (
        /* METRICS TAB */
        !instance ? (
          <div className="flex-1 flex items-center justify-center text-[#858585]">
            <div className="text-center">
              <Icons.Activity className="w-8 h-8 mx-auto mb-2 opacity-50" />
              <p className="text-sm">No telemetry data yet.</p>
              <p className="text-xs mt-1">WebView auto-collects metrics when debug=True.</p>
            </div>
          </div>
        ) : (
          <div className="flex-1 overflow-auto p-3 space-y-3">
            {/* Counters row */}
            <div className="grid grid-cols-5 gap-2">
              <MetricCard
                label="Events"
                value={counters?.emit_count ?? 0}
                icon={Icons.Zap}
                color="bg-yellow-500/20 text-yellow-400"
              />
              <MetricCard
                label="JS Evals"
                value={counters?.eval_js_count ?? 0}
                sub={histograms?.eval_js_avg_ms !== null ? `avg ${formatMs(histograms?.eval_js_avg_ms ?? null)}` : undefined}
                icon={Icons.Code}
                color="bg-blue-500/20 text-blue-400"
              />
              <MetricCard
                label="IPC Calls"
                value={counters?.ipc_call_count ?? 0}
                sub={histograms?.ipc_latency_avg_ms !== null ? `avg ${formatMs(histograms?.ipc_latency_avg_ms ?? null)}` : undefined}
                icon={Icons.ArrowLeftRight}
                color="bg-purple-500/20 text-purple-400"
              />
              <MetricCard
                label="Navigations"
                value={counters?.navigation_count ?? 0}
                sub={histograms?.load_time_avg_ms !== null ? `avg ${formatMs(histograms?.load_time_avg_ms ?? null)}` : undefined}
                icon={Icons.Globe}
                color="bg-emerald-500/20 text-emerald-400"
              />
              <MetricCard
                label="Errors"
                value={counters?.error_count ?? 0}
                sub={instance.last_error ?? undefined}
                icon={Icons.AlertTriangle}
                color={counters?.error_count ? "bg-red-500/20 text-red-400" : "bg-[#333] text-[#666]"}
              />
            </div>

            {/* Latency bars */}
            <div className="bg-[#252526] rounded-lg border border-[#3c3c3c] p-3 space-y-1.5">
              <div className="text-xs text-[#858585] mb-2 flex items-center gap-1.5">
                <Icons.Timer className="w-3 h-3" />
                Latency Distribution
              </div>
              <LatencyBar
                label="Page Load"
                avg={histograms?.load_time_avg_ms ?? null}
                p95={histograms?.load_time_p95_ms ?? null}
              />
              <LatencyBar
                label="JS Eval"
                avg={histograms?.eval_js_avg_ms ?? null}
                p95={histograms?.eval_js_p95_ms ?? null}
              />
              <LatencyBar
                label="IPC"
                avg={histograms?.ipc_latency_avg_ms ?? null}
                p95={histograms?.ipc_latency_p95_ms ?? null}
              />
            </div>

            {/* Last URL */}
            {instance.last_url && (
              <div className="flex items-center gap-2 text-xs px-1">
                <Icons.Link className="w-3 h-3 text-[#666] flex-shrink-0" />
                <span className="text-[#858585]">Last URL:</span>
                <span className="text-[#cccccc] truncate font-mono">{instance.last_url}</span>
              </div>
            )}

            {/* Multi-instance */}
            {snapshots.length > 1 && (
              <div className="bg-[#252526] rounded-lg border border-[#3c3c3c] p-3">
                <div className="text-xs text-[#858585] mb-2">{snapshots.length} Active Instances</div>
                <div className="space-y-1">
                  {snapshots.map(s => (
                    <div key={s.webview_id} className="flex items-center justify-between text-xs">
                      <span className="text-[#cccccc] font-mono">{s.webview_id}</span>
                      <div className="flex items-center gap-3 text-[#858585]">
                        <span>{s.counters.emit_count} events</span>
                        <span>{s.counters.ipc_call_count} IPC</span>
                        <span>{formatUptime(s.uptime_s)}</span>
                        {s.counters.error_count > 0 && (
                          <span className="text-red-400">{s.counters.error_count} err</span>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            )}
          </div>
        )
      ) : (
        /* LOGS TAB */
        <div className="flex-1 flex flex-col min-h-0">
          {/* Log toolbar */}
          <div className="flex items-center gap-2 px-3 py-1.5 border-b border-[#333] flex-shrink-0">
            {(['DEBUG', 'INFO', 'WARNING', 'ERROR'] as LogLevel[]).map(level => {
              const style = LEVEL_STYLES[level] || LEVEL_STYLES.DEBUG;
              const active = levelFilter.has(level);
              return (
                <button
                  key={level}
                  onClick={() => toggleLevel(level)}
                  className={cn(
                    "px-2 py-0.5 text-[10px] rounded border transition-colors",
                    active
                      ? `${style.bg} ${style.text} border-transparent`
                      : "text-[#555] border-[#333] opacity-50"
                  )}
                >
                  {level}
                </button>
              );
            })}
            <div className="flex-1" />
            <button
              onClick={() => setAutoScroll(!autoScroll)}
              className={cn(
                "text-[10px] px-2 py-0.5 rounded transition-colors",
                autoScroll ? "text-emerald-400" : "text-[#666]"
              )}
              title={autoScroll ? 'Auto-scroll on' : 'Auto-scroll off'}
            >
              <Icons.ArrowDownToLine className="w-3 h-3 inline mr-1" />
              {autoScroll ? 'Follow' : 'Paused'}
            </button>
            <button
              onClick={() => { setLogs([]); setLogCursor(0); }}
              className="text-[10px] text-[#666] hover:text-[#ccc] px-2 py-0.5 rounded transition-colors"
              title="Clear logs"
            >
              <Icons.Trash2 className="w-3 h-3" />
            </button>
          </div>

          {/* Log entries */}
          <div className="flex-1 overflow-auto font-mono text-[11px] leading-5">
            {filteredLogs.length === 0 ? (
              <div className="flex items-center justify-center h-full text-[#555]">
                <div className="text-center">
                  <Icons.ScrollText className="w-6 h-6 mx-auto mb-1 opacity-40" />
                  <p>No log entries yet.</p>
                  <p className="text-[10px] mt-0.5">auroraview.* Python logs appear here in real-time.</p>
                </div>
              </div>
            ) : (
              <div className="px-2">
                {filteredLogs.map(entry => {
                  const style = LEVEL_STYLES[entry.level] || LEVEL_STYLES.DEBUG;
                  return (
                    <div
                      key={entry.seq}
                      className={cn("flex items-start gap-2 py-[1px] hover:bg-[#2a2a2a]", entry.level === 'ERROR' && 'bg-red-500/5')}
                    >
                      <span className="text-[#555] w-20 flex-shrink-0 text-right">{formatTimestamp(entry.ts)}</span>
                      <span className={cn("w-12 flex-shrink-0 text-center rounded px-1", style.bg, style.text)}>
                        {entry.level.slice(0, 4)}
                      </span>
                      <span className="text-[#666] w-40 flex-shrink-0 truncate">{entry.logger}</span>
                      <span className="text-[#cccccc] whitespace-pre-wrap break-all">{entry.msg}</span>
                    </div>
                  );
                })}
                <div ref={logEndRef} />
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}
