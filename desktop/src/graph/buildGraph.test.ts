import { describe, expect, it } from "vitest";
import { buildCompanyGraph } from "./buildGraph";
import { MIN_BODY_GAP, SAM_RADIUS, clusterEnvelope, layoutWorkforce, pairwiseMinDistance, parentChildDistance } from "./layout";
import { shortSpecialty } from "./specialties";
import type { Employee, Handoff } from "../types";

const team: Employee[] = [
  { employeeId: "travis", name: "Travis", title: "Main Agent / Growth Director", reportsTo: "sam", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "saly", name: "Saly", title: "Research Director", reportsTo: "travis", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "WORKING" },
  { employeeId: "adam", name: "Adam", title: "Strategy Director", reportsTo: "travis", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "brain", name: "Brain", title: "Public Writer", reportsTo: "adam", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
];

describe("buildCompanyGraph", () => {
  it("includes Sam plus every employee from the registry payload", () => {
    const graph = buildCompanyGraph({ employees: team, handoffs: [], events: [], waitingApprovals: 0 });
    expect(graph.nodes.map((node) => node.id).sort()).toEqual(["adam", "brain", "saly", "sam", "travis"]);
    expect(graph.nodes.find((node) => node.id === "sam")?.role).toBe("human_authority");
  });

  it("adds a future employee without any renderer-specific name checks", () => {
    const leo: Employee = { employeeId: "leo", name: "Leo", title: "SEO Specialist", reportsTo: "travis", version: "1.0.0", definitionStatus: "draft", runtimeStatus: "IDLE" };
    const graph = buildCompanyGraph({ employees: [...team, leo], handoffs: [], events: [], waitingApprovals: 0 });
    expect(graph.nodes.some((node) => node.id === "leo")).toBe(true);
    expect(graph.nodes.find((node) => node.id === "leo")?.shortSpecialty).toBe("SEO");
    expect(graph.edges.some((edge) => edge.source === "travis" && edge.target === "leo" && edge.relationship === "reports_to")).toBe(true);
  });

  it("creates pulses only from real handoffs with known endpoints", () => {
    const now = Date.parse("2026-09-13T20:00:00Z");
    const handoffs: Handoff[] = [
      { handoffId: "h1", workItemId: "w1", sender: "saly", receiver: "travis", artifactId: "a1", status: "READY", createdAt: "2026-09-13T19:59:58Z" },
      { handoffId: "h2", workItemId: "w1", sender: "system", receiver: "sam", artifactId: "a2", status: "READY", createdAt: "2026-09-13T19:59:58Z" },
    ];
    const graph = buildCompanyGraph({ employees: team, handoffs, events: [], waitingApprovals: 0, now });
    expect(graph.pulses).toHaveLength(1);
    expect(graph.pulses[0]).toMatchObject({ source: "saly", target: "travis", evidenceId: "h1" });
  });

  it("does not invent pulses from idle companies", () => {
    const graph = buildCompanyGraph({ employees: team, handoffs: [], events: [], waitingApprovals: 0 });
    expect(graph.pulses).toEqual([]);
  });
});

describe("layoutWorkforce", () => {
  it("places Sam at the nucleus and keeps layout stable when a node is added", () => {
    const base = buildCompanyGraph({ employees: team, handoffs: [], events: [], waitingApprovals: 0 });
    const withLeo = buildCompanyGraph({
      employees: [...team, { employeeId: "leo", name: "Leo", title: "SEO Specialist", reportsTo: "travis", version: "1", definitionStatus: "draft", runtimeStatus: "IDLE" }],
      handoffs: [],
      events: [],
      waitingApprovals: 0,
    });
    const a = layoutWorkforce(base.nodes);
    const b = layoutWorkforce(withLeo.nodes);
    expect(a.get("sam")).toEqual([0, 0.12, 0]);
    expect(a.get("travis")).toEqual(b.get("travis"));
    expect(b.get("leo")).toBeTruthy();
    const travis = a.get("travis")!;
    expect(travis[1]).toBeLessThan(0.12);
    expect(Math.hypot(travis[0], travis[1] - 0.12, travis[2])).toBeGreaterThan(SAM_RADIUS + MIN_BODY_GAP);
    expect(pairwiseMinDistance(a)).toBeGreaterThan(0.36);
    const xs = [...a.entries()].filter(([id]) => id !== "sam").map(([, p]) => p[0]);
    expect(Math.max(...xs) - Math.min(...xs)).toBeGreaterThan(0.4);
    base.nodes.filter((n) => n.reportsTo).forEach((n) => {
      const d = parentChildDistance(a, n);
      expect(d).toBeGreaterThan(0.3);
      expect(d).toBeLessThan(1.1);
    });
  });

  it("layouts 100 employees without dropping nodes or collapsing", () => {
    const crowd: Employee[] = Array.from({ length: 100 }, (_, i) => ({
      employeeId: `e${i}`,
      name: `E${i}`,
      title: i % 5 === 0 ? "SEO Specialist" : "Growth Analyst",
      reportsTo: i === 0 ? "sam" : i < 8 ? "e0" : `e${Math.floor(i / 8)}`,
      version: "1.0.0",
      definitionStatus: "draft",
      runtimeStatus: "IDLE",
    }));
    const graph = buildCompanyGraph({ employees: crowd, handoffs: [], events: [], waitingApprovals: 0 });
    const positions = layoutWorkforce(graph.nodes);
    expect(positions.size).toBe(101);
    expect(positions.get("sam")).toEqual([0, 0.12, 0]);
    const span = [...positions.values()].reduce((m, p) => Math.max(m, Math.hypot(p[0], p[1] - 0.12, p[2])), 0);
    expect(span).toBeLessThanOrEqual(clusterEnvelope(101) + 0.05);
    expect(pairwiseMinDistance(positions, "sam")).toBeGreaterThan(0.12);
  });
});

describe("shortSpecialty", () => {
  it("uses founding copy and derives future titles", () => {
    expect(shortSpecialty("brain", "Public Writer")).toBe("Public Writing");
    expect(shortSpecialty("leo", "SEO Specialist")).toBe("SEO");
    expect(shortSpecialty("nova", "WhatsApp Growth Lead")).toBe("WhatsApp Growth");
  });
});
