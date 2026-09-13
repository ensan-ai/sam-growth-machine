export function newestFirst<T>(items: T[], timestamp: (item: T) => string | undefined | null): T[] {
  return [...items].sort((a, b) => {
    const ta = Date.parse(timestamp(a) || "") || 0;
    const tb = Date.parse(timestamp(b) || "") || 0;
    return tb - ta;
  });
}

export function nextFeedScrollTop(
  atTop: boolean,
  previous: { height: number; top: number },
  nextHeight: number,
): number {
  if (atTop) return 0;
  return Math.max(0, previous.top + (nextHeight - previous.height));
}
