import { useEffect, useMemo, useState } from "react";
import { Activity, Bot, CircleCheck, CircleX, UserRound } from "lucide-react";
import { runtime } from "../services/runtime";
import type { CommandTaskEvent } from "../types";

const roleLabel=(actor:string)=>actor.replaceAll("_"," ").replace(/\b\w/g,c=>c.toUpperCase());
const shortEvent=(event:string)=>event.replaceAll("."," · ").replaceAll("_"," ");
const fmt=(date:string)=>new Intl.DateTimeFormat(undefined,{hour:"2-digit",minute:"2-digit",second:"2-digit"}).format(new Date(date));

export function TaskActivityPanel({taskId}:{taskId:string}){
  const [events,setEvents]=useState<CommandTaskEvent[]>([]);
  const [error,setError]=useState<string>();
  const load=async()=>{try{setEvents(await runtime.commandTaskEvents(taskId));setError(undefined);}catch(e){setError(String(e));}};
  useEffect(()=>{load();const timer=window.setInterval(()=>{if(!document.hidden)load();},1500);return()=>window.clearInterval(timer);},[taskId]);

  const roles=useMemo(()=>{
    const map=new Map<string,{state:"ACTIVE"|"DONE"|"FAILED";event:string;at:string}>();
    for(const event of [...events].reverse()){
      const actor=event.actor;
      if(!actor || actor==="sam" || actor==="system") continue;
      const lower=event.eventType.toLowerCase();
      let state:"ACTIVE"|"DONE"|"FAILED"="ACTIVE";
      if(lower.includes("failed")) state="FAILED";
      else if(lower.includes("completed")||lower.includes("prepared")||lower.includes("review_requested")) state="DONE";
      else if(lower.includes("deployed")||lower.includes("started")) state="ACTIVE";
      map.set(actor,{state,event:event.eventType,at:event.createdAt});
    }
    return [...map.entries()];
  },[events]);

  return <section className="task-agent-hub">
    <div className="task-subhead"><div><span className="eyebrow">AGENT HUB</span><strong>Live execution trace</strong></div><Activity size={17}/></div>
    {error&&<small className="blocked-copy">{error}</small>}
    {roles.length>0&&<div className="task-role-grid">{roles.map(([role,state])=><div className={`task-role role-${state.state.toLowerCase()}`} key={role}>
      <span className="role-icon">{role==="orchestrator"?<Bot size={14}/>:<UserRound size={14}/>}</span>
      <div><strong>{roleLabel(role)}</strong><small>{state.state} · {fmt(state.at)}</small></div>
      {state.state==="DONE"?<CircleCheck size={13}/>:state.state==="FAILED"?<CircleX size={13}/>:<i/>}
    </div>)}</div>}
    <div className="task-event-feed">
      {events.slice(0,60).map(event=><details className="task-event" key={event.eventId}>
        <summary><span><i/><strong>{shortEvent(event.eventType)}</strong></span><small>{roleLabel(event.actor)} · {fmt(event.createdAt)}</small></summary>
        <pre>{JSON.stringify(event.detail,null,2)}</pre>
      </details>)}
      {!events.length&&<p className="task-empty-copy">No execution events yet.</p>}
    </div>
  </section>;
}
