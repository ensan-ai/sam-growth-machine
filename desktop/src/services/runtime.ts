import { invoke } from "@tauri-apps/api/core";
import type { Activity, Approval, Dashboard, Employee, ProviderStatus, Settings, WorkDetail, WorkItem } from "../types";

export const runtime = {
  dashboard: () => invoke<Dashboard>("get_dashboard"),
  team: () => invoke<Employee[]>("get_team"),
  workItems: () => invoke<WorkItem[]>("get_work_items"),
  workItem: (workItemId:string) => invoke<WorkDetail>("get_work_item", { workItemId }),
  approvals: () => invoke<Approval[]>("get_approvals"),
  activity: () => invoke<Activity>("get_activity"),
  settings: () => invoke<Settings>("get_settings"),
  providerStatus: () => invoke<ProviderStatus>("check_providers"),
  startWorkItem: (title:string,researchSignal:string) => invoke<WorkItem>("start_work_item", { request:{ title,researchSignal } }),
  runWorkItem: (workItemId:string) => invoke<WorkDetail>("run_work_item", { workItemId }),
  decideApproval: (workItemId:string,action:"APPROVE"|"REQUEST_REVISION"|"REJECT",feedback?:string) => invoke<WorkDetail>("decide_approval", { decision:{ workItemId,action,feedback } }),
  control: (action:"RUN"|"PAUSE"|"RESUME"|"STOP") => invoke<Settings>("company_control", { request:{ action } }),
  saveSettings: (settings:Omit<Settings,"companyControl">) => invoke<Settings>("save_settings", { request:settings }),
};
