import { useEffect, useState } from "react";
import { Play, Sparkles } from "lucide-react";
import { runtime } from "../services/runtime";
import type { CommandTask, CommandTaskPreparation } from "../types";

export function TaskPreparationPanel({task,onChanged}:{task:CommandTask;onChanged:()=>Promise<void>}){
  const [prep,setPrep]=useState<CommandTaskPreparation|null>(null);
  const [decision,setDecision]=useState("");
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState<string>();

  const load=async()=>{try{setPrep(await runtime.commandTaskPreparation(task.taskId));setError(undefined);}catch(e){setError(String(e));}};
  useEffect(()=>{load();setDecision("");},[task.taskId,task.updatedAt]);
  const act=async(fn:()=>Promise<unknown>)=>{setBusy(true);setError(undefined);try{await fn();await onChanged();await load();}catch(e){setError(String(e));}finally{setBusy(false);}};
  const continueWithDecision=()=>act(async()=>{await runtime.answerCommandTaskPreparation(task.taskId,decision.trim());await runtime.prepareCommandTask(task.taskId);setDecision("");});

  return <div className="task-preparation-panel">
    {error&&<div className="error-banner">{error}</div>}
    <dl><div><dt>Prepare state</dt><dd>{prep?.state?.replaceAll("_"," ") || (task.preparedAt?"PREPARED":"NOT STARTED")}</dd></div></dl>

    {prep?.explorerOutput&&<details className="task-prompt prep-report"><summary>Explorer report</summary><pre>{JSON.stringify(prep.explorerOutput,null,2)}</pre></details>}
    {prep?.researcherOutput&&<details className="task-prompt prep-report"><summary>Researcher report</summary><pre>{JSON.stringify(prep.researcherOutput,null,2)}</pre></details>}
    {prep?.orchestratorOutput&&<details className="task-prompt prep-report"><summary>Orchestrator plan</summary><pre>{JSON.stringify(prep.orchestratorOutput,null,2)}</pre></details>}
    {task.promptMarkdown&&<details className="task-prompt" open><summary>Prepared execution prompt</summary><pre>{task.promptMarkdown}</pre></details>}

    {prep?.state==="NEEDS_OPERATOR_INPUT"&&<div className="operator-input-card">
      <span className="eyebrow">OPERATOR INPUT REQUIRED</span>
      <strong>{prep.operatorQuestion}</strong>
      {prep.operatorRecommendation&&<p><b>Recommendation:</b> {prep.operatorRecommendation}</p>}
      <textarea value={decision} onChange={e=>setDecision(e.target.value)} placeholder="Sam's decision…"/>
      <button className="primary" disabled={busy||!decision.trim()} onClick={continueWithDecision}>Submit & continue prepare</button>
    </div>}

    {prep?.state==="FAILED"&&<div className="operator-input-card prepare-failed"><span className="eyebrow">PREPARE FAILED</span><strong>{prep.error||"Unknown preparation error"}</strong><button disabled={busy} onClick={()=>act(()=>runtime.prepareCommandTask(task.taskId))}>Retry prepare</button></div>}

    {task.status==="TODO"&&!task.preparedAt&&prep?.state!=="NEEDS_OPERATOR_INPUT"&&<button disabled={busy} onClick={()=>act(()=>runtime.prepareCommandTask(task.taskId))}><Sparkles size={15}/> {busy?"Preparing…":"Prepare task"}</button>}
    {task.status==="TODO"&&task.preparedAt&&prep?.state==="PREPARED"&&<button className="primary" disabled={busy} onClick={()=>act(()=>runtime.startCommandTask(task.taskId))}><Play size={15}/> Start task</button>}
  </div>;
}
