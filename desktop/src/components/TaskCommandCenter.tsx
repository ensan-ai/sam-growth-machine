import { useEffect, useMemo, useState } from "react";
import { CheckCircle2, CircleDot, Clock3, Play, Plus, Search, ShieldCheck, Sparkles } from "lucide-react";
import { runtime } from "../services/runtime";
import type { CommandExecutionMode, CommandTask, CommandTaskStatus } from "../types";
import { TaskActivityPanel } from "./TaskActivityPanel";
import { TaskPreparationPanel } from "./TaskPreparationPanel";
import "./TaskCommandCenter.css";

const lanes: {status:CommandTaskStatus; label:string; hint:string}[] = [
  {status:"BLOCKED", label:"Blocked", hint:"Waiting on dependencies"},
  {status:"TODO", label:"Todo", hint:"Ready or prepared"},
  {status:"IN_PROGRESS", label:"In progress", hint:"Being executed"},
  {status:"REVIEW", label:"Review", hint:"Needs Sam review"},
  {status:"DONE", label:"Done", hint:"Completed and unblocked"},
];

export function TaskCommandCenter(){
  const [tasks,setTasks]=useState<CommandTask[]>([]);
  const [selected,setSelected]=useState<CommandTask>();
  const [busy,setBusy]=useState(false);
  const [error,setError]=useState<string>();
  const [creating,setCreating]=useState(false);
  const [search,setSearch]=useState("");

  const refresh=async()=>{
    try{
      const rows=await runtime.commandTasks();
      setTasks(rows);
      setSelected(prev=>rows.find(t=>t.taskId===prev?.taskId) || rows[0]);
      setError(undefined);
    }catch(e){ setError(String(e)); }
  };
  useEffect(()=>{refresh(); const t=window.setInterval(()=>{if(!document.hidden) refresh();},3000); return()=>window.clearInterval(t);},[]);

  const visible=useMemo(()=>tasks.filter(t=>`${t.taskId} ${t.title} ${t.owner} ${t.milestone||""}`.toLowerCase().includes(search.toLowerCase())),[tasks,search]);
  const run=async(fn:()=>Promise<unknown>)=>{setBusy(true);setError(undefined);try{await fn();await refresh();}catch(e){setError(String(e));}finally{setBusy(false);}};

  return <section className="task-command-center">
    <div className="task-toolbar glass">
      <div>
        <span className="eyebrow">MISSION CONTROL</span>
        <h2>Task board</h2>
        <p>{tasks.length} total · {tasks.filter(t=>t.status==="DONE").length} done · {tasks.filter(t=>t.status==="BLOCKED").length} blocked</p>
      </div>
      <div className="task-toolbar-actions">
        <label className="task-search"><Search size={15}/><input value={search} onChange={e=>setSearch(e.target.value)} placeholder="Search tasks"/></label>
        <button className="primary" onClick={()=>setCreating(true)}><Plus size={16}/> New task</button>
      </div>
    </div>
    {error&&<div className="error-banner">{error}</div>}

    <div className="task-board-scroll">
      <div className="task-board">
        {lanes.map(lane=><section className="task-lane glass" key={lane.status}>
          <header><div><strong>{lane.label}</strong><span>{lane.hint}</span></div><b>{visible.filter(t=>t.status===lane.status).length}</b></header>
          <div className="task-stack">
            {visible.filter(t=>t.status===lane.status).map(task=><button className={`task-card ${selected?.taskId===task.taskId?"selected":""}`} key={task.taskId} onClick={()=>setSelected(task)}>
              <div className="task-card-top"><code>{task.taskId}</code><Mode mode={task.executionMode}/></div>
              <strong>{task.title}</strong>
              <p>{task.description || "No description"}</p>
              <div className="task-meta"><span>{task.owner}</span>{task.milestone&&<span>{task.milestone}</span>}<span>P{task.priority}</span></div>
              {task.dependencyIds.length>0&&<small>Depends on {task.dependencyIds.join(", ")}</small>}
              {task.blockedReason&&<small className="blocked-copy">{task.blockedReason}</small>}
              {task.preparedAt&&task.status==="TODO"&&<span className="prepared-badge"><Sparkles size={12}/> Prepared</span>}
            </button>)}
          </div>
        </section>)}
      </div>
    </div>

    {selected&&<aside className="task-inspector glass">
      <div className="task-inspector-head"><div><span className="eyebrow">{selected.taskId}</span><h3>{selected.title}</h3></div><StatusIcon status={selected.status}/></div>
      <p>{selected.description || "No additional description."}</p>
      <dl>
        <div><dt>Status</dt><dd>{selected.status.replaceAll("_"," ")}</dd></div>
        <div><dt>Owner</dt><dd>{selected.owner}</dd></div>
        <div><dt>Mode</dt><dd>{selected.executionMode}</dd></div>
        <div><dt>Milestone</dt><dd>{selected.milestone || "—"}</dd></div>
        <div><dt>Dependencies</dt><dd>{selected.dependencyIds.join(", ") || "None"}</dd></div>
      </dl>

      <TaskPreparationPanel task={selected} onChanged={refresh}/>

      <div className="task-actions">
        {selected.status==="IN_PROGRESS"&&<button className="primary" disabled={busy} onClick={()=>run(()=>runtime.reviewCommandTask(selected.taskId))}><ShieldCheck size={15}/> Send to review</button>}
        {selected.status==="REVIEW"&&<button className="primary" disabled={busy} onClick={()=>run(()=>runtime.completeCommandTask(selected.taskId))}><CheckCircle2 size={15}/> Complete task</button>}
      </div>

      <TaskActivityPanel taskId={selected.taskId}/>
    </aside>}

    {creating&&<CreateTaskModal tasks={tasks} close={()=>setCreating(false)} submit={async data=>{await run(()=>runtime.createCommandTask(data));setCreating(false);}}/>}
  </section>;
}

function Mode({mode}:{mode:CommandExecutionMode}){return <span className={`task-mode mode-${mode.toLowerCase()}`}>{mode}</span>}
function StatusIcon({status}:{status:CommandTaskStatus}){
  if(status==="DONE") return <CheckCircle2/>;
  if(status==="IN_PROGRESS") return <Play/>;
  if(status==="REVIEW") return <ShieldCheck/>;
  if(status==="BLOCKED") return <Clock3/>;
  return <CircleDot/>;
}

function CreateTaskModal({tasks,close,submit}:{tasks:CommandTask[];close:()=>void;submit:(data:{title:string;description:string;owner:string;executionMode:CommandExecutionMode;milestone?:string;priority:number;dependencyIds:string[]})=>Promise<void>}){
  const [title,setTitle]=useState("");
  const [description,setDescription]=useState("");
  const [owner,setOwner]=useState("travis");
  const [mode,setMode]=useState<CommandExecutionMode>("AGENT");
  const [milestone,setMilestone]=useState("");
  const [priority,setPriority]=useState(3);
  const [deps,setDeps]=useState<string[]>([]);
  return <div className="modal-backdrop"><form className="modal glass task-create-modal" onSubmit={async e=>{e.preventDefault();await submit({title,description,owner,executionMode:mode,milestone:milestone||undefined,priority,dependencyIds:deps})}}>
    <span className="eyebrow">COMMAND CENTER</span><h2>Create task</h2>
    <label>Task title<input required value={title} onChange={e=>setTitle(e.target.value)} placeholder="Analyze creator outliers"/></label>
    <label>Description<textarea value={description} onChange={e=>setDescription(e.target.value)} placeholder="What must be true when this task is complete?"/></label>
    <div className="task-form-grid">
      <label>Owner<input value={owner} onChange={e=>setOwner(e.target.value)}/></label>
      <label>Execution mode<select value={mode} onChange={e=>setMode(e.target.value as CommandExecutionMode)}><option>AGENT</option><option>HUMAN</option><option>PAIR</option></select></label>
      <label>Milestone<input value={milestone} onChange={e=>setMilestone(e.target.value)} placeholder="Radar v1"/></label>
      <label>Priority<select value={priority} onChange={e=>setPriority(Number(e.target.value))}>{[1,2,3,4,5].map(n=><option key={n} value={n}>P{n}</option>)}</select></label>
    </div>
    <label>Dependencies<select multiple value={deps} onChange={e=>setDeps(Array.from(e.target.selectedOptions).map(o=>o.value))}>{tasks.filter(t=>t.status!=="DONE").map(t=><option key={t.taskId} value={t.taskId}>{t.taskId} · {t.title}</option>)}</select></label>
    <div className="form-actions"><button type="button" onClick={close}>Cancel</button><button className="primary" type="submit">Create task</button></div>
  </form></div>
}
