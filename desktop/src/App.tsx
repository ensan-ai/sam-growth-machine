import { useCallback, useEffect, useState } from "react";
import { Activity as ActivityIcon, Bot, BrainCircuit, CirclePause, CirclePlay, Command, Home, ListChecks, Network, RefreshCw, Settings as SettingsIcon, ShieldCheck, Square, Users, Workflow, X } from "lucide-react";
import { MissionHome, EmployeeTeam, eventLabel } from "./components/MissionHome";
import { ApprovalReview } from "./components/ApprovalReview";
import { Feed } from "./components/Feed";
import { newestFirst } from "./chrono";
import { runtime } from "./services/runtime";
import type { Activity, Approval, Dashboard, Employee, Settings, WorkDetail, WorkItem } from "./types";

type View="HOME"|"TEAM"|"WORK"|"APPROVALS"|"ACTIVITY"|"SETTINGS";
const nav:[View,typeof Home][]=[["HOME",Home],["TEAM",Users],["WORK",Workflow],["APPROVALS",ShieldCheck],["ACTIVITY",ActivityIcon],["SETTINGS",SettingsIcon]];
const fmt=(date?:string)=>date?new Intl.DateTimeFormat(undefined,{month:"short",day:"numeric",hour:"2-digit",minute:"2-digit"}).format(new Date(date)):"—";
const short=(id?:string)=>id?id.split("-").slice(-1)[0].slice(0,8):"—";

export default function App(){
  useEffect(()=>{const visibility=()=>document.documentElement.classList.toggle("page-hidden",document.hidden);document.addEventListener("visibilitychange",visibility);visibility();return()=>document.removeEventListener("visibilitychange",visibility);},[]);
  const [view,setView]=useState<View>("HOME");const [dashboard,setDashboard]=useState<Dashboard>();const [team,setTeam]=useState<Employee[]>([]);const [activity,setActivity]=useState<Activity>({runs:[],events:[]});const [settings,setSettings]=useState<Settings>();const [approvals,setApprovals]=useState<Approval[]>([]);const [selected,setSelected]=useState<string>();const [detail,setDetail]=useState<WorkDetail>();const [error,setError]=useState<string>();const [busy,setBusy]=useState(false);const [newWork,setNewWork]=useState(false);
  const refresh=useCallback(async()=>{try{const [d,t,a,s,ap]=await Promise.all([runtime.dashboard(),runtime.team(),runtime.activity(),runtime.settings(),runtime.approvals()]);setDashboard(d);setTeam(t);setActivity(a);setSettings(s);setApprovals(ap);const id=selected||d.workItems[0]?.workItemId;if(id){setSelected(id);setDetail(await runtime.workItem(id));}setError(undefined);}catch(e){setError(String(e));}},[selected]);
  useEffect(()=>{refresh();const timer=window.setInterval(()=>{if(!document.hidden)refresh();},4000);return()=>clearInterval(timer);},[refresh]);
  const act=async(fn:()=>Promise<unknown>)=>{setBusy(true);setError(undefined);try{await fn();await refresh();}catch(e){setError(String(e));}finally{setBusy(false);}};
  const selectWork=async(id:string)=>{setSelected(id);setDetail(await runtime.workItem(id));setView("WORK");};
  const latestHandoffs=detail?.handoffs||[];
  return <div className={`app-shell view-${view.toLowerCase()}`}>
    <div className="aurora aurora-one"/><div className="aurora aurora-two"/><div className="aurora aurora-three"/>
    <aside className="sidebar glass">
      <div className="brand"><div className="brand-mark"><BrainCircuit size={28} strokeWidth={1}/></div><div><strong>SAM <em>NEURAL CORE</em></strong><span>AURORA MISSION CONTROL</span></div></div>
      <nav>{nav.map(([name,Icon])=><button key={name} className={view===name?"active":""} onClick={()=>setView(name)}><Icon size={18}/><span>{name}</span>{name==="APPROVALS"&&dashboard?.waitingApprovals?<b>{dashboard.waitingApprovals}</b>:null}</button>)}</nav>
      <div className="sidebar-foot"><span className={`status-light ${dashboard?.companyState?.toLowerCase()}`}/><div><small>COMPANY</small><strong>{dashboard?.companyState||"CONNECTING"}</strong></div></div>
    </aside>
    <main>
      <header><div><p className="eyebrow">SAM GROWTH MACHINE</p><h1>{view==="HOME"?"Mission control":view[0]+view.slice(1).toLowerCase()}</h1></div><div className="header-actions"><span className="provider-pill"><i className={dashboard?.providerStatus.ollamaModelAvailable?"tiny-light":"tiny-light blocked"}/>Ollama <span>{dashboard?.providerStatus.ollamaModelAvailable?dashboard.providerStatus.ollamaModel:"Unavailable"}</span></span><button className="icon-button" onClick={refresh} aria-label="Refresh"><RefreshCw size={15} className={busy?"spin":""}/></button><div className="company-controls"><ControlButton icon={CirclePlay} label="Run" onClick={()=>act(()=>runtime.control("RUN"))}/><ControlButton icon={CirclePause} label="Pause" onClick={()=>act(()=>runtime.control("PAUSE"))}/><ControlButton icon={Square} label="Stop" onClick={()=>act(()=>runtime.control("STOP"))}/></div><button className="primary" onClick={()=>setNewWork(true)}>New work item <span>＋</span></button></div></header>
      {error&&<div className="error-banner"><span>{error}</span><button onClick={()=>setError(undefined)}><X size={16}/></button></div>}
      {view==="HOME"&&dashboard&&<MissionHome dashboard={dashboard} team={team} activity={activity} handoffs={latestHandoffs} selectEmployee={()=>setView("TEAM")} selectWork={selectWork} openActivity={()=>setView("ACTIVITY")} openApprovals={()=>setView("APPROVALS")}/>} 
      {view==="TEAM"&&<EmployeeTeam team={team} activity={activity}/>} 
      {view==="WORK"&&<WorkView work={dashboard?.workItems||[]} detail={detail} selectWork={selectWork} run={()=>detail&&act(()=>runtime.runWorkItem(detail.workItem.workItemId))}/>} 
      {view==="APPROVALS"&&<ApprovalReview work={dashboard?.workItems||[]} approvals={approvals} load={runtime.workItem} decide={(id,action,feedback)=>act(()=>runtime.decideApproval(id,action,feedback))}/>} 
      {view==="ACTIVITY"&&<ActivityView activity={activity}/>} 
      {view==="SETTINGS"&&settings&&<SettingsView settings={settings} setSettings={setSettings} save={()=>act(()=>runtime.saveSettings(settings))} check={()=>act(()=>runtime.providerStatus())}/>} 
    </main>
    {newWork&&<NewWork close={()=>setNewWork(false)} submit={(title,signal)=>act(async()=>{const item=await runtime.startWorkItem(title,signal);setNewWork(false);setSelected(item.workItemId);await runtime.runWorkItem(item.workItemId);setView("APPROVALS");})}/>} 
  </div>;
}

function ControlButton({icon:Icon,label,onClick}:{icon:typeof Home;label:string;onClick:()=>void}){return <button className="control" onClick={onClick}><Icon size={15}/>{label}</button>}

function WorkView({work,detail,selectWork,run}:{work:WorkItem[];detail?:WorkDetail;selectWork:(id:string)=>void;run:()=>void}){
 const events=newestFirst(detail?.events||[],e=>e.createdAt);
 const artifacts=newestFirst(detail?.artifacts||[],a=>a.createdAt);
 return <section className="work-layout"><aside className="work-list glass">{work.map(w=><button key={w.workItemId} onClick={()=>selectWork(w.workItemId)} className={detail?.workItem.workItemId===w.workItemId?"selected":""}><State state={w.state}/><strong>{w.title}</strong><small>{w.currentOwner}</small></button>)}</aside>{detail?<div className="work-detail"><article className="glass detail-head"><div><span className="eyebrow">WORK ITEM · {short(detail.workItem.workItemId)}</span><h2>{detail.workItem.title}</h2><p>Current owner: {detail.workItem.currentOwner}</p></div><div><State state={detail.workItem.state}/>{!['READY_FOR_APPROVAL','MEASURED','REJECTED','CANCELLED'].includes(detail.workItem.state)&&<button className="primary" onClick={run}>Continue</button>}</div></article>{detail.workItem.blockedReason&&<div className="blocker">{detail.workItem.blockedReason}</div>}<div className="two-col"><article className="glass panel"><SectionTitle label="TRACE" title="Timeline" icon={ListChecks}/><Feed pinKey={`${events[0]?.eventId||""}:${events.length}`}>{events.map(e=><div className="timeline" key={e.eventId}><i/><div><strong>{e.eventType.replaceAll('.',' ')}</strong><span>{e.actor} · {fmt(e.createdAt)}</span></div></div>)}</Feed></article><article className="glass panel"><SectionTitle label="IMMUTABLE" title="Artifacts" icon={Command}/><Feed pinKey={`artifacts:${artifacts[0]?.artifactId||""}:${artifacts.length}`}>{artifacts.map(a=><details className="artifact" key={a.artifactId}><summary><div><strong>{a.artifactType}</strong><span>{a.producer} · v{a.version}</span></div><code>{short(a.artifactId)}</code></summary><pre>{JSON.stringify(a.payload,null,2)}</pre></details>)}</Feed></article></div></div>:<Empty text="Select a work item"/>}</section>;
}

function ActivityView({activity}:{activity:Activity}){
 const events=newestFirst(activity.events,e=>e.createdAt);
 const execRows=newestFirst([
  ...(activity.executions||[]).map(e=>({key:e.executionId,at:e.startedAt,execution:e})),
  ...activity.runs.map(r=>({key:r.runId,at:r.startedAt,run:r})),
 ],row=>row.at);
 return <section className="two-col activity-layout">
  <article className="glass panel">
   <SectionTitle label="EXECUTION" title="Capability & model runs" icon={Bot}/>
   <Feed pinKey={`exec:${execRows[0]?.key||""}:${execRows.length}`}>
    {execRows.map(row=>{
     if("execution" in row && row.execution){
      const e=row.execution;
      return <div className="run-row" key={e.executionId}><div><strong>{e.role} · {e.capability}</strong><span>{e.kind}{e.provider?` / ${e.provider}`:""}</span><time>Started {fmt(e.startedAt)}{e.finishedAt?` · Finished ${fmt(e.finishedAt)}`:""}</time></div><div><State state={e.success?"SUCCESS":e.finishedAt?"FAILED":"WORKING"}/></div>{e.error&&<p className="run-error">{e.error}</p>}</div>;
     }
     const r=row.run!;
     return <div className="run-row" key={r.runId}><div><strong>{r.agent} · {eventLabel(r.taskType)}</strong><span>{r.provider} / {r.model}</span><time>Started {fmt(r.startedAt)}{r.finishedAt?` · Finished ${fmt(r.finishedAt)}`:""}</time></div><div><State state={r.success?"SUCCESS":r.finishedAt?"FAILED":"WORKING"}/><small>{r.escalationOccurred?"Escalated":"Direct"} · {r.tokenUsage??"—"} tokens</small></div>{r.error&&<p className="run-error">{r.error}</p>}</div>;
    })}
   </Feed>
  </article>
  <article className="glass panel">
   <SectionTitle label="AUDIT" title="System events" icon={Network}/>
   <Feed pinKey={`events:${events[0]?.eventId||""}:${events.length}`}>
    {events.map(e=><div className="timeline" key={e.eventId}><i/><div><strong>{eventLabel(e.eventType)}</strong><span>{e.actor} · {fmt(e.createdAt)}</span><details><summary>Event details</summary><pre>{JSON.stringify(e.detail,null,2)}</pre></details></div></div>)}
   </Feed>
  </article>
 </section>;
}

function SettingsView({settings,setSettings,save,check}:{settings:Settings;setSettings:(s:Settings)=>void;save:()=>void;check:()=>void}){return <section className="settings-grid"><article className="glass panel form-card"><SectionTitle label="LOCAL FIRST" title="Model routing" icon={BrainCircuit}/><label>Ollama endpoint<input value={settings.ollamaEndpoint} onChange={e=>setSettings({...settings,ollamaEndpoint:e.target.value})}/></label><label>Ollama model<input value={settings.ollamaModel} onChange={e=>setSettings({...settings,ollamaModel:e.target.value})}/></label><label>OpenAI model<input value={settings.openaiModel} onChange={e=>setSettings({...settings,openaiModel:e.target.value})}/></label><label className="toggle"><input type="checkbox" checked={settings.allowOpenaiEscalation} onChange={e=>setSettings({...settings,allowOpenaiEscalation:e.target.checked})}/><span/>Allow paid OpenAI escalation after local quality failure</label><p className="safe-note">The API key is read from the secure runtime environment. It is never stored or displayed here.</p><div className="form-actions"><button onClick={check}>Check providers</button><button className="primary" onClick={save}>Save settings</button></div></article><article className="glass panel"><SectionTitle label="CONTROL PLANE" title="Remote-ready boundary" icon={Command}/><p>Status, work, approvals, activity, controls, and Travis instructions are isolated behind application commands. No public network port is exposed.</p><div className="security-stamp"><ShieldCheck/><div><strong>LOCAL ONLY</strong><span>SQLite + Tauri command boundary</span></div></div></article></section>}

function NewWork({close,submit}:{close:()=>void;submit:(title:string,signal:string)=>void}){const [title,setTitle]=useState("Practical AI workflow signal");const [signal,setSignal]=useState("A practical AI research signal about making everyday knowledge work more reliable, measurable, and human-controlled.");return <div className="modal-backdrop"><form className="modal glass" onSubmit={e=>{e.preventDefault();submit(title,signal)}}><button type="button" className="modal-close" onClick={close}><X/></button><span className="eyebrow">NEW COMPANY CYCLE</span><h2>Start with a research signal</h2><label>Work item title<input value={title} onChange={e=>setTitle(e.target.value)} required/></label><label>Practical AI signal<textarea value={signal} onChange={e=>setSignal(e.target.value)} required/></label><p>The company will work until the explicit Sam approval gate.</p><button className="primary" type="submit">Run to approval</button></form></div>}
function SectionTitle({label,title,icon:Icon}:{label:string;title:string;icon:typeof Home}){return <div className="section-title"><div><span>{label}</span><h2>{title}</h2></div><Icon size={20}/></div>}
function State({state}:{state:string}){return <span className={`state state-${state.toLowerCase().replaceAll('_','-')}`}>{state.replaceAll('_',' ')}</span>}
function Empty({text}:{text:string}){return <div className="empty"><BrainCircuit/><strong>{text}</strong><span>The core is quiet and ready.</span></div>}
