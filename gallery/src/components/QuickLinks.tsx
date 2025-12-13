import { cn } from '../lib/utils';
import * as Icons from 'lucide-react';

interface QuickLinkProps {
  href?: string;
  onClick?: () => void;
  icon: React.ReactNode;
  iconBg: string;
  title: string;
  description: string;
  external?: boolean;
}

function QuickLink({ href, onClick, icon, iconBg, title, description, external }: QuickLinkProps) {
  const handleClick = (e: React.MouseEvent) => {
    if (onClick) {
      e.preventDefault();
      onClick();
    }
  };

  return (
    <a
      href={href || '#'}
      onClick={handleClick}
      className={cn(
        "bg-card border border-border rounded-xl p-5 no-underline text-foreground",
        "fluent-card transition-all flex flex-col gap-3 cursor-pointer group",
        "hover:border-primary/30"
      )}
      target={external ? "_blank" : undefined}
      rel={external ? "noopener noreferrer" : undefined}
    >
      <div className={cn(
        "w-12 h-12 rounded-xl flex items-center justify-center",
        iconBg
      )}>
        {icon}
      </div>
      <div>
        <div className="font-semibold text-sm flex items-center gap-1.5">
          {title}
          {external && <Icons.ExternalLink className="w-3 h-3 text-muted-foreground" />}
        </div>
        <div className="text-xs text-muted-foreground mt-1">{description}</div>
      </div>
    </a>
  );
}

interface QuickLinksProps {
  onCategoryClick: (categoryId: string) => void;
  onOpenLink: (url: string, title?: string) => void;
}

export function QuickLinks({ onCategoryClick, onOpenLink }: QuickLinksProps) {
  return (
    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4 mb-10">
      <QuickLink
        onClick={() => onCategoryClick('getting_started')}
        icon={<Icons.Sparkles className="w-6 h-6 text-primary" />}
        iconBg="bg-primary/10"
        title="Getting Started"
        description="An overview of app development options and samples."
      />
      <QuickLink
        onClick={() => onOpenLink('https://github.com/loonghao/auroraview', 'GitHub - AuroraView')}
        icon={<Icons.Github className="w-6 h-6 text-foreground" />}
        iconBg="bg-muted"
        title="GitHub Repo"
        description="The latest design controls and styles for your applications."
        external
      />
      <QuickLink
        onClick={() => onCategoryClick('api_patterns')}
        icon={<Icons.Braces className="w-6 h-6 text-orange-500" />}
        iconBg="bg-orange-500/10"
        title="Code Samples"
        description="Find samples that demonstrate specific tasks, features and APIs."
      />
      <QuickLink
        onClick={() => onOpenLink('https://github.com/loonghao/auroraview/issues', 'GitHub Issues - AuroraView')}
        icon={<Icons.MessageSquare className="w-6 h-6 text-green-600" />}
        iconBg="bg-green-500/10"
        title="Send Feedback"
        description="Help us improve AuroraView by providing feedback."
        external
      />
    </div>
  );
}
