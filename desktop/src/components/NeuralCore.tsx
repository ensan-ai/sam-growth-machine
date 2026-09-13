import { memo, useEffect, useRef, useState } from "react";
import { Focus, Minus, Plus } from "lucide-react";
import * as THREE from "three";
import type { Employee, Handoff, SystemEvent } from "../types";

// The coordinates describe reporting relationships, never workflow routing.
const positions: Record<string, [number, number]> = {
  sam: [0, 85], travis: [0, -55], saly: [-218, 38], adam: [-155, -145],
  maro: [216, -90], lara: [190, 157], brain: [-230, -280], jax: [-35, -275],
};
const hierarchy = [["sam","travis"],["travis","saly"],["travis","adam"],["travis","maro"],["travis","lara"],["adam","brain"],["adam","jax"]];
const tint = (status: string) => status === "BLOCKED" ? "#ffad84" : status === "WAITING_APPROVAL" ? "#ffda9c" : status === "WORKING" ? "#83ffe0" : "#a2e7f4";
const vertex = `varying vec2 vUv; varying vec3 vNormal; void main(){vUv=uv;vNormal=normal;gl_Position=projectionMatrix*modelViewMatrix*vec4(position,1.);}`;
const glowFragment = `uniform vec3 color; varying vec2 vUv; void main(){float d=length(vUv-.5)*2.;float a=pow(max(0.,1.-d),3.);gl_FragColor=vec4(color,a*.62);}`;
const coreFragment = `uniform float time; uniform vec3 color; varying vec2 vUv; varying vec3 vNormal;
void main(){float rim=pow(1.-abs(vNormal.z),2.4);float wave=sin(vUv.y*20.+sin(vUv.x*12.+time*.25)*2.+time*.45);float vein=pow(.5+.5*wave,7.);float mist=.5+.5*sin(vUv.x*9.-vUv.y*11.+time*.22);vec3 c=mix(vec3(.06,.18,.31),color,vein*.6+rim*.65);c=mix(c,vec3(.27,.25,.58),mist*.27);gl_FragColor=vec4(c, .3+rim*.5+vein*.21);}`;

type Props = { employees: Employee[]; handoffs: Handoff[]; events: SystemEvent[]; waitingApprovals: number; onSelect: (id: string) => void };
export const NeuralCore = memo(function NeuralCore({ employees, handoffs, events, waitingApprovals, onSelect }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const live = useRef({employees,handoffs,events,waitingApprovals});
  live.current = {employees,handoffs,events,waitingApprovals};
  const [zoom,setZoom] = useState(1);
  const [selected,setSelected] = useState("sam");
  const [renderError,setRenderError] = useState(false);
  const sceneControl = useRef<{zoom:(value:number)=>void}|undefined>(undefined);
  useEffect(()=>{sceneControl.current?.zoom(zoom);},[zoom]);
  useEffect(()=>{
    if(!host.current)return;
    const element=host.current;
    let renderer:THREE.WebGLRenderer;
    try {renderer=new THREE.WebGLRenderer({alpha:true,antialias:true,powerPreference:"low-power"});} catch {setRenderError(true);return;}
    renderer.setPixelRatio(Math.min(window.devicePixelRatio,1.5));
    renderer.setClearColor(0x000000,0); element.prepend(renderer.domElement);
    const scene=new THREE.Scene();
    const camera=new THREE.OrthographicCamera(-350,350,300,-300,.1,1000);camera.position.z=600;
    const meshes:Record<string,{glow:THREE.Mesh;orb:THREE.Mesh;material:THREE.ShaderMaterial;glowMaterial:THREE.ShaderMaterial}>={};
    const disposables:Array<{dispose:()=>void}>=[];
    const makeCurve=(a:string,b:string)=>{const p=positions[a],q=positions[b];return new THREE.CubicBezierCurve3(new THREE.Vector3(...p,0),new THREE.Vector3(p[0]+(q[0]-p[0])*.1,p[1]+(q[1]-p[1])*.65,-16),new THREE.Vector3(q[0]-(q[0]-p[0])*.5,q[1],-8),new THREE.Vector3(...q,0));};
    hierarchy.forEach(([a,b])=>{const geo=new THREE.BufferGeometry().setFromPoints(makeCurve(a,b).getPoints(64));const mat=new THREE.LineBasicMaterial({color:0x92d4e3,transparent:true,opacity:.24});scene.add(new THREE.Line(geo,mat));disposables.push(geo,mat);});
    Object.entries(positions).forEach(([id,p])=>{
      const radius=id==="sam"?55:id==="travis"?17:10;
      const group=new THREE.Group();group.position.set(...p,0);scene.add(group);
      const glowMaterial=new THREE.ShaderMaterial({vertexShader:vertex,fragmentShader:glowFragment,uniforms:{color:{value:new THREE.Color(tint("IDLE"))}},transparent:true,depthWrite:false,blending:THREE.AdditiveBlending});
      const glowGeo=new THREE.PlaneGeometry(radius*6,radius*6);const glow=new THREE.Mesh(glowGeo,glowMaterial);group.add(glow);
      const material=new THREE.ShaderMaterial({vertexShader:vertex,fragmentShader:coreFragment,uniforms:{time:{value:0},color:{value:new THREE.Color(id==="sam"?"#6beacf":"#b4efff")}},transparent:true,depthWrite:false});
      const geo=new THREE.SphereGeometry(radius,32,24);const orb=new THREE.Mesh(geo,material);orb.position.z=2;group.add(orb);
      if(id!=="sam"){const seedGeo=new THREE.SphereGeometry(radius*.28,12,8);const seedMat=new THREE.MeshBasicMaterial({color:0xe4ffff});const seed=new THREE.Mesh(seedGeo,seedMat);seed.position.z=radius+3;group.add(seed);disposables.push(seedGeo,seedMat);}
      meshes[id]={glow,orb,material,glowMaterial};disposables.push(glowGeo,glowMaterial,geo,material);
    });
    const pulseGeo=new THREE.SphereGeometry(2.1,8,6);const pulseMat=new THREE.MeshBasicMaterial({color:0xb6ffe9});disposables.push(pulseGeo,pulseMat);
    const pulses=Array.from({length:6},()=>{const mesh=new THREE.Mesh(pulseGeo,pulseMat);mesh.visible=false;scene.add(mesh);return mesh;});
    const resize=()=>{const w=element.clientWidth,h=element.clientHeight;renderer.setSize(w,h);camera.left=-350;camera.right=350;camera.top=300;camera.bottom=-330;camera.updateProjectionMatrix();Object.values(meshes).forEach(node=>{if(node.orb.parent)node.orb.parent.scale.x=(h/630)/(w/700);});};
    sceneControl.current={zoom:value=>{camera.zoom=value;camera.updateProjectionMatrix();}};
    const observer=new ResizeObserver(resize);observer.observe(element);resize();
    let frame=0,last=0;const reduced=window.matchMedia("(prefers-reduced-motion: reduce)");
    const render=(ms:number)=>{
      frame=requestAnimationFrame(render);if(document.hidden||ms-last<(reduced.matches?200:1000/30))return;last=ms;
      const now=Date.now();const data=live.current;
      Object.entries(meshes).forEach(([id,node])=>{const state=id==="sam"?(data.waitingApprovals>0?"WAITING_APPROVAL":"IDLE"):data.employees.find(e=>e.employeeId===id)?.runtimeStatus||"IDLE";
        const recent=data.events.some(e=>e.actor===id&&e.eventType==="agent.completed"&&now-new Date(e.createdAt).getTime()>=0&&now-new Date(e.createdAt).getTime()<8000);
        const active=state==="WORKING"||state==="WAITING_APPROVAL";const color=new THREE.Color(tint(state));
        node.glowMaterial.uniforms.color.value.copy(color);node.material.uniforms.color.value.copy(id==="sam"&&state==="IDLE"?new THREE.Color("#63dcca"):color);
        node.material.uniforms.time.value=reduced.matches?0:ms/1000;node.glow.scale.setScalar((active?1.25:recent?1.4:1)+(reduced.matches?0:Math.sin(ms/1800)*.035));
      });
      const recent=data.handoffs.filter(h=>positions[h.sender.toLowerCase()]&&positions[h.receiver.toLowerCase()]&&now-new Date(h.createdAt).getTime()>=0&&now-new Date(h.createdAt).getTime()<6000).slice(-6);
      pulses.forEach((mesh,i)=>{const h=recent[i];mesh.visible=!!h&&!reduced.matches;if(h){const progress=(now-new Date(h.createdAt).getTime())/6000;mesh.position.copy(makeCurve(h.sender.toLowerCase(),h.receiver.toLowerCase()).getPoint(progress));}});
      renderer.render(scene,camera);
    };frame=requestAnimationFrame(render);
    const visibility=()=>{cancelAnimationFrame(frame);if(!document.hidden)frame=requestAnimationFrame(render);};document.addEventListener("visibilitychange",visibility);
    return()=>{cancelAnimationFrame(frame);document.removeEventListener("visibilitychange",visibility);observer.disconnect();disposables.forEach(d=>d.dispose());renderer.dispose();renderer.domElement.remove();sceneControl.current=undefined;};
  },[]);
  const nodes=[{employeeId:"sam",name:"Sam",title:"Founder / CEO / Human Authority",runtimeStatus:waitingApprovals?"WAITING_APPROVAL":"AUTHORITY"},...employees];
  const focus=nodes.find(n=>n.employeeId===selected);
  return <div className="neural-stage">
    <div className="graph-heading"><span className="eyebrow">NEURAL CORE</span><span>Human direction. Collective intelligence.</span></div>
    <div className="neural-canvas" ref={host} aria-label="Sam manages Travis. Travis manages Saly, Adam, Maro and Lara. Adam manages Brain and Jax.">
      {renderError&&<p className="graph-error">3D rendering is unavailable. Employee controls remain available.</p>}
      <div className="node-labels" style={{transform:`scale(${zoom})`}}>{nodes.map(node=>{const p=positions[node.employeeId];if(!p)return null;return <button key={node.employeeId} onClick={()=>setSelected(node.employeeId)} className={`node-label node-${node.employeeId} ${selected===node.employeeId?"selected":""} ${node.runtimeStatus.toLowerCase()}`} style={{left:`${(p[0]+350)/700*100}%`,top:`${(300-p[1])/630*100}%`}} aria-label={`${node.name}, ${node.title}, ${node.runtimeStatus}`}><span className="node-hit"/><span className="node-name">{node.name}</span><small>{node.employeeId==="sam"?"HUMAN AUTHORITY":node.employeeId==="travis"?"GROWTH DIRECTOR":node.runtimeStatus==="IDLE"?"":node.runtimeStatus.replaceAll("_"," ")}</small></button>;})}</div>
    </div>
    <div className="graph-focus"><span className={`tiny-light ${focus?.runtimeStatus.toLowerCase()}`}/><strong>{focus?.name}</strong><span>{focus?.title}</span>{selected!=="sam"&&<button onClick={()=>onSelect(selected)}>View employee →</button>}</div>
    <div className="graph-controls"><button aria-label="Zoom out" disabled={zoom<=.8} onClick={()=>setZoom(v=>Math.max(.8,v-.1))}><Minus size={14}/></button><button aria-label="Reset graph view" onClick={()=>setZoom(1)}><Focus size={15}/></button><button aria-label="Zoom in" disabled={zoom>=1.2} onClick={()=>setZoom(v=>Math.min(1.2,v+.1))}><Plus size={14}/></button><span>Select a node to explore</span></div>
  </div>;
});
