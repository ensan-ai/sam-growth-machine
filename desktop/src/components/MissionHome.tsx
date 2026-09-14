import { GitBranch } from "lucide-react";
import { RubricCommandHome } from "./RubricCommandHome";
import { newestFirst } from "../chrono";
import type { Activity, Dashboard, Employee, Handoff } from "../types";

export const eventLabel=(value:string)=>value.replaceAll("."," ").replaceAll("_"," ").replace(/^\w/,c=>c.toUpperCase());
const time=(value:string)=>new Date(value).toLocaleTimeString([],{hour:"2-digit",minute:"2-digit"});
const count=(n:number)=>n.toString().padStart(2,"0");

type Props={dashboard:Dashboard;team:Employee[];activity:Activity;handoffs:Handoff[];selectEmployee:(id:string)=>void;selectWork:(id:string)=>void;openActivity:()=>void;openApprovals:()=>void};

export function MissionHome(props:Props){
 return <RubricCommandHome {...props}/>;
}

export function EmployeeTeam({team,activity}:{team:Employee[];activity:Activity}){
 const events=newestFirst(activity.events,e=>e.createdAt);
 const runs=newestFirst(activity.runs,r=>r.startedAt);
 return <><div className="page-intro"><span className="eyebrow">THE PEOPLE BEHIND THE INTELLIGENCE</span><h2>Specialists. Working as one.</h2><p>Seven distinct responsibilities, connected by a shared purpose.</p></div><section className="team-grid">{team.map((e,index)=>{const run=runs.find(r=>r.agent===e.employeeId&&!r.finishedAt);const event=events.find(a=>a.actor===e.employeeId);return <article className={`employee-card glass employee-${e.employeeId}`} key={e.employeeId}><div className="employee-top"><div className={`employee-orb ${e.runtimeStatus.toLowerCase()}`}><span>{e.name.slice(0,1)}</span></div><span className="employee-index">{count(index+1)} / 07</span><span className={`state state-${e.runtimeStatus.toLowerCase()}`}>{e.runtimeStatus}</span></div><h2>{e.name}</h2><p className="employee-role">{e.title}</p><div className="employee-report"><GitBranch size={13}/> Reports to <strong>{e.reportsTo}</strong></div><div className="employee-current"><span className="eyebrow">{run?"WORKING ON":"LAST SIGNAL"}</span><strong>{run?eventLabel(run.taskType):event?eventLabel(event.eventType):"Ready for an assignment"}</strong><small>{run?`${run.provider} · ${run.model}`:event?time(event.createdAt):"No recorded activity"}</small></div><div className="employee-bottom"><span>Definition v{e.version}</span><span>{e.definitionStatus}</span></div></article>;})}</section></>;
}
