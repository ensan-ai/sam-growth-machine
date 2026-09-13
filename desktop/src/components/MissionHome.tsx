import { useMemo } from "react";
import { ArrowUpRight, ArrowRight, CircleCheck, GitBranch } from "lucide-react";
import { NeuralCore } from "./NeuralCore";
import type { Activity, Dashboard, Employee, Handoff, SystemEvent } from "../types";

export const eventLabel=(value:string)=>value.replaceAll("."," ").replaceAll("_"," ").replace(/^\w/,c=>c.toUpperCase());
const time=(value:string)=>new Date(value).toLocaleTimeString([],{hour:"2-digit",minute:"2-digit"});
const tone=(event:SystemEvent)=>/failed|blocked/.test(event.eventType)?"warning":/approval/.test(event.eventType)?"approval":/completed/.test(event.eventType)?"complete":"neutral";
const count=(n:number)=>n.toString().padStart(2,"0");
type Props={dashboard:Dashboard;team:Employee[];activity:Activity;handoffs:Handoff[];selectEmployee:(id:string)=>void;selectWork:(id:string)=>void;openActivity:()=>void;openApprovals:()=>void};
export function MissionHome({dashboard,team,activity,handoffs,selectEmployee,selectWork,openActivity,openApprovals}:Props){
 const latest=dashboard.workItems[0];
 const signals=activity.events.filter(e=>e.eventType!=="agent.started").slice(0,5);
 return <section className="mission-home">
  <div className="mission-body">
   <aside className="summary-rail">
    <p className="eyebrow"><span className="tiny-light"/> THE COMPANY, CONNECTED</p>
    <h2>One vision.<br/>A living<br/><em>intelligence.</em></h2>
    <p className="mission-intro">Seven specialists.<br/>One shared direction.<br/>Every action, a trace.</p>
    <div className="company-numbers"><div><strong>{count(team.length)}</strong><span>Employees</span></div><div><strong>{count(dashboard.activeAgents.length)}</strong><span>Working now</span></div></div>
    <div className="summary-states"><button onClick={openApprovals}><span><i className="tiny-light approval"/>Waiting for Sam</span><strong>{count(dashboard.waitingApprovals)}</strong></button><div><span><i className="tiny-light blocked"/>Blocked work</span><strong>{count(dashboard.blockedWork)}</strong></div><div><span><i className="tiny-light"/>Company</span><strong className="company-word">{dashboard.companyState.toLowerCase()}</strong></div></div>
    <div className="current-cycle"><span className="eyebrow">LATEST CYCLE</span>{latest?<button onClick={()=>selectWork(latest.workItemId)}><span className={`cycle-status ${latest.state.toLowerCase()}`}><CircleCheck size={13}/>{latest.state.replaceAll("_"," ")}</span><strong>{latest.title}</strong><small>Owner · {latest.currentOwner}<ArrowUpRight size={16}/></small></button>:<p>Ready for your first signal.</p>}</div>
   </aside>
   <NeuralCore employees={team} handoffs={handoffs} events={activity.events} waitingApprovals={dashboard.waitingApprovals} onSelect={selectEmployee}/>
   <aside className="signal-rail"><div className="rail-heading"><h2>Latest signals</h2><GitBranch size={16}/></div><p className="rail-subtitle">From your company, in real time</p><div className="signal-items">{signals.length?signals.map(e=><button className="signal-item" key={e.eventId} onClick={()=>e.workItemId?selectWork(e.workItemId):openActivity()}><div><span className={`signal-dot ${tone(e)}`}/><span className="actor">{e.actor}</span><time>{time(e.createdAt)}</time></div><strong>{eventLabel(e.eventType)}</strong><p>{signalDescription(e)}</p></button>):<p className="quiet-note">The signal stream is quiet.<br/>New activity will appear here.</p>}</div><button className="text-link" onClick={openActivity}>View all activity <ArrowRight size={15}/></button></aside>
  </div>
  <ExecutionRhythm events={activity.events} openActivity={openActivity}/>
  <footer className="mission-footer"><span>SAM GROWTH MACHINE <i/> LOCAL INTELLIGENCE</span><span>Human authority at the core.</span><span>{dashboard.providerStatus.openaiConfigured?"OpenAI available by policy":"OpenAI · not configured"}</span></footer>
 </section>;
}
function signalDescription(e:SystemEvent){const d=e.detail as Record<string,unknown>|null;
 if(e.eventType==="cycle.completed")return "Performance reviewed. Learning returned to Travis.";
 if(e.eventType==="agent.output_schema_recovered")return "Contract recovery recorded in the execution trace.";
 if(e.eventType==="publication.mocked")return "Mock publication recorded. No external distribution.";
 if(e.eventType==="performance_snapshot.imported")return "Deterministic performance evidence received.";
 if(e.eventType==="approval.granted")return `Sam approved package version ${d?.package_version??"—"}.`;
 if(e.eventType==="agent.completed")return d?.artifact_type?`${String(d.artifact_type).replaceAll("_"," ").toLowerCase()} delivered.`:"Work completed and recorded.";
 if(e.eventType==="workflow.retrying")return "Resuming from the last saved checkpoint.";
 if(d?.error)return String(d.error);
 return "Recorded in the company activity ledger.";
}
function ExecutionRhythm({events,openActivity}:{events:SystemEvent[];openActivity:()=>void}){
 const data=useMemo(()=>{const times=events.map(e=>new Date(e.createdAt).getTime()).filter(Number.isFinite);const end=times.length?Math.max(...times):Date.now();const start=times.length?Math.min(...times):end-3600000;const span=Math.max(end-start,60000);const bins=Array.from({length:100},()=>({count:0,tone:"neutral",events:[] as SystemEvent[]}));events.forEach(e=>{const index=Math.min(99,Math.max(0,Math.floor((new Date(e.createdAt).getTime()-start)/span*99)));bins[index].count++;bins[index].tone=tone(e);bins[index].events.push(e);});return{bins,start,end,max:Math.max(1,...bins.map(b=>b.count))};},[events]);
 return <div className="execution-rhythm"><div className="rhythm-heading"><div><h2>Execution rhythm</h2><span>{events.length} recorded events · {new Date(data.end).toLocaleDateString([],{day:"2-digit",month:"short"})}</span></div><button onClick={openActivity}>Explore the trace <ArrowUpRight size={14}/></button></div><div className="rhythm-bars" aria-label={`${events.length} actual events from ${time(new Date(data.start).toISOString())} to ${time(new Date(data.end).toISOString())}`}>{data.bins.map((b,i)=><button key={i} className={`rhythm-bin ${b.tone}`} disabled={!b.count} onClick={openActivity} title={b.events.map(e=>`${time(e.createdAt)} · ${eventLabel(e.eventType)}`).join("\n")}><span style={{height:b.count?`${15+b.count/data.max*85}%`:0}}/></button>)}</div><div className="rhythm-times">{Array.from({length:5},(_,i)=><span key={i}>{time(new Date(data.start+(data.end-data.start)*i/4).toISOString())}</span>)}</div></div>;
}
export function EmployeeTeam({team,activity}:{team:Employee[];activity:Activity}){
 return <><div className="page-intro"><span className="eyebrow">THE PEOPLE BEHIND THE INTELLIGENCE</span><h2>Specialists. Working as one.</h2><p>Seven distinct responsibilities, connected by a shared purpose.</p></div><section className="team-grid">{team.map((e,index)=>{const run=activity.runs.find(r=>r.agent===e.employeeId&&!r.finishedAt);const event=activity.events.find(a=>a.actor===e.employeeId);return <article className={`employee-card glass employee-${e.employeeId}`} key={e.employeeId}><div className="employee-top"><div className={`employee-orb ${e.runtimeStatus.toLowerCase()}`}><span>{e.name.slice(0,1)}</span></div><span className="employee-index">{count(index+1)} / 07</span><span className={`state state-${e.runtimeStatus.toLowerCase()}`}>{e.runtimeStatus}</span></div><h2>{e.name}</h2><p className="employee-role">{e.title}</p><div className="employee-report"><GitBranch size={13}/> Reports to <strong>{e.reportsTo}</strong></div><div className="employee-current"><span className="eyebrow">{run?"WORKING ON":"LAST SIGNAL"}</span><strong>{run?eventLabel(run.taskType):event?eventLabel(event.eventType):"Ready for an assignment"}</strong><small>{run?`${run.provider} · ${run.model}`:event?time(event.createdAt):"No recorded activity"}</small></div><div className="employee-bottom"><span>Definition v{e.version}</span><span>{e.definitionStatus}</span></div></article>;})}</section></>;
}
