/**
 * PageTransition - Smooth page transition wrapper with GSAP
 *
 * Features:
 * - Fade and slide transitions
 * - Staggered content reveal
 * - Loading state animation
 */

import { useRef, useEffect, type ReactNode } from 'react';
import gsap from 'gsap';

interface PageTransitionProps {
  children: ReactNode;
  className?: string;
  transitionKey?: string | number;
}

export function PageTransition({ children, className, transitionKey }: PageTransitionProps) {
  const containerRef = useRef<HTMLDivElement>(null);
  const previousKeyRef = useRef(transitionKey);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    // Only animate if the key changed
    if (previousKeyRef.current !== transitionKey) {
      // Exit animation
      gsap.fromTo(
        container,
        { opacity: 1, y: 0 },
        {
          opacity: 0,
          y: -20,
          duration: 0.2,
          ease: 'power2.in',
          onComplete: () => {
            // Enter animation
            gsap.fromTo(
              container,
              { opacity: 0, y: 20 },
              {
                opacity: 1,
                y: 0,
                duration: 0.4,
                ease: 'power3.out',
              }
            );
          },
        }
      );
    } else {
      // Initial entrance
      gsap.fromTo(
        container,
        { opacity: 0, y: 30 },
        {
          opacity: 1,
          y: 0,
          duration: 0.6,
          ease: 'power3.out',
        }
      );
    }

    previousKeyRef.current = transitionKey;
  }, [transitionKey]);

  return (
    <div ref={containerRef} className={className}>
      {children}
    </div>
  );
}

/**
 * StaggeredList - Animates list items with stagger effect
 */
interface StaggeredListProps {
  children: ReactNode;
  className?: string;
  stagger?: number;
  delay?: number;
}

export function StaggeredList({
  children,
  className,
  stagger = 0.08,
  delay = 0,
}: StaggeredListProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const container = containerRef.current;
    if (!container) return;

    const items = container.children;

    gsap.fromTo(
      items,
      {
        opacity: 0,
        y: 30,
        scale: 0.95,
      },
      {
        opacity: 1,
        y: 0,
        scale: 1,
        duration: 0.5,
        stagger,
        delay,
        ease: 'power3.out',
      }
    );
  }, [stagger, delay]);

  return (
    <div ref={containerRef} className={className}>
      {children}
    </div>
  );
}

/**
 * LoadingSpinner - Animated loading spinner
 */
interface LoadingSpinnerProps {
  size?: number;
  color?: string;
}

export function LoadingSpinner({ size = 40, color = '#00d4ff' }: LoadingSpinnerProps) {
  const spinnerRef = useRef<HTMLDivElement>(null);
  const dotsRef = useRef<HTMLDivElement[]>([]);

  useEffect(() => {
    const dots = dotsRef.current;

    dots.forEach((dot, i) => {
      gsap.to(dot, {
        scale: 1.5,
        opacity: 1,
        duration: 0.4,
        delay: i * 0.15,
        repeat: -1,
        yoyo: true,
        ease: 'power2.inOut',
      });
    });
  }, []);

  return (
    <div
      ref={spinnerRef}
      className="flex items-center justify-center gap-2"
      style={{ width: size, height: size }}
    >
      {[0, 1, 2].map((i) => (
        <div
          key={i}
          ref={(el) => {
            if (el) dotsRef.current[i] = el;
          }}
          className="rounded-full"
          style={{
            width: size / 5,
            height: size / 5,
            backgroundColor: color,
            opacity: 0.5,
          }}
        />
      ))}
    </div>
  );
}

/**
 * ProgressBar - Animated progress bar
 */
interface ProgressBarProps {
  progress: number;
  className?: string;
  color?: string;
  height?: number;
}

export function ProgressBar({
  progress,
  className,
  color = '#00d4ff',
  height = 4,
}: ProgressBarProps) {
  const barRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const bar = barRef.current;
    if (!bar) return;

    gsap.to(bar, {
      width: `${progress}%`,
      duration: 0.5,
      ease: 'power2.out',
    });
  }, [progress]);

  return (
    <div
      className={`bg-muted rounded-full overflow-hidden ${className}`}
      style={{ height }}
    >
      <div
        ref={barRef}
        className="h-full rounded-full"
        style={{
          backgroundColor: color,
          width: 0,
          boxShadow: `0 0 10px ${color}`,
        }}
      />
    </div>
  );
}

/**
 * SkeletonLoader - Animated skeleton loading placeholder
 */
interface SkeletonLoaderProps {
  width?: string | number;
  height?: string | number;
  className?: string;
  rounded?: boolean;
}

export function SkeletonLoader({
  width = '100%',
  height = 20,
  className,
  rounded = false,
}: SkeletonLoaderProps) {
  const skeletonRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const skeleton = skeletonRef.current;
    if (!skeleton) return;

    gsap.to(skeleton, {
      backgroundPosition: '200% 0',
      duration: 1.5,
      repeat: -1,
      ease: 'none',
    });
  }, []);

  return (
    <div
      ref={skeletonRef}
      className={`${rounded ? 'rounded-full' : 'rounded'} ${className}`}
      style={{
        width,
        height,
        background: 'linear-gradient(90deg, rgba(255,255,255,0.05) 25%, rgba(255,255,255,0.1) 50%, rgba(255,255,255,0.05) 75%)',
        backgroundSize: '200% 100%',
      }}
    />
  );
}
