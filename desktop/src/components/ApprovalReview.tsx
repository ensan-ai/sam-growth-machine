import { useEffect, useState } from "react";
import { Check, ShieldCheck } from "lucide-react";
import { Feed } from "./Feed";
import { newestFirst } from "../chrono";
import type { Approval, WorkDetail, WorkItem } from "../types";

type Props={work:WorkItem[];approvals:Approval[];load:(id:string)=>Promise<WorkDetail>;decide:(id:string,a:"APPROVE"|"REQUEST_REVISION"|"REJECT",f?:string)=>void};
const object=(value:unknown):Record<string,unknown>=>value&&typeof value==="object"?value as Record<string,unknown>:{};
const list=(value:unknown)=>Array.isArray(value)?value.map(v=>typeof v==="string"?v:JSON.stringify(v)):[];
export function ApprovalReview({work,approvals,load,decide}:Props){
 const waiting=work.filter(w=>w.state==="READY_FOR_APPROVAL");const [chosen,setChosen]=useState<string>();const id=waiting.some(w=>w.workItemId===chosen)?chosen:waiting[0]?.workItemId;
 const [detail,setDetail]=useState<WorkDetail>();const [feedback,setFeedback]=useState("");const [error,setError]=useState<string>();const [pending,setPending]=useState(false);
 const stamp=work.find(w=>w.workItemId===id)?.updatedAt;
 useEffect(()=>{let current=true;setDetail(undefined);setError(undefined);setFeedback("");setPending(false);if(id)load(id).then(d=>{if(current)setDetail(d);}).catch(e=>{if(current)setError(String(e));});return()=>{current=false;};},[id,stamp,load]);
 const latest=(type:string)=>detail?.artifacts.filter(a=>a.artifactType===type).sort((a,b)=>b.version-a.version)[0];
 const draft=latest("CONTENT_DRAFT");
 const visual=latest("NO_VISUAL_REQUIRED")||latest("CREATIVE_PACKAGE");
 const pack=latest("PUBLISH_PACKAGE");
 const payload=object(pack?.payload);
 const targetPlatforms=Array.isArray(payload.target_platforms)&&payload.target_platforms.length?payload.target_platforms:payload.intended_platforms;
 const platforms=list(targetPlatforms);
 const noVisual=visual?.artifactType==="NO_VISUAL_REQUIRED";
 const flags=[...list(payload.approval_sensitive_flags),...list(object(visual?.payload).approval_sensitive_items),...list(object(draft?.payload).claims_requiring_verification),...(noVisual?["NO VISUAL REQUIRED"]: [])];
 const act=async(action:"APPROVE"|"REQUEST_REVISION"|"REJECT")=>{if(!id||pending)return;setPending(true);try{await decide(id,action,feedback||undefined);}finally{setPending(false);}};
 const ledger=newestFirst(approvals,a=>a.createdAt).slice(0,8);
 return <section className="approval-list">
  {waiting.length>1&&<div className="approval-selector">{waiting.map(w=><button key={w.workItemId} onClick={()=>setChosen(w.workItemId)} className={w.workItemId===id?"selected":""}>{w.title}</button>)}</div>}
  {id?<article className="approval-card glass"><div className="approval-title"><div><span className="eyebrow">YOUR JUDGMENT. THE FINAL GATE.</span><h2>{detail?.workItem.title||"Loading review package…"}</h2></div><span className="state state-ready-for-approval">WAITING FOR SAM</span></div>{error&&<p className="blocker">{error}</p>}<div className="version-bar"><span>Copy v{draft?.version??"—"}</span><span>{noVisual?"No visual":"Creative"} v{visual?.version??"—"}</span><span>Package v{pack?.version??"—"}</span>{platforms.map(p=><span key={p}>{p}</span>)}</div><div className="approval-columns"><Review title="Strategy" value={latest("CONTENT_BRIEF")?.payload}/><Review title="Final copy" value={draft?.payload}/><Review title={noVisual?"Jax: no visual required":"Creative"} value={visual?.payload}/></div><div className="approval-flags"><ShieldCheck size={14}/><span>Approval applies to the exact versions shown above.</span>{flags.map((f,i)=><span key={i}>{String(f).replaceAll("_"," ")}</span>)}</div><textarea aria-label="Revision or rejection guidance" value={feedback} onChange={e=>setFeedback(e.target.value)} placeholder="Optional revision or rejection guidance"/><div className="approval-actions"><button className="approve" disabled={!draft||!visual||!pack||pending} onClick={()=>act("APPROVE")}><Check size={15}/>{pending?"Applying decision…":"Approve exact versions"}</button><button disabled={!detail||pending} onClick={()=>act("REQUEST_REVISION")}>Request revision</button><button className="reject" disabled={!detail||pending} onClick={()=>act("REJECT")}>Reject</button></div></article>:<div className="empty"><ShieldCheck/><strong>Every decision has its moment.</strong><span>No packages are waiting for Sam's approval.</span></div>}
  <article className="glass panel"><div className="section-title"><div><span>DECISION LEDGER</span><h2>Approval history</h2></div><ShieldCheck size={20}/></div>{ledger.length?<Feed pinKey={`approvals:${ledger[0]?.approvalId||""}:${ledger.length}`}>{ledger.map(a=><details key={a.approvalId}><summary className="ledger-row"><span className={`state state-${a.status.toLowerCase()}`}>{a.status}</span><span>{a.approver} · {a.workItemId.slice(-8)}</span><span>Package v{a.packageVersion}</span><time>{new Date(a.createdAt).toLocaleString([],{month:"short",day:"numeric",hour:"2-digit",minute:"2-digit"})}</time></summary><div className="version-bar"><span>Copy v{a.contentVersion}</span><span>Creative v{a.creativeVersion}</span><span>Package v{a.packageVersion}</span>{a.platformScope.map(p=><span key={p}>{p}</span>)}</div><p className="safe-note">{a.feedback||"No additional feedback recorded."}</p></details>)}</Feed>:<p className="quiet-note">Your decisions will appear here.</p>}</article>
 </section>;
}
function Review({title,value}:{title:string;value:unknown}){const record=object(value);const keys=["title","core_idea","audience_problem","core_takeaway","hook","body","cta","creative_objective","visual_direction","visual_decision","reasoning"];const texts=keys.filter(k=>typeof record[k]==="string");return <div className="review-block"><span>{title}</span><div className="review-prose">{texts.length?texts.map(k=><p key={k}>{String(record[k])}</p>):<p>{value?"See the full artifact for this package.":"Loading…"}</p>}</div><details><summary>Full artifact</summary><pre>{JSON.stringify(value,null,2)}</pre></details></div>;}
