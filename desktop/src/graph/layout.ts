import type { GraphNode, Vec3 } from "./types";

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

/**
 * Deterministic 3D layout from reportsTo. No hardcoded employee ids.
 * Children cluster around their manager; depth grows with org level.
 */
export function layoutWorkforce(nodes: GraphNode[]): Map<string, Vec3> {
  const positions = new Map<string, Vec3>();
  const byId = new Map(nodes.map((node) => [node.id.toLowerCase(), node]));
  const root = nodes.find((node) => node.role === "human_authority") || nodes.find((node) => !node.reportsTo) || nodes[0];
  if (!root) return positions;
  positions.set(root.id, [0, 0.35, 0]);

  const walk = (parent: GraphNode, depth: number, parentPos: Vec3, parentYaw: number) => {
    const kids = childrenOf(nodes, parent.id);
    if (!kids.length) return;
    const radius = 1.55 + depth * 1.15 + Math.max(0, kids.length - 3) * 0.18;
    const cone = Math.PI * (0.55 + Math.min(kids.length, 12) * 0.04);
    kids.forEach((child, index) => {
      const t = kids.length === 1 ? 0.5 : index / (kids.length - 1);
      const yaw = parentYaw + (t - 0.5) * cone + (hash01(child.id) - 0.5) * 0.18;
      const lift = -0.55 - depth * 0.15 + (hash01(child.id + ":y") - 0.5) * 0.35;
      const swirl = (hash01(child.id + ":z") - 0.5) * 0.55;
      const pos: Vec3 = [
        parentPos[0] + Math.sin(yaw) * radius,
        parentPos[1] + lift,
        parentPos[2] + Math.cos(yaw) * radius * 0.86 + swirl,
      ];
      positions.set(child.id, pos);
      walk(child, depth + 1, pos, yaw);
    });
  };

  walk(root, 1, positions.get(root.id)!, 0);

  nodes.forEach((node) => {
    if (positions.has(node.id)) return;
    const parent = node.reportsTo ? byId.get(node.reportsTo.toLowerCase()) : undefined;
    const parentPos = parent ? positions.get(parent.id) : undefined;
    const h = hash01(node.id);
    const yaw = h * Math.PI * 2;
    const radius = parentPos ? 2.1 : 3.4;
    const origin = parentPos || [0, 0, 0];
    positions.set(node.id, [
      origin[0] + Math.sin(yaw) * radius,
      origin[1] - 0.7,
      origin[2] + Math.cos(yaw) * radius,
    ]);
  });

  return positions;
}

export function nodeRadius(node: GraphNode, count: number): number {
  if (node.role === "human_authority") return 0.62;
  const shrink = count > 40 ? 0.78 : count > 20 ? 0.9 : 1;
  const senior = (node.reportsTo || "sam") === "sam";
  return (senior ? 0.28 : 0.2) * shrink;
}
