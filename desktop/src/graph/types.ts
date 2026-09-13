export type NodeStatus = "IDLE" | "WORKING" | "BLOCKED" | "WAITING_FOR_SAM" | "AUTHORITY";

export type GraphNode = {
  id: string;
  name: string;
  shortSpecialty: string;
  title: string;
  reportsTo?: string;
  status: NodeStatus;
  role: "human_authority" | "employee";
  activeMission?: string;
};

export type GraphEdge = {
  id: string;
  source: string;
  target: string;
  relationship: "reports_to" | "handoff";
  active: boolean;
  missionId?: string;
};

export type GraphPulse = {
  id: string;
  source: string;
  target: string;
  startedAt: number;
  durationMs: number;
  kind: "handoff";
  evidenceId: string;
};

export type CompanyGraph = {
  nodes: GraphNode[];
  edges: GraphEdge[];
  pulses: GraphPulse[];
};

export type Vec3 = [number, number, number];
