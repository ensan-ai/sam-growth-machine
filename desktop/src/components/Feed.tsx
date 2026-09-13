import { useLayoutEffect, useRef, type ReactNode, type UIEvent } from "react";
import { nextFeedScrollTop } from "../chrono";

const TOP_PX = 32;

type Props = { children: ReactNode; className?: string; pinKey: string };

export function Feed({ children, className, pinKey }: Props) {
  const ref = useRef<HTMLDivElement>(null);
  const pin = useRef({ atTop: true, height: 0, top: 0 });

  const onScroll = (event: UIEvent<HTMLDivElement>) => {
    const el = event.currentTarget;
    pin.current.atTop = el.scrollTop <= TOP_PX;
    pin.current.top = el.scrollTop;
    pin.current.height = el.scrollHeight;
  };

  useLayoutEffect(() => {
    const el = ref.current;
    if (!el) return;
    el.scrollTop = nextFeedScrollTop(pin.current.atTop, pin.current, el.scrollHeight);
    pin.current.height = el.scrollHeight;
    pin.current.top = el.scrollTop;
  }, [pinKey]);

  return (
    <div ref={ref} className={`feed${className ? ` ${className}` : ""}`} onScroll={onScroll}>
      {children}
    </div>
  );
}
