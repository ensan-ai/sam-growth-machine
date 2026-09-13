import type { ReactNode } from "react";

type Props = {
  label: string;
  title?: string;
  children?: ReactNode;
  className?: string;
  onClick?: () => void;
};

export function GlassHUD({ label, title, children, className, onClick }: Props) {
  const Tag = onClick ? "button" : "section";
  return (
    <Tag className={`glass-hud ${className || ""}`} onClick={onClick}>
      <header>
        <span className="eyebrow">{label}</span>
        {title ? <h2>{title}</h2> : null}
      </header>
      {children}
    </Tag>
  );
}

export function GlassChip({ children, tone }: { children: ReactNode; tone?: string }) {
  return <span className={`glass-chip ${tone || ""}`}>{children}</span>;
}
