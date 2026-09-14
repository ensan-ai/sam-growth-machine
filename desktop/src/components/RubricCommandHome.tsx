import { useEffect, useMemo, useRef, useState } from "react";
import { Activity, ArrowUpRight, Focus, Minus, Plus, ShieldCheck, Users, Workflow } from "lucide-react";
import type { Activity as ActivityData, Dashboard, Employee, Handoff, SystemEvent, WorkItem } from "../types";
import "./RubricCommandHome.css";

type NodeKind = "core" | "employee" | "work" | "signal";
type GraphNode = {
  id:string; kind:NodeKind; label:string; sublabel:string; x:number; y:number; vx:number; vy:number;
  ax:number; ay:number; radius:number; color:string; status?:string; employee?:Employee; work?:WorkItem; event?:SystemEvent;
};
type Edge = { a:string; b:string; kind:"hierarchy"|"owner"|"signal"|"handoff"; createdAt?:string };
type Props = {
  dashboard:Dashboard; team:Employee[]; activity:ActivityData; handoffs:Handoff[];
  selectEmployee:(id:string)=>void; selectWork:(id:string)=>void; openActivity:()=>void; openApprovals:()=>void;
};

const EMPLOYEE_RING=170, WORK_RING=300, SIGNAL_RING=405;
const employeeOrder=["travis","saly","adam","brain","jax","maro","lara"];
const hierarchy:[[string,string]] extends never ? never : Array<[string,string]> = [
  ["sam","travis"],["travis","saly"],["travis","adam"],["travis","maro"],["travis","lara"],["adam","brain"],["adam","jax"]
];
const ownerIds=(owner:string)=>owner.toLowerCase().split(/\s*\+\s*|\s*,\s*|\s+and\s+/).map(v=>v.trim()).filter(Boolean);
const eventLabel=(value:string)=>value.replaceAll("."," ").replaceAll("_"," ").replace(/^\w/,c=>c.toUpperCase());
const statusColor=(state?:string)=>{
  const s=(state||"").toUpperCase();
  if(/BLOCK|FAIL|REJECT/.test(s))return "#ff987c";
  if(/APPROVAL|REVIEW|WAIT/.test(s))return "#ffd48a";
  if(/WORK|PROGRESS|RUN/.test(s))return "#72f1d0";
  if(/DONE|SUCCESS|MEASURED|COMPLETE/.test(s))return "#9bd8ff";
  return "#79b8db";
};
const hash=(s:string)=>{let h=2166136261;for(let i=0;i<s.length;i++)h=Math.imul(h^s.charCodeAt(i),16777619);return Math.abs(h>>>0);};
const ringPoint=(index:number,total:number,radius:number,offset=0)=>{const a=(index/Math.max(1,total))*Math.PI*2-Math.PI/2+offset;return{x:Math.cos(a)*radius,y:Math.sin(a)*radius};};

function buildGraph(team:Employee[], work:WorkItem[], events:SystemEvent[], handoffs:Handoff[]){
  const nodes:GraphNode[]=[]; const edges:Edge[]=[];
  nodes.push({id:"sam",kind:"core",label:"SAM",sublabel:"HUMAN AUTHORITY",x:0,y:0,vx:0,vy:0,ax:0,ay:0,radius:46,color:"#76f1dd",status:"AUTHORITY"});
  const sorted=[...team].sort((a,b)=>employeeOrder.indexOf(a.employeeId)-employeeOrder.indexOf(b.employeeId));
  sorted.forEach((e,i)=>{const p=ringPoint(i,sorted.length,EMPLOYEE_RING,.12);nodes.push({id:e.employeeId,kind:"employee",label:e.name,sublabel:e.title,x:p.x,y:p.y,vx:0,vy:0,ax:p.x,ay:p.y,radius:e.employeeId==="travis"?18:13,color:statusColor(e.runtimeStatus),status:e.runtimeStatus,employee:e});});
  hierarchy.forEach(([a,b])=>{if(nodes.some(n=>n.id===a)&&nodes.some(n=>n.id===b))edges.push({a,b,kind:"hierarchy"});});

  const visibleWork=work.slice(0,12);
  visibleWork.forEach((w,i)=>{const p=ringPoint(i,visibleWork.length,WORK_RING,-.07);nodes.push({id:`work:${w.workItemId}`,kind:"work",label:w.title,sublabel:w.state.replaceAll("_"," "),x:p.x,y:p.y,vx:0,vy:0,ax:p.x,ay:p.y,radius:8,color:statusColor(w.state),status:w.state,work:w});ownerIds(w.currentOwner).forEach(owner=>{if(nodes.some(n=>n.id===owner))edges.push({a:owner,b:`work:${w.workItemId}`,kind:"owner"});});});

  const visibleEvents=[...events].sort((a,b)=>Date.parse(b.createdAt)-Date.parse(a.createdAt)).slice(0,18);
  visibleEvents.forEach((e,i)=>{const base=ringPoint(i,visibleEvents.length,SIGNAL_RING,.18);const jitter=(hash(e.eventId)%21)-10;const x=base.x+(base.x?Math.sign(base.x)*jitter:0),y=base.y+(base.y?Math.sign(base.y)*jitter:0);nodes.push({id:`event:${e.eventId}`,kind:"signal",label:eventLabel(e.eventType),sublabel:e.actor,x,y,vx:0,vy:0,ax:x,ay:y,radius:4.2,color:statusColor(e.eventType),event:e});const actor=e.actor.toLowerCase();if(nodes.some(n=>n.id===actor))edges.push({a:actor,b:`event:${e.eventId}`,kind:"signal"});else if(e.workItemId&&nodes.some(n=>n.id===`work:${e.workItemId}`))edges.push({a:`work:${e.workItemId}`,b:`event:${e.eventId}`,kind:"signal"});});

  handoffs.slice(-18).forEach(h=>{const a=h.sender.toLowerCase(),b=h.receiver.toLowerCase();if(nodes.some(n=>n.id===a)&&nodes.some(n=>n.id===b))edges.push({a,b,kind:"handoff",createdAt:h.createdAt});});
  return {nodes,edges};
}

export function RubricCommandHome({dashboard,team,activity,handoffs,selectEmployee,selectWork,openActivity,openApprovals}:Props){
  const host=useRef<HTMLDivElement>(null); const canvas=useRef<HTMLCanvasElement>(null);
  const graph=useMemo(()=>buildGraph(team,dashboard.workItems,activity.events,handoffs),[team,dashboard.workItems,activity.events,handoffs]);
  const live=useRef(graph); live.current=graph;
  const camera=useRef({x:0,y:0,zoom:1});
  const [selectedId,setSelectedId]=useState("sam"); const [hoverId,setHoverId]=useState<string>(); const [,forceUi]=useState(0);
  const selected=graph.nodes.find(n=>n.id===selectedId)||graph.nodes[0];

  useEffect(()=>{
    const el=host.current,c=canvas.current;if(!el||!c)return;const ctx=c.getContext("2d");if(!ctx)return;
    let raf=0,last=0,dragging=false,moved=false,start={x:0,y:0},origin={x:0,y:0};
    const dpr=Math.min(window.devicePixelRatio||1,2);
    const resize=()=>{const r=el.getBoundingClientRect();c.width=Math.max(1,Math.floor(r.width*dpr));c.height=Math.max(1,Math.floor(r.height*dpr));c.style.width=`${r.width}px`;c.style.height=`${r.height}px`;};
    const observer=new ResizeObserver(resize);observer.observe(el);resize();
    const world=(clientX:number,clientY:number)=>{const r=c.getBoundingClientRect(),cam=camera.current;return{x:(clientX-r.left-r.width/2)/cam.zoom-cam.x,y:(clientY-r.top-r.height/2)/cam.zoom-cam.y};};
    const hit=(clientX:number,clientY:number)=>{const p=world(clientX,clientY);let best:GraphNode|undefined,dist=Infinity;for(const n of live.current.nodes){const d=Math.hypot(n.x-p.x,n.y-p.y);const threshold=Math.max(12,n.radius+7);if(d<threshold&&d<dist){dist=d;best=n;}}return best;};
    const onMove=(e:PointerEvent)=>{if(dragging){const dx=e.clientX-start.x,dy=e.clientY-start.y;if(Math.abs(dx)+Math.abs(dy)>3)moved=true;camera.current.x=origin.x+dx/camera.current.zoom;camera.current.y=origin.y+dy/camera.current.zoom;return;}const n=hit(e.clientX,e.clientY);setHoverId(n?.id);c.style.cursor=n?"pointer":"grab";};
    const onDown=(e:PointerEvent)=>{dragging=true;moved=false;start={x:e.clientX,y:e.clientY};origin={x:camera.current.x,y:camera.current.y};c.setPointerCapture(e.pointerId);c.style.cursor="grabbing";};
    const onUp=(e:PointerEvent)=>{if(!dragging)return;dragging=false;c.releasePointerCapture(e.pointerId);const n=!moved?hit(e.clientX,e.clientY):undefined;if(n){setSelectedId(n.id);forceUi(v=>v+1);}c.style.cursor=n?"pointer":"grab";};
    const onWheel=(e:WheelEvent)=>{e.preventDefault();camera.current.zoom=Math.max(.65,Math.min(1.65,camera.current.zoom*(e.deltaY>0?.92:1.08)));forceUi(v=>v+1);};
    c.addEventListener("pointermove",onMove);c.addEventListener("pointerdown",onDown);c.addEventListener("pointerup",onUp);c.addEventListener("wheel",onWheel,{passive:false});

    const draw=(ms:number)=>{raf=requestAnimationFrame(draw);if(document.hidden||ms-last<1000/30)return;last=ms;const {nodes,edges}=live.current;const r=c.getBoundingClientRect();ctx.setTransform(dpr,0,0,dpr,0,0);ctx.clearRect(0,0,r.width,r.height);ctx.save();ctx.translate(r.width/2,r.height/2);const cam=camera.current;ctx.scale(cam.zoom,cam.zoom);ctx.translate(cam.x,cam.y);
      // Ambient field and semantic rings.
      const g=ctx.createRadialGradient(0,0,20,0,0,SIGNAL_RING+130);g.addColorStop(0,"rgba(34,116,123,.14)");g.addColorStop(.45,"rgba(18,54,78,.07)");g.addColorStop(1,"rgba(0,0,0,0)");ctx.fillStyle=g;ctx.beginPath();ctx.arc(0,0,SIGNAL_RING+130,0,Math.PI*2);ctx.fill();
      [[EMPLOYEE_RING,"EMPLOYEES"],[WORK_RING,"WORK"],[SIGNAL_RING,"SIGNALS"]].forEach(([rad,label])=>{ctx.beginPath();ctx.arc(0,0,rad as number,0,Math.PI*2);ctx.strokeStyle="rgba(122,184,205,.11)";ctx.lineWidth=1;ctx.stroke();ctx.fillStyle="rgba(141,191,208,.42)";ctx.font="8px ui-monospace, SFMono-Regular, Menlo, monospace";ctx.letterSpacing="1px";ctx.fillText(label as string,10,-(rad as number)+4);});
      // Gentle attraction to each semantic layer keeps the graph alive without becoming random.
      nodes.forEach(n=>{if(n.kind==="core")return;const dx=n.ax-n.x,dy=n.ay-n.y;n.vx=(n.vx+dx*.0022)*.92;n.vy=(n.vy+dy*.0022)*.92;n.x+=n.vx;n.y+=n.vy;});
      // Connections.
      const byId=new Map(nodes.map(n=>[n.id,n]));edges.forEach(e=>{const a=byId.get(e.a),b=byId.get(e.b);if(!a||!b)return;ctx.beginPath();ctx.moveTo(a.x,a.y);const mx=(a.x+b.x)/2,my=(a.y+b.y)/2;const bend=(hash(e.a+e.b)%19)-9;ctx.quadraticCurveTo(mx-bend,my+bend,b.x,b.y);ctx.strokeStyle=e.kind==="handoff"?"rgba(111,241,211,.34)":e.kind==="hierarchy"?"rgba(126,201,220,.24)":"rgba(105,152,178,.12)";ctx.lineWidth=e.kind==="handoff"?1.25:.7;ctx.stroke();if(e.kind==="handoff"&&e.createdAt){const age=Date.now()-Date.parse(e.createdAt);if(age>=0&&age<12000){const t=(age%4000)/4000;const x=(1-t)*(1-t)*a.x+2*(1-t)*t*(mx-bend)+t*t*b.x,y=(1-t)*(1-t)*a.y+2*(1-t)*t*(my+bend)+t*t*b.y;ctx.fillStyle="#baffec";ctx.shadowColor="#78f5d6";ctx.shadowBlur=10;ctx.beginPath();ctx.arc(x,y,2.4,0,Math.PI*2);ctx.fill();ctx.shadowBlur=0;}}});
      // Nodes.
      nodes.forEach(n=>{const active=n.id===selectedId||n.id===hoverId;const pulse=n.kind==="core"?1+Math.sin(ms/1100)*.045:1;const radius=n.radius*pulse;ctx.save();ctx.translate(n.x,n.y);ctx.globalAlpha=n.kind==="signal"?.72:1;ctx.shadowColor=n.color;ctx.shadowBlur=active?24:n.kind==="core"?28:10;const ng=ctx.createRadialGradient(-radius*.28,-radius*.3,1,0,0,radius*1.6);ng.addColorStop(0,"rgba(235,255,255,.92)");ng.addColorStop(.12,n.color);ng.addColorStop(.55,`${n.color}66`);ng.addColorStop(1,"rgba(4,12,20,0)");ctx.fillStyle=ng;ctx.beginPath();ctx.arc(0,0,radius*(n.kind==="core"?1.75:1.45),0,Math.PI*2);ctx.fill();ctx.shadowBlur=0;ctx.fillStyle=n.kind==="core"?"rgba(6,23,31,.94)":"rgba(8,23,34,.92)";ctx.strokeStyle=active?n.color:`${n.color}88`;ctx.lineWidth=active?1.6:.8;ctx.beginPath();ctx.arc(0,0,radius,0,Math.PI*2);ctx.fill();ctx.stroke();if(n.kind==="core"){ctx.fillStyle="#dffff7";ctx.textAlign="center";ctx.textBaseline="middle";ctx.font="500 17px -apple-system, BlinkMacSystemFont, sans-serif";ctx.fillText("SAM",0,-2);}ctx.restore();
        if(n.kind!=="signal"&&(active||n.kind==="employee"||n.kind==="core")){ctx.fillStyle=active?"#efffff":"rgba(205,235,242,.82)";ctx.font=n.kind==="core"?"500 10px -apple-system, sans-serif":"500 10px -apple-system, sans-serif";ctx.textAlign=n.x>=0?"left":"right";const offset=radius+9;ctx.fillText(n.kind==="core"?"HUMAN AUTHORITY":n.label,n.x+(n.x>=0?offset:-offset),n.y-2);if(n.kind==="employee"){ctx.fillStyle="rgba(133,166,181,.72)";ctx.font="7px ui-monospace, monospace";ctx.fillText((n.status||"").replaceAll("_"," "),n.x+(n.x>=0?offset:-offset),n.y+10);}}
      });ctx.restore();};
    raf=requestAnimationFrame(draw);
    return()=>{cancelAnimationFrame(raf);observer.disconnect();c.removeEventListener("pointermove",onMove);c.removeEventListener("pointerdown",onDown);c.removeEventListener("pointerup",onUp);c.removeEventListener("wheel",onWheel);};
  },[]);

  const zoom=(delta:number)=>{camera.current.zoom=Math.max(.65,Math.min(1.65,camera.current.zoom+delta));forceUi(v=>v+1);};
  const reset=()=>{camera.current={x:0,y:0,zoom:1};forceUi(v=>v+1);};
  const action=()=>{if(selected.kind==="employee"&&selected.employee)selectEmployee(selected.employee.employeeId);else if(selected.kind==="work"&&selected.work)selectWork(selected.work.workItemId);else if(selected.kind==="signal")selected.event?.workItemId?selectWork(selected.event.workItemId):openActivity();else openApprovals();};
  return <section className="rubric-command-home">
    <div className="rubric-graph-brand"><span>SAM</span> NEURAL GROWTH<small>Live company topology</small></div>
    <div className="rubric-layer-key"><span><i className="layer-employee"/>Employees</span><span><i className="layer-work"/>Work</span><span><i className="layer-signal"/>Signals</span></div>
    <div className="rubric-canvas-host" ref={host}><canvas ref={canvas}/></div>
    <div className="rubric-stats">
      <button onClick={()=>selectedId!=="sam"&&setSelectedId("sam")}><Users size={14}/><span><strong>{team.length}</strong> employees</span></button>
      <button onClick={openActivity}><Activity size={14}/><span><strong>{activity.events.length}</strong> signals</span></button>
      <button onClick={()=>dashboard.workItems[0]&&selectWork(dashboard.workItems[0].workItemId)}><Workflow size={14}/><span><strong>{dashboard.workItems.length}</strong> work items</span></button>
      <button onClick={openApprovals}><ShieldCheck size={14}/><span><strong>{dashboard.waitingApprovals}</strong> waiting for Sam</span></button>
    </div>
    <aside className="rubric-node-inspector">
      <span className="inspector-kicker">{selected.kind.toUpperCase()}</span><h2>{selected.label}</h2><p>{selected.sublabel}</p>
      <div className="inspector-meta"><span>Status</span><strong style={{color:selected.color}}>{selected.status||selected.kind}</strong></div>
      {selected.kind==="employee"&&selected.employee&&<><div className="inspector-meta"><span>Reports to</span><strong>{selected.employee.reportsTo}</strong></div><div className="inspector-meta"><span>Definition</span><strong>v{selected.employee.version}</strong></div></>}
      {selected.kind==="work"&&selected.work&&<><div className="inspector-meta"><span>Owner</span><strong>{selected.work.currentOwner}</strong></div><div className="inspector-meta"><span>Updated</span><strong>{new Date(selected.work.updatedAt).toLocaleTimeString([],{hour:"2-digit",minute:"2-digit"})}</strong></div></>}
      {selected.kind==="signal"&&selected.event&&<><div className="inspector-meta"><span>Actor</span><strong>{selected.event.actor}</strong></div><div className="inspector-meta"><span>Time</span><strong>{new Date(selected.event.createdAt).toLocaleTimeString([],{hour:"2-digit",minute:"2-digit"})}</strong></div></>}
      <button className="inspector-action" onClick={action}>{selected.kind==="core"?"Open approvals":selected.kind==="signal"?"Open trace":"Open details"}<ArrowUpRight size={14}/></button>
    </aside>
    <div className="rubric-graph-controls"><button aria-label="Zoom out" onClick={()=>zoom(-.1)}><Minus size={14}/></button><button aria-label="Reset view" onClick={reset}><Focus size={15}/></button><button aria-label="Zoom in" onClick={()=>zoom(.1)}><Plus size={14}/></button><span>Drag to pan · Scroll to zoom · Select any node</span></div>
  </section>;
}
