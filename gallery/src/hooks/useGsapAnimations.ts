/**
 * GSAP Animation Hooks for AuroraView Gallery
 *
 * Provides reusable animation hooks using GSAP for smooth,
 * professional animations throughout the gallery.
 */

import { useEffect, useRef, useCallback } from 'react';
import gsap from 'gsap';

/**
 * Hook for staggered entrance animations on a list of elements
 */
export function useStaggerEntrance(
  selector: string,
  options: {
    duration?: number;
    stagger?: number;
    delay?: number;
    y?: number;
    opacity?: number;
    ease?: string;
  } = {}
) {
  const containerRef = useRef<HTMLDivElement>(null);
  const animationRef = useRef<gsap.core.Tween | null>(null);

  const {
    duration = 0.6,
    stagger = 0.08,
    delay = 0,
    y = 30,
    opacity = 0,
    ease = 'power3.out',
  } = options;

  useEffect(() => {
    if (!containerRef.current) return;

    const elements = containerRef.current.querySelectorAll(selector);
    if (elements.length === 0) return;

    // Set initial state
    gsap.set(elements, { y, opacity });

    // Animate
    animationRef.current = gsap.to(elements, {
      y: 0,
      opacity: 1,
      duration,
      stagger,
      delay,
      ease,
    });

    return () => {
      animationRef.current?.kill();
    };
  }, [selector, duration, stagger, delay, y, opacity, ease]);

  return containerRef;
}

/**
 * Hook for hover scale animation
 */
export function useHoverScale(scale: number = 1.05) {
  const elementRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    const handleMouseEnter = () => {
      gsap.to(element, {
        scale,
        duration: 0.3,
        ease: 'power2.out',
      });
    };

    const handleMouseLeave = () => {
      gsap.to(element, {
        scale: 1,
        duration: 0.3,
        ease: 'power2.out',
      });
    };

    element.addEventListener('mouseenter', handleMouseEnter);
    element.addEventListener('mouseleave', handleMouseLeave);

    return () => {
      element.removeEventListener('mouseenter', handleMouseEnter);
      element.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, [scale]);

  return elementRef;
}

/**
 * Hook for magnetic cursor effect
 */
export function useMagneticEffect(strength: number = 0.3) {
  const elementRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    const handleMouseMove = (e: MouseEvent) => {
      const rect = element.getBoundingClientRect();
      const centerX = rect.left + rect.width / 2;
      const centerY = rect.top + rect.height / 2;

      const deltaX = (e.clientX - centerX) * strength;
      const deltaY = (e.clientY - centerY) * strength;

      gsap.to(element, {
        x: deltaX,
        y: deltaY,
        duration: 0.3,
        ease: 'power2.out',
      });
    };

    const handleMouseLeave = () => {
      gsap.to(element, {
        x: 0,
        y: 0,
        duration: 0.5,
        ease: 'elastic.out(1, 0.5)',
      });
    };

    element.addEventListener('mousemove', handleMouseMove);
    element.addEventListener('mouseleave', handleMouseLeave);

    return () => {
      element.removeEventListener('mousemove', handleMouseMove);
      element.removeEventListener('mouseleave', handleMouseLeave);
    };
  }, [strength]);

  return elementRef;
}

/**
 * Hook for page transition animations
 */
export function usePageTransition() {
  const containerRef = useRef<HTMLDivElement>(null);

  const animateIn = useCallback(() => {
    if (!containerRef.current) return;

    gsap.fromTo(
      containerRef.current,
      {
        opacity: 0,
        y: 20,
      },
      {
        opacity: 1,
        y: 0,
        duration: 0.5,
        ease: 'power3.out',
      }
    );
  }, []);

  const animateOut = useCallback(() => {
    return new Promise<void>((resolve) => {
      if (!containerRef.current) {
        resolve();
        return;
      }

      gsap.to(containerRef.current, {
        opacity: 0,
        y: -20,
        duration: 0.3,
        ease: 'power3.in',
        onComplete: resolve,
      });
    });
  }, []);

  useEffect(() => {
    animateIn();
  }, [animateIn]);

  return { containerRef, animateIn, animateOut };
}

/**
 * Hook for scroll-triggered animations
 */
export function useScrollReveal(
  options: {
    threshold?: number;
    rootMargin?: string;
    once?: boolean;
  } = {}
) {
  const elementRef = useRef<HTMLElement>(null);
  const hasAnimated = useRef(false);

  const { threshold = 0.2, rootMargin = '0px', once = true } = options;

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    // Set initial state
    gsap.set(element, { opacity: 0, y: 40 });

    const observer = new IntersectionObserver(
      (entries) => {
        entries.forEach((entry) => {
          if (entry.isIntersecting) {
            if (once && hasAnimated.current) return;

            gsap.to(element, {
              opacity: 1,
              y: 0,
              duration: 0.8,
              ease: 'power3.out',
            });

            hasAnimated.current = true;

            if (once) {
              observer.unobserve(element);
            }
          } else if (!once) {
            gsap.to(element, {
              opacity: 0,
              y: 40,
              duration: 0.4,
              ease: 'power3.in',
            });
          }
        });
      },
      { threshold, rootMargin }
    );

    observer.observe(element);

    return () => {
      observer.disconnect();
    };
  }, [threshold, rootMargin, once]);

  return elementRef;
}

/**
 * Hook for ripple effect on click
 */
export function useRippleEffect() {
  const elementRef = useRef<HTMLElement>(null);

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    const handleClick = (e: MouseEvent) => {
      const rect = element.getBoundingClientRect();
      const x = e.clientX - rect.left;
      const y = e.clientY - rect.top;

      const ripple = document.createElement('span');
      ripple.style.cssText = `
        position: absolute;
        border-radius: 50%;
        background: rgba(255, 255, 255, 0.3);
        pointer-events: none;
        transform: scale(0);
        left: ${x}px;
        top: ${y}px;
        width: 10px;
        height: 10px;
        margin-left: -5px;
        margin-top: -5px;
      `;

      element.style.position = 'relative';
      element.style.overflow = 'hidden';
      element.appendChild(ripple);

      gsap.to(ripple, {
        scale: 20,
        opacity: 0,
        duration: 0.6,
        ease: 'power2.out',
        onComplete: () => ripple.remove(),
      });
    };

    element.addEventListener('click', handleClick);

    return () => {
      element.removeEventListener('click', handleClick);
    };
  }, []);

  return elementRef;
}

/**
 * Hook for floating/levitating animation
 */
export function useFloatingAnimation(
  options: {
    amplitude?: number;
    duration?: number;
    delay?: number;
  } = {}
) {
  const elementRef = useRef<HTMLElement>(null);
  const animationRef = useRef<gsap.core.Tween | null>(null);

  const { amplitude = 10, duration = 3, delay = 0 } = options;

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    animationRef.current = gsap.to(element, {
      y: amplitude,
      duration,
      delay,
      ease: 'sine.inOut',
      yoyo: true,
      repeat: -1,
    });

    return () => {
      animationRef.current?.kill();
    };
  }, [amplitude, duration, delay]);

  return elementRef;
}

/**
 * Hook for pulse glow animation
 */
export function usePulseGlow(
  options: {
    color?: string;
    intensity?: number;
    duration?: number;
  } = {}
) {
  const elementRef = useRef<HTMLElement>(null);
  const animationRef = useRef<gsap.core.Tween | null>(null);

  const {
    color = 'rgba(0, 212, 255, 0.5)',
    intensity = 20,
    duration = 2,
  } = options;

  useEffect(() => {
    const element = elementRef.current;
    if (!element) return;

    animationRef.current = gsap.to(element, {
      boxShadow: `0 0 ${intensity}px ${color}`,
      duration,
      ease: 'sine.inOut',
      yoyo: true,
      repeat: -1,
    });

    return () => {
      animationRef.current?.kill();
    };
  }, [color, intensity, duration]);

  return elementRef;
}

/**
 * Utility function to create a timeline for complex animations
 */
export function createAnimationTimeline() {
  return gsap.timeline();
}

/**
 * Utility to animate a counter from one number to another
 */
export function animateCounter(
  element: HTMLElement,
  from: number,
  to: number,
  duration: number = 1
) {
  const obj = { value: from };
  gsap.to(obj, {
    value: to,
    duration,
    ease: 'power2.out',
    onUpdate: () => {
      element.textContent = Math.round(obj.value).toString();
    },
  });
}
