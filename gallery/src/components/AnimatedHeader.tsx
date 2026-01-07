/**
 * AnimatedHeader - Header with GSAP text and element animations
 *
 * Features:
 * - Typewriter effect for title
 * - Floating animation for logo
 * - Staggered entrance for elements
 * - Particle background effect
 */

import { useRef, useEffect, type ReactNode } from 'react';
import gsap from 'gsap';

interface AnimatedHeaderProps {
  title: string;
  subtitle?: string;
  children?: ReactNode;
}

export function AnimatedHeader({ title, subtitle, children }: AnimatedHeaderProps) {
  const headerRef = useRef<HTMLElement>(null);
  const titleRef = useRef<HTMLHeadingElement>(null);
  const subtitleRef = useRef<HTMLParagraphElement>(null);
  const childrenRef = useRef<HTMLDivElement>(null);
  const particlesRef = useRef<HTMLDivElement>(null);

  // Title animation with character split
  useEffect(() => {
    const titleEl = titleRef.current;
    if (!titleEl) return;

    // Split text into characters
    const chars = title.split('');
    titleEl.innerHTML = chars
      .map((char) => `<span class="inline-block">${char === ' ' ? '&nbsp;' : char}</span>`)
      .join('');

    const charElements = titleEl.querySelectorAll('span');

    gsap.fromTo(
      charElements,
      {
        opacity: 0,
        y: 50,
        rotateX: -90,
      },
      {
        opacity: 1,
        y: 0,
        rotateX: 0,
        duration: 0.8,
        stagger: 0.03,
        ease: 'back.out(1.7)',
      }
    );
  }, [title]);

  // Subtitle animation
  useEffect(() => {
    const subtitleEl = subtitleRef.current;
    if (!subtitleEl || !subtitle) return;

    gsap.fromTo(
      subtitleEl,
      {
        opacity: 0,
        y: 20,
      },
      {
        opacity: 1,
        y: 0,
        duration: 0.6,
        delay: 0.5,
        ease: 'power3.out',
      }
    );
  }, [subtitle]);

  // Children animation
  useEffect(() => {
    const childrenEl = childrenRef.current;
    if (!childrenEl) return;

    gsap.fromTo(
      childrenEl,
      {
        opacity: 0,
        x: 30,
      },
      {
        opacity: 1,
        x: 0,
        duration: 0.6,
        delay: 0.3,
        ease: 'power3.out',
      }
    );
  }, []);

  // Particle effect
  useEffect(() => {
    const container = particlesRef.current;
    if (!container) return;

    const particles: HTMLDivElement[] = [];
    const particleCount = 25;

    for (let i = 0; i < particleCount; i++) {
      const particle = document.createElement('div');
      particle.className = 'absolute rounded-full pointer-events-none';
      // Use primary color (violet/purple) for particles
      const hue = 270 + Math.random() * 30; // Purple range
      particle.style.cssText = `
        width: ${Math.random() * 4 + 2}px;
        height: ${Math.random() * 4 + 2}px;
        background: hsl(${hue}, 70%, ${50 + Math.random() * 20}%);
        opacity: ${Math.random() * 0.4 + 0.1};
        left: ${Math.random() * 100}%;
        top: ${Math.random() * 100}%;
        filter: blur(${Math.random() * 1}px);
      `;
      container.appendChild(particle);
      particles.push(particle);

      // Animate each particle
      gsap.to(particle, {
        y: -100 - Math.random() * 100,
        x: (Math.random() - 0.5) * 100,
        opacity: 0,
        duration: 3 + Math.random() * 2,
        delay: Math.random() * 2,
        repeat: -1,
        ease: 'power1.out',
      });
    }

    return () => {
      particles.forEach((p) => p.remove());
    };
  }, []);

  return (
    <header ref={headerRef} className="mb-10 flex items-start justify-between relative">
      {/* Particle container */}
      <div
        ref={particlesRef}
        className="absolute inset-0 overflow-hidden pointer-events-none"
        style={{ zIndex: 0 }}
      />

      <div className="relative z-[1]">
        <h1
          ref={titleRef}
          className="text-4xl font-bold mb-3 bg-gradient-to-r from-foreground via-primary to-purple-400 bg-clip-text text-transparent"
          style={{ perspective: '1000px' }}
        >
          {title}
        </h1>
        {subtitle && (
          <p ref={subtitleRef} className="text-muted-foreground text-lg">
            {subtitle}
          </p>
        )}
      </div>

      <div ref={childrenRef} className="relative z-[1]">
        {children}
      </div>
    </header>
  );
}

/**
 * AnimatedTitle - Standalone animated title component
 */
interface AnimatedTitleProps {
  text: string;
  className?: string;
  delay?: number;
}

export function AnimatedTitle({ text, className, delay = 0 }: AnimatedTitleProps) {
  const titleRef = useRef<HTMLHeadingElement>(null);

  useEffect(() => {
    const titleEl = titleRef.current;
    if (!titleEl) return;

    // Split text into words
    const words = text.split(' ');
    titleEl.innerHTML = words
      .map((word) => `<span class="inline-block mr-2">${word}</span>`)
      .join('');

    const wordElements = titleEl.querySelectorAll('span');

    gsap.fromTo(
      wordElements,
      {
        opacity: 0,
        y: 30,
        scale: 0.8,
      },
      {
        opacity: 1,
        y: 0,
        scale: 1,
        duration: 0.5,
        stagger: 0.1,
        delay,
        ease: 'power3.out',
      }
    );
  }, [text, delay]);

  return (
    <h2 ref={titleRef} className={className}>
      {text}
    </h2>
  );
}

/**
 * GlowingText - Text with animated glow effect
 */
interface GlowingTextProps {
  children: ReactNode;
  className?: string;
  glowColor?: string;
}

export function GlowingText({
  children,
  className,
  glowColor = 'rgba(167, 139, 250, 0.5)',
}: GlowingTextProps) {
  const textRef = useRef<HTMLSpanElement>(null);

  useEffect(() => {
    const text = textRef.current;
    if (!text) return;

    gsap.to(text, {
      textShadow: `0 0 20px ${glowColor}, 0 0 40px ${glowColor}`,
      duration: 1.5,
      ease: 'sine.inOut',
      yoyo: true,
      repeat: -1,
    });
  }, [glowColor]);

  return (
    <span ref={textRef} className={className}>
      {children}
    </span>
  );
}
