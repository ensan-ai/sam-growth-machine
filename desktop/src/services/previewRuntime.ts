import type { Activity, Approval, Dashboard, Employee, NovaResearchReport, ProviderStatus, Settings, WorkDetail, WorkItem } from "../types";

const team: Employee[] = [
  { employeeId: "travis", name: "Travis", title: "Main Agent / Growth Director", reportsTo: "sam", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "saly", name: "Saly", title: "Research Director", reportsTo: "travis", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "WORKING" },
  { employeeId: "nova", name: "NOVA", title: "Instagram Content Researcher", reportsTo: "saly", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "adam", name: "Adam", title: "Strategy Director", reportsTo: "travis", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "brain", name: "Brain", title: "Public Writer", reportsTo: "adam", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "jax", name: "Jax", title: "Creative Director", reportsTo: "adam", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "maro", name: "Maro", title: "Publishing Operator", reportsTo: "travis", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
  { employeeId: "lara", name: "Lara", title: "Performance Analyst", reportsTo: "travis", version: "1.0.0", definitionStatus: "reviewable", runtimeStatus: "IDLE" },
];

const previewOrigin = Date.now();
function ago(ms: number) {
  return new Date(previewOrigin - ms).toISOString();
}

const workItems: WorkItem[] = [
  { workItemId: "preview-1", title: "KERNEL GROK TEST 01", state: "IN_PRODUCTION", currentOwner: "brain + jax", createdAt: ago(120000), updatedAt: ago(4000) },
];

export const previewRuntime = {
  dashboard: async (): Promise<Dashboard> => ({
    companyState: "RUNNING",
    providerStatus: { ollamaAvailable: false, ollamaModelAvailable: false, ollamaEndpoint: "http://127.0.0.1:11434", ollamaModel: "qwen3:14b", openaiConfigured: false, openaiModel: "gpt-5.4", message: "Preview mode · kernel disconnected" },
    activeAgents: ["saly"],
    waitingApprovals: 0,
    blockedWork: 0,
    workItems,
  }),
  team: async () => team,
  workItems: async () => workItems,
  workItem: async (workItemId: string): Promise<WorkDetail> => ({
    workItem: workItems[0],
    artifacts: [],
    handoffs: [
      { handoffId: "preview-h1", workItemId, sender: "saly", receiver: "travis", artifactId: "a1", status: "READY", createdAt: ago(900) },
      { handoffId: "preview-h2", workItemId, sender: "travis", receiver: "adam", artifactId: "a2", status: "READY", createdAt: ago(400) },
    ],
    approvals: [],
    runs: [],
    events: [
      { eventId: "e1", workItemId, eventType: "capability.ran", actor: "saly", detail: { capability: "package_opportunity" }, createdAt: ago(2000) },
    ],
    executions: [],
  }),
  approvals: async (): Promise<Approval[]> => [],
  activity: async (): Promise<Activity> => ({
    runs: [],
    events: [
      { eventId: "e1", workItemId: "preview-1", eventType: "capability.ran", actor: "saly", detail: { capability: "package_opportunity" }, createdAt: ago(2000) },
      { eventId: "e2", workItemId: "preview-1", eventType: "capability.ran", actor: "travis", detail: { capability: "prioritize_and_assign" }, createdAt: ago(800) },
    ],
    executions: [],
  }),
  settings: async (): Promise<Settings> => ({ ollamaEndpoint: "http://127.0.0.1:11434", ollamaModel: "qwen3:14b", openaiModel: "gpt-5.4", allowOpenaiEscalation: false, companyControl: "RUNNING" }),
  providerStatus: async (): Promise<ProviderStatus> => ({ ollamaAvailable: false, ollamaModelAvailable: false, ollamaEndpoint: "", ollamaModel: "", openaiConfigured: false, openaiModel: "", message: "Preview mode" }),
  startWorkItem: async () => workItems[0],
  runWorkItem: async (id: string) => previewRuntime.workItem(id),
  decideApproval: async (id: string) => previewRuntime.workItem(id),
  control: async () => previewRuntime.settings(),
  saveSettings: async () => previewRuntime.settings(),
  novaResearchCreator: async (instagramUsername: string, maxReels = 20): Promise<NovaResearchReport> => ({
    type: "CREATOR_RESEARCH_REPORT",
    creator: instagramUsername.startsWith("@") ? instagramUsername : `@${instagramUsername}`,
    status: "COMPLETE",
    existingVideos: 12,
    newVideosAdded: Math.min(maxReels, 3),
    totalScriptsStored: 15,
    relevantVideos: [],
    notion: { creatorMemoryUrl: "https://www.notion.so/", scriptsDatabaseUrl: "https://www.notion.so/", cvPortfolioUrl: "https://www.notion.so/" },
    knownVideoIdsSkipped: 12,
    metricsRefreshed: 12,
    transcriptionFailures: 0,
    notes: "Preview result — no external provider was called.",
  }),
};

export function isTauriRuntime() {
  return typeof window !== "undefined" && Boolean((window as Window & { __TAURI_INTERNALS__?: unknown }).__TAURI_INTERNALS__);
}
