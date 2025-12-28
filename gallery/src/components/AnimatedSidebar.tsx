/**
 * AnimatedSidebar - Sidebar with GSAP animations
 *
 * Features:
 * - Smooth icon hover animations
 * - Active indicator slide animation
 * - Tooltip animations
 * - Entrance stagger animation
 */

import { useRef, useEffect, useState } from 'react';
import gsap from 'gsap';
import { cn } from '../lib/utils';
import { CATEGORIES, getSamplesByCategory } from '../data/samples';
import * as Icons from 'lucide-react';

interface AnimatedSidebarProps {
  activeCategory: string | null;
  onCategoryClick: (categoryId: string) => void;
  onSettingsClick: () => void;
  onOpenLink: (url: string, title?: string) => void;
  onConsoleClick?: () => void;
  onExtensionsClick?: () => void;
  consoleOpen?: boolean;
}

const iconMap: Record<string, React.ComponentType<{ className?: string }>> = {
  rocket: Icons.Rocket,
  code: Icons.Code,
  layout: Icons.Layout,
  monitor: Icons.Monitor,
  box: Icons.Box,
};

interface SidebarButtonProps {
  icon: React.ComponentType<{ className?: string }>;
  title: string;
  isActive?: boolean;
  onClick: () => void;
  badge?: number;
  index: number;
}

function SidebarButton({
  icon: Icon,
  title,
  isActive,
  onClick,
  badge,
  index,
}: SidebarButtonProps) {
  const buttonRef = useRef<HTMLButtonElement>(null);
  const iconRef = useRef<HTMLDivElement>(null);
  const indicatorRef = useRef<HTMLDivElement>(null);
  const [showTooltip, setShowTooltip] = useState(false);
  const tooltipRef = useRef<HTMLDivElement>(null);

  // Entrance animation
  useEffect(() => {
    const button = buttonRef.current;
    if (!button) return;

    gsap.fromTo(
      button,
      { opacity: 0, x: -20 },
      {
        opacity: 1,
        x: 0,
        duration: 0.4,
        delay: index * 0.05,
        ease: 'power2.out',
      }
    );
  }, [index]);

  // Active indicator animation
  useEffect(() => {
    const indicator = indicatorRef.current;
    if (!indicator) return;

    if (isActive) {
      gsap.to(indicator, {
        scaleY: 1,
        opacity: 1,
        duration: 0.3,
        ease: 'power2.out',
      });
    } else {
      gsap.to(indicator, {
        scaleY: 0,
        opacity: 0,
        duration: 0.2,
        ease: 'power2.in',
      });
    }
  }, [isActive]);

  // Hover animation
  const handleMouseEnter = () => {
    setShowTooltip(true);

    if (iconRef.current) {
      gsap.to(iconRef.current, {
        scale: 1.15,
        rotate: 5,
        duration: 0.3,
        ease: 'back.out(1.7)',
      });
    }

    if (tooltipRef.current) {
      gsap.fromTo(
        tooltipRef.current,
        { opacity: 0, x: -10 },
        { opacity: 1, x: 0, duration: 0.2, ease: 'power2.out' }
      );
    }
  };

  const handleMouseLeave = () => {
    setShowTooltip(false);

    if (iconRef.current) {
      gsap.to(iconRef.current, {
        scale: 1,
        rotate: 0,
        duration: 0.3,
        ease: 'power2.out',
      });
    }
  };

  // Click animation
  const handleClick = () => {
    if (iconRef.current) {
      gsap.to(iconRef.current, {
        scale: 0.85,
        duration: 0.1,
        ease: 'power2.in',
        onComplete: () => {
          gsap.to(iconRef.current, {
            scale: 1.15,
            duration: 0.3,
            ease: 'elastic.out(1, 0.5)',
          });
        },
      });
    }

    onClick();
  };

  return (
    <div className="relative">
      <button
        ref={buttonRef}
        onClick={handleClick}
        onMouseEnter={handleMouseEnter}
        onMouseLeave={handleMouseLeave}
        className={cn(
          'w-10 h-10 rounded-lg flex items-center justify-center transition-colors relative',
          'text-muted-foreground hover:bg-accent hover:text-foreground',
          isActive && 'bg-primary/10 text-primary'
        )}
        title={title}
      >
        {/* Active indicator */}
        <div
          ref={indicatorRef}
          className="absolute left-0 top-1/2 -translate-y-1/2 w-0.5 h-4 bg-primary rounded-r origin-center"
          style={{ opacity: 0, transform: 'scaleY(0) translateY(-50%)' }}
        />

        {/* Icon container */}
        <div ref={iconRef} style={{ willChange: 'transform' }}>
          <Icon className="w-5 h-5" />
        </div>

        {/* Badge */}
        {badge !== undefined && badge > 0 && (
          <span className="absolute -top-1 -right-1 w-4 h-4 bg-primary text-[10px] text-primary-foreground rounded-full flex items-center justify-center font-medium">
            {badge > 9 ? '9+' : badge}
          </span>
        )}
      </button>

      {/* Tooltip */}
      {showTooltip && (
        <div
          ref={tooltipRef}
          className="absolute left-full ml-2 top-1/2 -translate-y-1/2 z-50 px-2 py-1 bg-popover text-popover-foreground text-xs rounded shadow-lg whitespace-nowrap"
        >
          {title}
        </div>
      )}
    </div>
  );
}

export function AnimatedSidebar({
  activeCategory,
  onCategoryClick,
  onSettingsClick,
  onOpenLink,
  onConsoleClick,
  onExtensionsClick,
  consoleOpen,
}: AnimatedSidebarProps) {
  const sidebarRef = useRef<HTMLElement>(null);
  const logoRef = useRef<HTMLDivElement>(null);
  const samplesByCategory = getSamplesByCategory();

  // Logo entrance animation
  useEffect(() => {
    const logo = logoRef.current;
    if (!logo) return;

    gsap.fromTo(
      logo,
      { opacity: 0, scale: 0.5, rotate: -180 },
      {
        opacity: 1,
        scale: 1,
        rotate: 0,
        duration: 0.8,
        ease: 'back.out(1.7)',
      }
    );
  }, []);

  // Logo hover animation
  const handleLogoHover = (isHovering: boolean) => {
    if (!logoRef.current) return;

    if (isHovering) {
      gsap.to(logoRef.current, {
        scale: 1.1,
        rotate: 10,
        duration: 0.3,
        ease: 'power2.out',
      });
    } else {
      gsap.to(logoRef.current, {
        scale: 1,
        rotate: 0,
        duration: 0.3,
        ease: 'power2.out',
      });
    }
  };

  const handleGitHubClick = () => {
    onOpenLink('https://github.com/loonghao/auroraview', 'GitHub - AuroraView');
  };

  let buttonIndex = 0;

  return (
    <aside
      ref={sidebarRef}
      className="w-14 bg-card border-r border-border fixed h-screen flex flex-col items-center py-4"
    >
      {/* Logo */}
      <div
        ref={logoRef}
        className="mb-6 cursor-pointer"
        onMouseEnter={() => handleLogoHover(true)}
        onMouseLeave={() => handleLogoHover(false)}
        onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
      >
        <img
          src="./auroraview-logo.png"
          alt="AuroraView"
          className="w-9 h-9 rounded-lg object-contain"
        />
      </div>

      {/* Navigation icons */}
      <nav className="flex-1 flex flex-col items-center gap-1">
        {/* Home */}
        <SidebarButton
          icon={Icons.Home}
          title="Home"
          isActive={!activeCategory}
          onClick={() => window.scrollTo({ top: 0, behavior: 'smooth' })}
          index={buttonIndex++}
        />

        {/* Category icons */}
        {Object.entries(CATEGORIES).map(([catId, catInfo]) => {
          const Icon = iconMap[catInfo.icon] || Icons.Circle;
          const count = samplesByCategory[catId]?.length || 0;
          return (
            <SidebarButton
              key={catId}
              icon={Icon}
              title={`${catInfo.title} (${count})`}
              isActive={activeCategory === catId}
              onClick={() => onCategoryClick(catId)}
              badge={count}
              index={buttonIndex++}
            />
          );
        })}
      </nav>

      {/* Bottom icons */}
      <div className="flex flex-col items-center gap-1">
        {onConsoleClick && (
          <SidebarButton
            icon={Icons.Terminal}
            title="Process Console"
            isActive={consoleOpen}
            onClick={onConsoleClick}
            index={buttonIndex++}
          />
        )}
        {onExtensionsClick && (
          <SidebarButton
            icon={Icons.Puzzle}
            title="Extensions"
            onClick={onExtensionsClick}
            index={buttonIndex++}
          />
        )}
        <SidebarButton
          icon={Icons.Github}
          title="GitHub"
          onClick={handleGitHubClick}
          index={buttonIndex++}
        />
        <SidebarButton
          icon={Icons.Settings}
          title="Settings"
          onClick={onSettingsClick}
          index={buttonIndex++}
        />
      </div>
    </aside>
  );
}
