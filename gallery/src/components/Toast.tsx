import { useEffect } from 'react';
import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

interface ToastProps {
  message: string;
  isVisible: boolean;
  onHide: () => void;
  type?: 'success' | 'error';
}

export function Toast({ message, isVisible, onHide, type = 'success' }: ToastProps) {
  useEffect(() => {
    if (isVisible) {
      const timer = setTimeout(onHide, 2000);
      return () => clearTimeout(timer);
    }
  }, [isVisible, onHide]);

  return (
    <div
      className={cn(
        "fixed bottom-5 right-5 bg-secondary border rounded-lg px-5 py-3",
        "flex items-center gap-2.5 z-[1001] transition-all duration-300",
        type === 'success' ? "border-green-500 text-green-500" : "border-red-500 text-red-500",
        isVisible ? "translate-y-0 opacity-100" : "translate-y-24 opacity-0"
      )}
    >
      {type === 'success' ? (
        <Icons.Check className="w-4 h-4" />
      ) : (
        <Icons.X className="w-4 h-4" />
      )}
      <span>{message}</span>
    </div>
  );
}
