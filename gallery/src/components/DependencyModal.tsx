/**
 * Dependency Installation Modal
 * 
 * Shows when a sample has missing dependencies. Provides:
 * - List of missing packages
 * - Install button to trigger pip install
 * - Real-time progress display with pip output
 * - Success/error status with auto-close on completion
 * 
 * Style: White Minimalism (Light theme, high contrast, professional)
 */

import { useState, useEffect, useRef } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';
import type {
  DependencyProgress,
  DependencyComplete,
  DependencyError,
} from '../hooks/useAuroraView';
import type { EventHandler } from '@auroraview/sdk';

interface DependencyModalProps {
  isOpen: boolean;
  sampleId: string;
  sampleTitle: string;
  missing: string[];
  onInstall: (sampleId: string) => Promise<void>;
  onCancel: () => void;
  onCancelInstall?: () => Promise<any>;
  onComplete: () => void;
}

type Phase = 'pending' | 'installing' | 'complete' | 'error' | 'cancelled';

export function DependencyModal({
  isOpen,
  sampleId,
  sampleTitle,
  missing,
  onInstall,
  onCancel,
  onCancelInstall,
  onComplete,
}: DependencyModalProps) {
  const [phase, setPhase] = useState<Phase>('pending');
  const [currentPackage, setCurrentPackage] = useState('');
  const [progress, setProgress] = useState(0);
  const [logs, setLogs] = useState<string[]>([]);
  const logsRef = useRef<HTMLDivElement>(null);

  // Reset state when modal opens
  useEffect(() => {
    if (isOpen) {
      setPhase('pending');
      setCurrentPackage('');
      setProgress(0);
      setLogs([]);
    }
  }, [isOpen, sampleId]);

  // Subscribe to dependency events
  useEffect(() => {
    if (!isOpen) return;

    console.log(`[DependencyModal] Setting up event listeners for sample_id=${sampleId}`);

    const handleStart: EventHandler<unknown> = (data: unknown) => {
      const startData = data as { sample_id: string; packages: string[]; total: number };
      console.log(`[DependencyModal] dep:start event received:`, startData);
      
      if (startData.sample_id !== sampleId) {
        console.log(`[DependencyModal] Ignoring dep:start for different sample: ${startData.sample_id} !== ${sampleId}`);
        return;
      }
      
      console.log(`[DependencyModal] Processing dep:start for ${sampleId}`);
      setPhase('installing');
      setProgress(0);
      setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] 🚀 Starting installation of ${startData.total} package(s)...`]);
    };

    const handleProgress: EventHandler<unknown> = (data: unknown) => {
      const progressData = data as DependencyProgress;
      console.log(`[DependencyModal] dep:progress event received:`, progressData);
      
      if (progressData.sample_id !== sampleId) {
        console.log(`[DependencyModal] Ignoring dep:progress for different sample: ${progressData.sample_id} !== ${sampleId}`);
        return;
      }

      console.log(`[DependencyModal] Processing dep:progress for ${sampleId}, phase=${progressData.phase}`);
      
      if (phase !== 'cancelled') {
        setPhase('installing');
      }
      
      if (progressData.package) {
        console.log(`[DependencyModal] Setting current package: ${progressData.package}`);
        setCurrentPackage(progressData.package);
      }
      if (progressData.index !== undefined && progressData.total) {
        const newProgress = ((progressData.index + 1) / progressData.total) * 100;
        console.log(`[DependencyModal] Setting progress: ${newProgress}% (${progressData.index + 1}/${progressData.total})`);
        setProgress(newProgress);
      }
      if (progressData.line) {
        console.log(`[DependencyModal] Adding log line: ${progressData.line}`);
        setLogs(prev => [...prev, progressData.line!].slice(-100));
      }
      if (progressData.message) {
        console.log(`[DependencyModal] Adding log message: ${progressData.message}`);
        setLogs(prev => [...prev, progressData.message!].slice(-100));
      }
    };

    const handleComplete: EventHandler<unknown> = (data: unknown) => {
      const completeData = data as DependencyComplete;
      console.log(`[DependencyModal] dep:complete event received:`, completeData);
      
      if (completeData.sample_id !== sampleId) {
        console.log(`[DependencyModal] Ignoring dep:complete for different sample: ${completeData.sample_id} !== ${sampleId}`);
        return;
      }
      
      console.log(`[DependencyModal] Processing dep:complete for ${sampleId}`);
      setPhase('complete');
      setProgress(100);
      setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ✅ ${completeData.message}`]);
      // Auto-close after success
      setTimeout(() => {
        console.log(`[DependencyModal] Auto-closing modal after success`);
        onComplete();
      }, 1500);
    };

    const handleError: EventHandler<unknown> = (data: unknown) => {
      const errorData = data as (DependencyError & { cancelled?: boolean });
      console.log(`[DependencyModal] dep:error event received:`, errorData);
      
      if (errorData.sample_id !== sampleId) {
        console.log(`[DependencyModal] Ignoring dep:error for different sample: ${errorData.sample_id} !== ${sampleId}`);
        return;
      }
      
      console.log(`[DependencyModal] Processing dep:error for ${sampleId}, cancelled=${errorData.cancelled}`);
      
      if (errorData.cancelled) {
        setPhase('cancelled');
        setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ⏹️ Installation cancelled.`]);
      } else {
        setPhase('error');
        setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ❌ Error: ${errorData.error}`]);
      }
    };

    // Subscribe to events
    if (window.auroraview?.on) {
      console.log(`[DependencyModal] Subscribing to events`);
      window.auroraview.on('dep:start', handleStart);
      window.auroraview.on('dep:progress', handleProgress);
      window.auroraview.on('dep:complete', handleComplete);
      window.auroraview.on('dep:error', handleError);

      return () => {
        console.log(`[DependencyModal] Unsubscribing from events`);
        if (window.auroraview?.off) {
          window.auroraview.off('dep:start', handleStart);
          window.auroraview.off('dep:progress', handleProgress);
          window.auroraview.off('dep:complete', handleComplete);
          window.auroraview.off('dep:error', handleError);
        }
      };
    } else {
      console.error(`[DependencyModal] window.auroraview.on is not available!`);
    }
  }, [isOpen, sampleId, onComplete, phase]);

  // Auto-scroll logs
  useEffect(() => {
    if (logsRef.current) {
      logsRef.current.scrollTop = logsRef.current.scrollHeight;
    }
  }, [logs]);

  const handleInstall = async () => {
    console.log(`[DependencyModal] Starting installation for sample_id=${sampleId}`);
    setPhase('installing');
    setLogs([`[${new Date().toLocaleTimeString()}] Starting installation...`]);
    
    try {
      console.log(`[DependencyModal] Calling onInstall(${sampleId})`);
      await onInstall(sampleId);
      console.log(`[DependencyModal] onInstall completed successfully`);
    } catch (error) {
      console.error(`[DependencyModal] onInstall failed:`, error);
      setPhase('error');
      setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ❌ Failed to start installation: ${error}`]);
    }
  };

  const handleCancelInstallation = async () => {
    console.log(`[DependencyModal] Cancelling installation`);
    if (onCancelInstall) {
      try {
        console.log(`[DependencyModal] Calling onCancelInstall()`);
        await onCancelInstall();
        console.log(`[DependencyModal] onCancelInstall completed`);
        // Phase will be set to 'cancelled' via event
      } catch (error) {
        console.error(`[DependencyModal] onCancelInstall failed:`, error);
        setLogs(prev => [...prev, `[${new Date().toLocaleTimeString()}] ❌ Failed to cancel: ${error}`]);
      }
    }
  };

  if (!isOpen) return null;

  return (
    <div className="fixed inset-0 bg-slate-900/40 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className={cn(
        "bg-white border border-slate-200 rounded-xl shadow-xl w-full max-w-lg overflow-hidden",
        "animate-in fade-in zoom-in-95 duration-200"
      )}>
        {/* Header */}
        <div className="flex items-center justify-between p-6 border-b border-slate-100">
          <div className="flex items-center gap-4">
            <div className={cn(
              "w-10 h-10 rounded-full flex items-center justify-center transition-colors",
              phase === 'error' 
                ? "bg-rose-50 text-rose-600" 
                : phase === 'complete'
                ? "bg-emerald-50 text-emerald-600"
                : phase === 'cancelled'
                ? "bg-slate-100 text-slate-600"
                : "bg-indigo-50 text-indigo-600"
            )}>
              {phase === 'installing' ? (
                <Icons.Loader2 className="w-5 h-5 animate-spin" />
              ) : phase === 'complete' ? (
                <Icons.Check className="w-5 h-5" />
              ) : phase === 'error' ? (
                <Icons.AlertCircle className="w-5 h-5" />
              ) : phase === 'cancelled' ? (
                <Icons.StopCircle className="w-5 h-5" />
              ) : (
                <Icons.Box className="w-5 h-5" />
              )}
            </div>
            <div>
              <h3 className="font-bold text-slate-900 leading-none">Install Dependencies</h3>
              <p className="text-sm text-slate-500 mt-1.5">{sampleTitle}</p>
            </div>
          </div>
          <button
            onClick={onCancel}
            disabled={phase === 'installing'}
            className="p-2 text-slate-400 hover:text-slate-600 hover:bg-slate-50 rounded-lg transition-all disabled:opacity-30"
          >
            <Icons.X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-6 space-y-5">
          {phase === 'pending' && (
            <>
              <p className="text-sm text-slate-600">
                To run this demo, the following Python packages are required:
              </p>
              <div className="flex flex-wrap gap-2">
                {missing.map((pkg) => (
                  <span
                    key={pkg}
                    className="px-3 py-1 bg-slate-50 text-slate-700 border border-slate-200 rounded-md text-sm font-medium font-mono"
                  >
                    {pkg}
                  </span>
                ))}
              </div>
            </>
          )}

          {(phase === 'installing' || phase === 'cancelled' || phase === 'error' || phase === 'complete') && (
            <>
              {/* Progress */}
              <div className="space-y-2.5">
                <div className="flex justify-between items-end">
                  <span className="text-sm font-medium text-slate-700">
                    {phase === 'installing' ? (
                      <>Installing <span className="text-indigo-600 font-bold">{currentPackage}</span></>
                    ) : phase === 'complete' ? (
                      <span className="text-emerald-600">All Done</span>
                    ) : phase === 'cancelled' ? (
                      <span className="text-slate-500">Cancelled</span>
                    ) : (
                      <span className="text-rose-600">Installation Failed</span>
                    )}
                  </span>
                  <span className="text-xs font-bold text-slate-400 tracking-wider">
                    {Math.round(progress)}%
                  </span>
                </div>
                <div className="h-1.5 bg-slate-100 rounded-full overflow-hidden">
                  <div
                    className={cn(
                      "h-full transition-all duration-500 rounded-full",
                      phase === 'error' ? "bg-rose-500" : 
                      phase === 'complete' ? "bg-emerald-500" : 
                      phase === 'cancelled' ? "bg-slate-300" : "bg-indigo-600"
                    )}
                    style={{ width: `${progress}%` }}
                  />
                </div>
              </div>

              {/* Logs */}
              <div className="relative group">
                <div
                  ref={logsRef}
                  className="h-48 rounded-lg p-4 font-mono text-[11px] overflow-auto bg-slate-50 border border-slate-100 text-slate-500 leading-relaxed shadow-inner"
                >
                  {logs.length === 0 ? (
                    <div className="text-slate-300 italic">Waiting for logs...</div>
                  ) : (
                    logs.map((line, i) => (
                      <div key={i} className={cn(
                        "mb-0.5",
                        line.includes('Error') || line.includes('❌') ? "text-rose-500" : 
                        line.includes('✓') || line.includes('✅') ? "text-emerald-600" : ""
                      )}>
                        {line}
                      </div>
                    ))
                  )}
                </div>
                <div className="absolute top-2 right-2 opacity-0 group-hover:opacity-100 transition-opacity">
                   <div className="bg-white/80 backdrop-blur px-2 py-1 rounded text-[10px] text-slate-400 border border-slate-200">
                      Terminal Output
                   </div>
                </div>
              </div>
            </>
          )}
        </div>

        {/* Footer */}
        <div className="px-6 py-5 bg-slate-50/50 border-t border-slate-100 flex justify-end gap-3">
          {phase === 'pending' && (
            <>
              <button
                onClick={onCancel}
                className="px-4 py-2 text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors"
              >
                Not Now
              </button>
              <button
                onClick={handleInstall}
                className="px-6 py-2 text-sm font-bold bg-slate-900 text-white rounded-lg hover:bg-slate-800 transition-all shadow-sm hover:shadow-md active:scale-95 flex items-center gap-2"
              >
                <Icons.Download className="w-4 h-4" />
                Install Now
              </button>
            </>
          )}

          {phase === 'installing' && (
            <button
              onClick={handleCancelInstallation}
              className="px-6 py-2 text-sm font-bold border border-slate-200 text-slate-600 rounded-lg hover:bg-rose-50 hover:text-rose-600 hover:border-rose-200 transition-all flex items-center gap-2"
            >
              <Icons.StopCircle className="w-4 h-4" />
              Cancel Installation
            </button>
          )}

          {(phase === 'error' || phase === 'cancelled') && (
            <>
              <button
                onClick={onCancel}
                className="px-4 py-2 text-sm font-medium text-slate-600 hover:text-slate-900 transition-colors"
              >
                Close
              </button>
              <button
                onClick={handleInstall}
                className="px-6 py-2 text-sm font-bold bg-indigo-600 text-white rounded-lg hover:bg-indigo-700 transition-all shadow-sm flex items-center gap-2"
              >
                <Icons.RefreshCw className="w-4 h-4" />
                Try Again
              </button>
            </>
          )}

          {phase === 'complete' && (
            <div className="flex items-center gap-2 text-emerald-600 font-bold text-sm">
              <Icons.CheckCircle2 className="w-4 h-4" />
              Launching Demo...
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
