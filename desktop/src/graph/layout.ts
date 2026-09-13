import type { GraphNode, Vec3 } from "./types";

export const CLUSTER_ENVELOPE = 1.72;

function hash01(value: string): number {
  let h = 2166136261;
  for (let i = 0; i < value.length; i++) {
    h ^= value.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return ((h >>> 0) % 10000) / 10000;
}

function childrenOf(nodes: GraphNode[], parentId: string): GraphNode[] {
  return nodes
    .filter((node) => (node.reportsTo || "").toLowerCase() === parentId.toLowerCase())
    .sort((a, b) => a.id.localeCompare(b.id));
}

function add(a: Vec3, b: Vec3): Vec3 {
  return [a[0] + b[0], a[1] + b[1], a[2] + b[2]];
}

function scale(a: Vec3, s: number): Vec3 {
  return [a[0] * s, a[1] * s, a[2] * s];
}

function length(a: Vec3): number {
  return Math.hypot(a[0], a[1], a[2]);
}

function clampToEnvelope(pos: Vec3, minR = 0.82): Vec3 {
  let next: Vec3 = [pos[0], Math.max(-0.72, Math.min(0.78, pos[1])), Math.max(-0.22, pos[2])];
  let r = length(next);
  if (r < 0.04) next = [0.12, -0.18, 0.78];
  r = length(next);
  if (r < minR) next = scale(next, minR / r);
  r = length(next);
  if (r > CLUSTER_ENVELOPE) next = scale(next, CLUSTER_ENVELOPE / r);
  return next;
}

/**
 * Compact constellation around Sam. Parent-child links stay short.
 * Nodes stay inside a bounded volume so the default camera can frame everyone.
 */
export function layoutWorkforce(nodes: GraphNode[]): Map<string, Vec3> {
  const positions = new Map<string, Vec3>();
  const byId = new Map(nodes.map((node) => [node.id.toLowerCase(), node]));
  const root = nodes.find((node) => node.role === "human_authority") || nodes.find((node) => !node.reportsTo) || nodes[0];
  if (!root) return positions;
  positions.set(root.id, [0, 0.06, 0]);

  const walk = (parent: GraphNode, depth: number) => {
    const kids = childrenOf(nodes, parent.id);
    if (!kids.length) return;
    const parentPos = positions.get(parent.id) || [0, 0, 0];
    const reach = depth === 1 ? 0.9 : 0.38 + Math.min(kids.length, 8) * 0.02;
    const basisYaw = hash01(parent.id) * Math.PI * 2;
    kids.forEach((child, index) => {
      const t = kids.length === 1 ? 0 : index / kids.length;
      const yaw = basisYaw + t * Math.PI * 2 + (hash01(child.id) - 0.5) * 0.22;
      const pitch = 0.18 + (hash01(child.id + ":p") - 0.5) * 0.32 + (depth === 1 ? -0.08 : 0.06);
      const local: Vec3 = [
        Math.cos(pitch) * Math.sin(yaw) * reach,
        Math.sin(pitch) * reach * 0.85,
        Math.cos(pitch) * Math.cos(yaw) * reach * 0.62 + 0.12,
      ];
      const pos = clampToEnvelope(add(parentPos, local));
      positions.set(child.id, pos);
      walk(child, depth + 1);
    });
  };

  walk(root, 1);

  nodes.forEach((node) => {
    if (positions.has(node.id)) return;
    const parent = node.reportsTo ? byId.get(node.reportsTo.toLowerCase()) : undefined;
    const origin = (parent && positions.get(parent.id)) || [0, 0.06, 0];
    const yaw = hash01(node.id) * Math.PI * 2;
    positions.set(node.id, clampToEnvelope(add(origin, [
      Math.sin(yaw) * 0.5,
      -0.12,
      Math.cos(yaw) * 0.32 + 0.1,
    ])));
  });

  return positions;
}

export function clusterBounds(positions: Map<string, Vec3>): { center: Vec3; radius: number } {
  const pts = [...positions.values()];
  if (!pts.length) return { center: [0, 0, 0], radius: 1 };
  const center: Vec3 = [0, 0, 0];
  pts.forEach((p) => {
    center[0] += p[0];
    center[1] += p[1];
    center[2] += p[2];
  });
  center[0] /= pts.length;
  center[1] /= pts.length;
  center[2] /= pts.length;
  let radius = 0.4;
  pts.forEach((p) => {
    radius = Math.max(radius, Math.hypot(p[0] - center[0], p[1] - center[1], p[2] - center[2]));
  });
  return { center, radius };
}

export function defaultCamera(positions: Map<string, Vec3>, aspect = 1.45): { position: Vec3; target: Vec3 } {
  const { center, radius } = clusterBounds(positions);
  const fov = 42 * Math.PI / 180;
  const fit = radius * 1.35;
  const dist = Math.min(4.15, Math.max(2.55, fit / Math.tan(fov / 2) / Math.max(0.82, Math.min(aspect, 1.7) * 0.62)));
  return {
    position: [center[0] * 0.15, center[1] + 0.42, dist],
    target: [center[0] * 0.2, center[1] * 0.4 + 0.04, center[2] * 0.15],
  };
}

export function nodeRadius(node: GraphNode, count: number): number {
  if (node.role === "human_authority") return 0.38;
  const shrink = count > 40 ? 0.82 : count > 20 ? 0.92 : 1;
  const senior = (node.reportsTo || "sam") === "sam";
  return (senior ? 0.16 : 0.13) * shrink;
}

export function parentChildDistance(positions: Map<string, Vec3>, child: GraphNode): number {
  if (!child.reportsTo) return 0;
  const a = positions.get(child.id);
  const b = positions.get(child.reportsTo);
  if (!a || !b) return 0;
  return Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2]);
}
