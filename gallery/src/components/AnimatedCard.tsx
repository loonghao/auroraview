/**
 * AnimatedCard - A card component with GSAP animations
 *
 * Features:
 * - Smooth hover effects with scale and glow
 * - Staggered entrance animation
 * - Magnetic cursor effect
 * - Click ripple effect
 */

import { useRef, useEffect, type ReactNode } from 'react';
import gsap from 'gsap';
import { cn } from '../lib/utils';

interface AnimatedCardProps {
  children: ReactNode;
  className?: string;
  onClick?: () => void;
  delay?: number;
  enableMagnetic?: boolean;
  enableRipple?: boolean;
  glowColor?: string;
}

export function AnimatedCard({
  children,
  className,
  onClick,
  delay = 0,
  enableMagnetic = false,
  enableRipple = true,
  glowColor = 'rgba(0, 212, 255, 0.3)',
}: AnimatedCardProps) {
  const cardRef = useRef<HTMLDivElement>(null);
  const glowRef = useRef<HTMLDivElement>(null);

  // Entrance animation
  useEffect(() => {
    const card = cardRef.current;
    if (!card) return;

    gsap.fromTo(
      card,
      {
        opacity: 0,
        y: 30,
        scale: 0.95,
      },
      {
        opacity: 1,
        y: 0,
        scale: 1,
        duration: 0.6,
        delay,
        ease: 'power3.out',
      }
    );
  }, [delay]);

  // Hover animation
  useEffect(() => {
    const card = cardRef.current;
    const glow = glowRef.current;
    if (!card) return;

    const handleMouseEnter = () => {
      gsap.to(card, {
        scale: 1.02,
        y: -4,
        duration: 0.3,
        ease: 'power2.out',
      });

      if (glow) {
        gsap.to(glow, {
          opacity: 1,
          duration: 0.3,
        });
      }
    };

    const handleMouseLeave = () => {
      gsap.to(card, {
        scale: 1,
        y: 0,
        x: 0,
        duration: 0.3,
        ease: 'power2.out',
      });

      if (glow) {
        gsap.to(glow, {
          opacity: 0,
          duration: 0.3,
        });
      }
    };

    card.addEventListener('mouseenter', handleMouseEnter);
    card.addEventListener('mouseleave', handleMouseLeave);

    return () => {
      card.removeEventListener('mouseenter', handleMouseEnter);
      card.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, []);

  // Magnetic effect
  useEffect(() => {
    if (!enableMagnetic) return;

    const card = cardRef.current;
    if (!card) return;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = card.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      const deltaX = (e.clientX - centerX) * 0.1;
      const deltaY = (e.clientY - centerY) * 0.1;

      gsap.to(card, {
        x: deltaX,
        y: deltaY - 4,
        duration: 0.3,
        ease: 'power2.out',
      });
    };

    card.addEventListener('mousemove', handleMouseMove);

    return () => {
      card.removeEventListener('mousemove', handleMouseMove);
    };
  }, [enableMagnetic]);

  // Ripple effect
  const handleClick = (e: React.MouseEvent) => {
    if (enableRipple && cardRef.current) {
      const rect = cardRef.current.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      const ripple = document.createElement('span');
      ripple.style.cssText = `
        position: absolute;
        border-radius: 50%;
        background: ${glowColor};
        pointer-events: none;
        transform: scale(0);
        left: ${x}px;
        top: ${y}px;
        width: 10px;
        height: 10px;
        margin-left: -5px;
        margin-top: -5px;
        z-index: 10;
      `;

      cardRef.current.appendChild(ripple);

      gsap.to(ripple, {
        scale: 30,
        opacity: 0,
        duration: 0.6,
        ease: 'power2.out',
        onComplete: () => ripple.remove(),
      });
    }

    onClick?.();
  };

  return (
    <div
      ref={cardRef}
      className={cn(
        'relative overflow-hidden cursor-pointer',
        'bg-card border border-border rounded-xl',
        'transition-colors',
        className
      )}
      onClick={handleClick}
      style={{ willChange: 'transform, opacity' }}
    >
      {/* Glow effect overlay */}
      <div
        ref={glowRef}
        className="absolute inset-0 pointer-events-none opacity-0 rounded-xl"
        style={{
          background: `radial-gradient(circle at center, ${glowColor} 0%, transparent 70%)`,
          filter: 'blur(20px)',
        }}
      />

      {/* Content */}
      <div className="relative z-[1]">{children}</div>
    </div>
  );
}

/**
 * AnimatedCardGrid - A grid container that staggers card animations
 */
interface AnimatedCardGridProps {
  children: ReactNode;
  className?: string;
  staggerDelay?: number;
}

export function AnimatedCardGrid({
  children,
  className,
  staggerDelay = 0.08,
}: AnimatedCardGridProps) {
  const gridRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const grid = gridRef.current;
    if (!grid) return;

    const cards = grid.querySelectorAll('[data-animated-card]');

    gsap.fromTo(
      cards,
      {
        opacity: 0,
        y: 40,
        scale: 0.9,
      },
      {
        opacity: 1,
        y: 0,
        scale: 1,
        duration: 0.6,
        stagger: staggerDelay,
        ease: 'power3.out',
      }
    );
  }, [staggerDelay]);

  return (
    <div ref={gridRef} className={className}>
      {children}
    </div>
  );
}
