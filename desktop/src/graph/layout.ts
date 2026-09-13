import type { GraphNode, Vec3 } from "./types";

export const EMPLOYEE_RADIUS = 0.078;
export const COORDINATOR_RADIUS = 0.094;
export const SAM_RADIUS = 0.156;
export const MIN_BODY_GAP = 0.24;

export type LabelAnchor = "left" | "right" | "below" | "above";

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

function sub(a: Vec3, b: Vec3): Vec3 {
  return [a[0] - b[0], a[1] - b[1], a[2] - b[2]];
}

function scale(a: Vec3, s: number): Vec3 {
  return [a[0] * s, a[1] * s, a[2] * s];
}

export function length(a: Vec3): number {
  return Math.hypot(a[0], a[1], a[2]);
}

function norm(a: Vec3, fallback: Vec3): Vec3 {
  const r = length(a);
  return r < 1e-6 ? fallback : scale(a, 1 / r);
}

export function clusterEnvelope(count: number): number {
  return 1.12 + Math.sqrt(Math.max(1, count - 1)) * 0.26;
}

export function nodeRadius(node: GraphNode, _count = 0): number {
  if (node.role === "human_authority") return SAM_RADIUS;
  if ((node.reportsTo || "sam") === "sam") return COORDINATOR_RADIUS;
  return EMPLOYEE_RADIUS;
}

function radiusOf(node: GraphNode): number {
  return nodeRadius(node);
}

function minSep(a: GraphNode, b: GraphNode, count: number): number {
  const gap = count > 40 ? 0.1 : MIN_BODY_GAP;
  return radiusOf(a) + radiusOf(b) + gap;
}

/**
 * Flattened 2.5D neural constellation.
 * Sam is the nucleus. Direct reports sit close as coordinators.
 * Deeper employees fan away from the nucleus with short local branches.
 * A relaxation pass enforces body separation and angular spread.
 */
export function layoutWorkforce(nodes: GraphNode[]): Map<string, Vec3> {
  const positions = new Map<string, Vec3>();
  const root = nodes.find((node) => node.role === "human_authority") || nodes.find((node) => !node.reportsTo) || nodes[0];
  if (!root) return positions;
  positions.set(root.id, [0, 0.12, 0]);

  const walk = (parent: GraphNode, depth: number) => {
    const kids = childrenOf(nodes, parent.id);
    if (!kids.length) return;
    const parentPos = positions.get(parent.id) || [0, 0, 0];
    const away = depth === 1
      ? ([0.2, -0.96, 0.1] as Vec3)
      : norm(sub(parentPos, [0, 0.12, 0]), [0, -1, 0]);
    const right = norm([-away[1], away[0], 0], [1, 0, 0]);
    const reach = depth === 1 ? 0.52 : 0.44 + Math.min(kids.length, 6) * 0.018;
    const spread = 0.4 + Math.min(kids.length, 8) * 0.06;
    const span = kids.length === 1 ? 0 : Math.min(2.2, Math.max(0.9, (kids.length - 1) * 0.58));
    kids.forEach((child, index) => {
      const t = kids.length === 1 ? 0 : index / (kids.length - 1) - 0.5;
      const theta = t * span + (hash01(child.id) - 0.5) * 0.1;
      const z = (hash01(child.id + ":z") - 0.5) * 0.46;
      const along = add(scale(away, reach), scale(right, Math.sin(theta) * spread));
      along[1] += Math.cos(theta) * spread * 0.22;
      along[2] = z + 0.08;
      positions.set(child.id, add(parentPos, along));
      walk(child, depth + 1);
    });
  };

  walk(root, 1);
  nodes.forEach((node) => {
    if (!positions.has(node.id)) positions.set(node.id, [hash01(node.id) - 0.5, -0.4, 0.1]);
  });

  relax(nodes, positions, root.id);
  return positions;
}

function relax(nodes: GraphNode[], positions: Map<string, Vec3>, rootId: string) {
  const envelope = clusterEnvelope(nodes.length);
  const byId = new Map(nodes.map((n) => [n.id, n]));
  const pinned = new Set([rootId, ...childrenOf(nodes, rootId).map((n) => n.id)]);
  const iters = nodes.length > 36 ? 72 : 48;
  for (let iter = 0; iter < iters; iter++) {
    for (let i = 0; i < nodes.length; i++) {
      for (let j = i + 1; j < nodes.length; j++) {
        const a = nodes[i];
        const b = nodes[j];
        const pa = positions.get(a.id)!;
        const pb = positions.get(b.id)!;
        const d = sub(pb, pa);
        const dist = length(d) || 0.0001;
        const need = minSep(a, b, nodes.length);
        if (dist >= need) continue;
        const push = scale(norm(d, [1, 0, 0]), (need - dist) * 0.55);
        const pinA = pinned.has(a.id);
        const pinB = pinned.has(b.id);
        if (a.id === rootId || (pinA && !pinB)) positions.set(b.id, add(pb, push));
        else if (b.id === rootId || (pinB && !pinA)) positions.set(a.id, sub(pa, push));
        else if (!pinA && !pinB) {
          positions.set(a.id, sub(pa, scale(push, 0.5)));
          positions.set(b.id, add(pb, scale(push, 0.5)));
        }
      }
    }

    nodes.forEach((node) => {
      if (node.id === rootId) return;
      let p = positions.get(node.id)!;
      p = [p[0], p[1], Math.max(-0.34, Math.min(0.4, p[2]))];
      const fromRoot = sub(p, [0, 0.12, 0]);
      const r = length(fromRoot);
      if (r > envelope) p = add([0, 0.12, 0], scale(fromRoot, envelope / r));
      positions.set(node.id, p);
    });
    positions.set(rootId, [0, 0.12, 0]);
  }

  if (nodes.length <= 24) {
  nodes.forEach((node) => {
    if (node.id === rootId || !node.reportsTo) return;
    const parent = byId.get(node.reportsTo);
    if (!parent) return;
    const pa = positions.get(parent.id);
    const pb = positions.get(node.id);
    if (!pa || !pb) return;
    const dist = length(sub(pb, pa));
    if (dist > 1.05) {
      const dir = norm(sub(pb, pa), [0, -1, 0]);
      positions.set(node.id, add(pa, scale(dir, 0.72)));
    }
  });
  }
}

export function labelAnchor(id: string, pos: Vec3, root: Vec3 = [0, 0.12, 0]): LabelAnchor {
  if (id.toLowerCase() === "sam") return "above";
  const dx = pos[0] - root[0];
  const dy = pos[1] - root[1];
  if (Math.abs(dx) >= Math.abs(dy)) return dx >= 0 ? "right" : "left";
  return dy >= 0 ? "above" : "below";
}

export function clusterBounds(positions: Map<string, Vec3>): { center: Vec3; radius: number } {
  const pts = [...positions.values()];
  if (!pts.length) return { center: [0, 0.12, 0], radius: 1 };
  const center: Vec3 = [0, 0, 0];
  pts.forEach((p) => {
    center[0] += p[0];
    center[1] += p[1];
    center[2] += p[2];
  });
  center[0] /= pts.length;
  center[1] /= pts.length;
  center[2] /= pts.length;
  let radius = 0.55;
  pts.forEach((p) => {
    radius = Math.max(radius, Math.hypot(p[0] - center[0], p[1] - center[1], p[2] - center[2]));
  });
  return { center, radius };
}

export function defaultCamera(positions: Map<string, Vec3>, aspect = 1.45): { position: Vec3; target: Vec3 } {
  const { center, radius } = clusterBounds(positions);
  const fov = 40 * Math.PI / 180;
  const fit = radius * 1.28;
  const dist = Math.min(4.4, Math.max(2.35, (fit / Math.tan(fov / 2)) / Math.max(0.9, Math.min(aspect, 1.8) * 0.72)));
  return {
    position: [center[0] * 0.08, center[1] + 0.18, dist],
    target: [center[0] * 0.15, center[1] * 0.55 + 0.04, 0],
  };
}

export function parentChildDistance(positions: Map<string, Vec3>, child: GraphNode): number {
  if (!child.reportsTo) return 0;
  const a = positions.get(child.id);
  const b = positions.get(child.reportsTo);
  if (!a || !b) return 0;
  return length(sub(a, b));
}

export function pairwiseMinDistance(positions: Map<string, Vec3>, skipId?: string): number {
  const ids = [...positions.keys()].filter((id) => id !== skipId);
  let min = Infinity;
  for (let i = 0; i < ids.length; i++) {
    for (let j = i + 1; j < ids.length; j++) {
      const a = positions.get(ids[i])!;
      const b = positions.get(ids[j])!;
      min = Math.min(min, length(sub(a, b)));
    }
  }
  return min;
}
