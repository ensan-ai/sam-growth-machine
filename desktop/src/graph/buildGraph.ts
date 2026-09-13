import type { Employee, Execution, Handoff, SystemEvent, WorkItem } from "../types";
import { shortSpecialty } from "./specialties";
import type { CompanyGraph, GraphEdge, GraphNode, GraphPulse, NodeStatus } from "./types";

const PULSE_MS = 2800;
const PULSE_LIMIT = 10;
const HUMAN = "sam";

function normalize(id: string): string {
  return id.trim().toLowerCase();
}

function statusOf(employee: Employee, waitingIds: Set<string>): NodeStatus {
  const raw = (employee.runtimeStatus || "IDLE").toUpperCase().replaceAll(" ", "_");
  if (waitingIds.has(normalize(employee.employeeId))) return "WAITING_FOR_SAM";
  if (raw === "BLOCKED") return "BLOCKED";
  if (raw === "WORKING") return "WORKING";
  if (raw === "WAITING_APPROVAL" || raw === "WAITING_FOR_SAM") return "WAITING_FOR_SAM";
  return "IDLE";
}

function waitingEmployeeIds(workItems: WorkItem[]): Set<string> {
  const ids = new Set<string>();
  workItems.forEach((item) => {
    if (item.state === "READY_FOR_APPROVAL" && item.currentOwner) {
      item.currentOwner.split(/[+,&/]/).forEach((part) => {
        const id = normalize(part);
        if (id && id !== HUMAN && id !== "system") ids.add(id);
      });
    }
  });
  return ids;
}

export function buildCompanyGraph(input: {
  employees: Employee[];
  handoffs: Handoff[];
  events: SystemEvent[];
  executions?: Execution[];
  workItems?: WorkItem[];
  waitingApprovals: number;
  now?: number;
}): CompanyGraph {
  const now = input.now ?? Date.now();
  const waitingIds = waitingEmployeeIds(input.workItems || []);
  const employees = input.employees.filter((employee) => normalize(employee.employeeId) !== HUMAN);

  const sam: GraphNode = {
    id: HUMAN,
    name: "Sam",
    shortSpecialty: shortSpecialty("sam", "Founder / CEO"),
    title: "Founder / CEO / Human Authority",
    status: input.waitingApprovals > 0 || waitingIds.size ? "WAITING_FOR_SAM" : "AUTHORITY",
    role: "human_authority",
  };

  const nodes: GraphNode[] = [
    sam,
    ...employees.map((employee) => ({
      id: normalize(employee.employeeId),
      name: employee.name,
      shortSpecialty: shortSpecialty(employee.employeeId, employee.title),
      title: employee.title,
      reportsTo: employee.reportsTo ? normalize(employee.reportsTo) : HUMAN,
      status: statusOf(employee, waitingIds),
      role: "employee" as const,
    })),
  ];

  const known = new Set(nodes.map((node) => node.id));
  const edges: GraphEdge[] = [];
  nodes.forEach((node) => {
    if (!node.reportsTo || !known.has(node.reportsTo) || node.reportsTo === node.id) return;
    edges.push({
      id: `org:${node.id}:${node.reportsTo}`,
      source: node.reportsTo,
      target: node.id,
      relationship: "reports_to",
      active: node.status === "WORKING" || node.status === "WAITING_FOR_SAM",
    });
  });

  const pulses: GraphPulse[] = [];
  const seenPulse = new Set<string>();
  const consider = (source: string, target: string, startedAt: number, evidenceId: string) => {
    const from = normalize(source);
    const to = normalize(target);
    if (!known.has(from) || !known.has(to) || from === to) return;
    if (now - startedAt < 0 || now - startedAt > PULSE_MS) return;
    const key = `${evidenceId}:${from}:${to}`;
    if (seenPulse.has(key)) return;
    seenPulse.add(key);
    pulses.push({
      id: key,
      source: from,
      target: to,
      startedAt,
      durationMs: PULSE_MS,
      kind: "handoff",
      evidenceId,
    });
    edges.push({
      id: `handoff:${key}`,
      source: from,
      target: to,
      relationship: "handoff",
      active: true,
      missionId: evidenceId,
    });
  };

  input.handoffs.forEach((handoff) => {
    consider(handoff.sender, handoff.receiver, Date.parse(handoff.createdAt) || 0, handoff.handoffId);
  });

  input.events.forEach((event) => {
    const detail = event.detail && typeof event.detail === "object" ? event.detail as Record<string, unknown> : {};
    const sender = typeof detail.sender === "string" ? detail.sender : typeof detail.from === "string" ? detail.from : "";
    const receiver = typeof detail.receiver === "string" ? detail.receiver : typeof detail.to === "string" ? detail.to : "";
    if (sender && receiver) consider(sender, receiver, Date.parse(event.createdAt) || 0, event.eventId);
  });

  pulses.sort((a, b) => b.startedAt - a.startedAt);
  return { nodes, edges, pulses: pulses.slice(0, PULSE_LIMIT) };
}
