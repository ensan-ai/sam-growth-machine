import { memo, useEffect, useMemo, useRef, useState } from "react";
import { Focus, Minus, Plus } from "lucide-react";
import { buildCompanyGraph } from "../graph/buildGraph";
import { NeuralScene, type LabelState } from "../neural/NeuralScene";
import type { Employee, Execution, Handoff, SystemEvent, WorkItem } from "../types";

type Props = {
  employees: Employee[];
  handoffs: Handoff[];
  events: SystemEvent[];
  executions?: Execution[];
  workItems?: WorkItem[];
  waitingApprovals: number;
  onSelect: (id: string) => void;
};

export const NeuralCore = memo(function NeuralCore({
  employees, handoffs, events, executions, workItems, waitingApprovals, onSelect,
}: Props) {
  const host = useRef<HTMLDivElement>(null);
  const sceneRef = useRef<NeuralScene | null>(null);
  const labelRefs = useRef(new Map<string, HTMLButtonElement>());
  const [hovered, setHovered] = useState<string | null>(null);
  const [selected, setSelected] = useState("sam");
  const [renderError, setRenderError] = useState(false);
  const graph = useMemo(
    () => buildCompanyGraph({ employees, handoffs, events, executions, workItems, waitingApprovals }),
    [employees, handoffs, events, executions, workItems, waitingApprovals],
  );

  useEffect(() => {
    if (!host.current) return;
    let scene: NeuralScene;
    try {
      scene = new NeuralScene(host.current);
    } catch {
      setRenderError(true);
      return;
    }
    sceneRef.current = scene;
    scene.onHover = setHovered;
    scene.onSelect = (id) => {
      setSelected(id);
      onSelect(id);
    };
    const observer = new ResizeObserver(scene.resize);
    observer.observe(host.current);
    let frame = 0;
    const pump = () => {
      frame = requestAnimationFrame(pump);
      scene.labels.forEach((label) => applyLabel(labelRefs.current.get(label.id), label));
    };
    frame = requestAnimationFrame(pump);
    return () => {
      cancelAnimationFrame(frame);
      observer.disconnect();
      scene.dispose();
      sceneRef.current = null;
    };
  }, [onSelect]);

  useEffect(() => {
    sceneRef.current?.setGraph(graph);
  }, [graph]);

  const focus = graph.nodes.find((node) => node.id === selected) || graph.nodes[0];
  const hoverNode = graph.nodes.find((node) => node.id === hovered);

  return (
    <div className="neural-stage">
      <div className="neural-canvas" ref={host} aria-label="Living 3D neural core of the company. Orbit, zoom, and select employees.">
        {renderError && <p className="graph-error">3D rendering is unavailable. Employee controls remain available.</p>}
        <div className="node-labels">
          {graph.nodes.map((node) => (
            <button
              key={node.id}
              ref={(el) => { if (el) labelRefs.current.set(node.id, el); else labelRefs.current.delete(node.id); }}
              className={`node-label ${node.role === "human_authority" ? "node-sam" : ""} ${selected === node.id ? "selected" : ""} ${node.status.toLowerCase()}`}
              onClick={() => {
                setSelected(node.id);
                sceneRef.current?.focus(node.id);
                onSelect(node.id);
              }}
            >
              <span className="node-name">{node.name}</span>
              {(hovered === node.id || selected === node.id) && <small>{node.shortSpecialty}</small>}
            </button>
          ))}
        </div>
      </div>
      <div className="graph-focus glass-hud compact">
        <span className={`tiny-light ${focus?.status.toLowerCase()}`} />
        <strong>{focus?.name}</strong>
        <span>{hoverNode && hoverNode.id !== focus?.id ? hoverNode.shortSpecialty : focus?.shortSpecialty}</span>
      </div>
      <div className="graph-controls">
        <button aria-label="Zoom out" onClick={() => { const c = sceneRef.current; if (c) c.camera.position.multiplyScalar(1.12); }}><Minus size={14} /></button>
        <button aria-label="Reset company view" onClick={() => { setSelected("sam"); sceneRef.current?.resetView(); }}><Focus size={15} /></button>
        <button aria-label="Zoom in" onClick={() => { const c = sceneRef.current; if (c) c.camera.position.multiplyScalar(0.88); }}><Plus size={14} /></button>
        <span>Drag to orbit · scroll to zoom · click to focus</span>
      </div>
    </div>
  );
});

function applyLabel(el: HTMLButtonElement | undefined, label: LabelState) {
  if (!el) return;
  el.style.transform = `translate(-50%, 0) translate(${label.x}px, ${label.y}px)`;
  el.style.visibility = label.visible ? "visible" : "hidden";
  el.style.zIndex = String(Math.round((1 - label.depth) * 100));
}
