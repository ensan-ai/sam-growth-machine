import { invoke } from "@tauri-apps/api/core";
import type { Activity, Approval, CommandTask, CommandTaskEvent, CommandTaskPreparation, CreateCommandTask, Dashboard, Employee, PrepareCommandTaskResult, ProviderStatus, Settings, WorkDetail, WorkItem } from "../types";

const invokePrepare = (taskId:string) => invoke<PrepareCommandTaskResult>("prepare_command_task", { taskId });

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

  commandTasks: () => invoke<CommandTask[]>("get_command_tasks"),
  commandTaskEvents: (taskId?:string) => invoke<CommandTaskEvent[]>("get_command_task_events", { taskId }),
  commandTaskPreparation: (taskId:string) => invoke<CommandTaskPreparation|null>("get_command_task_preparation", { taskId }),
  createCommandTask: (request:CreateCommandTask) => invoke<CommandTask>("create_command_task", { request }),
  prepareCommandTask: async (taskId:string) => {
    const first=await invokePrepare(taskId);
    if(first.preparation.state!=="NEEDS_OPERATOR_INPUT") return first;
    const question=first.preparation.operatorQuestion || "The preparation agents need an operator decision before they can continue.";
    const recommendation=first.preparation.operatorRecommendation ? `\n\nRecommendation: ${first.preparation.operatorRecommendation}` : "";
    const decision=window.prompt(`${question}${recommendation}\n\nEnter your decision:`);
    if(!decision?.trim()) return first;
    await invoke<CommandTaskPreparation>("answer_command_task_preparation", { taskId,decision:decision.trim() });
    return invokePrepare(taskId);
  },
  answerCommandTaskPreparation: (taskId:string,decision:string) => invoke<CommandTaskPreparation>("answer_command_task_preparation", { taskId,decision }),
  startCommandTask: (taskId:string) => invoke<CommandTask>("start_command_task", { taskId }),
  reviewCommandTask: (taskId:string) => invoke<CommandTask>("review_command_task", { taskId }),
  completeCommandTask: (taskId:string) => invoke<CommandTask>("complete_command_task", { taskId }),
};
