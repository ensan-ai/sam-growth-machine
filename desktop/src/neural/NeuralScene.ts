import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { layoutWorkforce, nodeRadius } from "../graph/layout";
import type { CompanyGraph, GraphEdge, Vec3 } from "../graph/types";
import { glowFragment, glowVertex, membraneFragment, organicVertex } from "./shaders";

export type LabelState = {
  id: string;
  name: string;
  specialty: string;
  x: number;
  y: number;
  depth: number;
  visible: boolean;
  status: string;
  hovered: boolean;
  selected: boolean;
  authority: boolean;
};

type NodeMesh = {
  group: THREE.Group;
  orb: THREE.Mesh;
  glow: THREE.Mesh;
  material: THREE.ShaderMaterial;
  glowMaterial: THREE.ShaderMaterial;
  rest: THREE.Vector3;
  radius: number;
  seed: number;
};

type Pathway = {
  id: string;
  source: string;
  target: string;
  curve: THREE.CubicBezierCurve3;
  mesh: THREE.Mesh;
  synapses: THREE.Mesh[];
  twigs: THREE.Line[];
  relationship: GraphEdge["relationship"];
  restColor: number;
};

const tmp = new THREE.Vector3();
const MAX_PULSES = 10;
const MAX_DUST = 90;

function hash(value: string): number {
  let h = 0;
  for (let i = 0; i < value.length; i++) h = (h * 31 + value.charCodeAt(i)) | 0;
  return Math.abs(h);
}

function axonCurve(a: THREE.Vector3, b: THREE.Vector3, seed: number): THREE.CubicBezierCurve3 {
  const dir = b.clone().sub(a);
  const up = new THREE.Vector3(0, 1, 0);
  let side = new THREE.Vector3().crossVectors(dir, up);
  if (side.lengthSq() < 1e-5) side.crossVectors(dir, new THREE.Vector3(1, 0, 0));
  side.normalize();
  const lift = 0.28 + (seed % 80) / 220;
  const c1 = a.clone().lerp(b, 0.28).addScaledVector(side, lift).addScaledVector(up, 0.22);
  const c2 = a.clone().lerp(b, 0.72).addScaledVector(side, -lift * 0.45).addScaledVector(up, 0.38);
  return new THREE.CubicBezierCurve3(a, c1, c2, b);
}

export class NeuralScene {
  readonly renderer: THREE.WebGLRenderer;
  readonly camera: THREE.PerspectiveCamera;
  readonly controls: OrbitControls;
  labels: LabelState[] = [];
  hoveredId: string | null = null;
  selectedId = "sam";
  onHover?: (id: string | null) => void;
  onSelect?: (id: string) => void;
  private scene = new THREE.Scene();
  private nodes = new Map<string, NodeMesh>();
  private pathways: Pathway[] = [];
  private pulses: THREE.Mesh[] = [];
  private graph: CompanyGraph = { nodes: [], edges: [], pulses: [] };
  private positions = new Map<string, Vec3>();
  private orbGeo: THREE.IcosahedronGeometry;
  private glowGeo: THREE.PlaneGeometry;
  private synapseGeo: THREE.SphereGeometry;
  private pulseGeo: THREE.SphereGeometry;
  private pulseMat: THREE.MeshBasicMaterial;
  private synapseMat: THREE.MeshBasicMaterial;
  private lineMat: THREE.LineBasicMaterial;
  private activeLineMat: THREE.LineBasicMaterial;
  private handoffLineMat: THREE.LineBasicMaterial;
  private raycaster = new THREE.Raycaster();
  private pointer = new THREE.Vector2();
  private frame = 0;
  private last = 0;
  private idleRotateUntil = 0;
  private reduced: MediaQueryList;
  private element: HTMLElement;
  private dust?: THREE.Points;
  private disposed = false;
  private nodeCount = 0;

  constructor(element: HTMLElement) {
    this.element = element;
    this.renderer = new THREE.WebGLRenderer({ alpha: true, antialias: true, powerPreference: "high-performance" });
    this.renderer.setClearColor(0x000000, 0);
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.75));
    this.renderer.outputColorSpace = THREE.SRGBColorSpace;
    element.appendChild(this.renderer.domElement);
    this.camera = new THREE.PerspectiveCamera(42, 1, 0.1, 80);
    this.camera.position.set(0.8, 2.4, 7.4);
    this.scene.fog = new THREE.FogExp2(0x05121d, 0.048);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.06;
    this.controls.minDistance = 2.6;
    this.controls.maxDistance = 18;
    this.controls.target.set(0, 0.1, 0);
    this.controls.autoRotate = true;
    this.controls.autoRotateSpeed = 0.35;
    this.controls.addEventListener("start", () => {
      this.controls.autoRotate = false;
      this.idleRotateUntil = performance.now() + 9000;
    });
    this.orbGeo = new THREE.IcosahedronGeometry(1, 3);
    this.glowGeo = new THREE.PlaneGeometry(1, 1);
    this.synapseGeo = new THREE.SphereGeometry(1, 10, 8);
    this.pulseGeo = new THREE.SphereGeometry(1, 12, 10);
    this.pulseMat = new THREE.MeshBasicMaterial({ color: 0xc8fff0, transparent: true, opacity: 0.9, blending: THREE.AdditiveBlending, depthWrite: false });
    this.synapseMat = new THREE.MeshBasicMaterial({ color: 0x9ee7f2, transparent: true, opacity: 0.45, blending: THREE.AdditiveBlending, depthWrite: false });
    this.lineMat = new THREE.LineBasicMaterial({ color: 0x8fd4e4, transparent: true, opacity: 0.42 });
    this.activeLineMat = new THREE.LineBasicMaterial({ color: 0xb7fff0, transparent: true, opacity: 0.78 });
    this.handoffLineMat = new THREE.LineBasicMaterial({ color: 0xe7ffb0, transparent: true, opacity: 0.9 });
    this.pulses = Array.from({ length: MAX_PULSES }, () => {
      const mesh = new THREE.Mesh(this.pulseGeo, this.pulseMat);
      mesh.visible = false;
      mesh.scale.setScalar(0.055);
      this.scene.add(mesh);
      return mesh;
    });
    this.addDust();
    this.reduced = window.matchMedia("(prefers-reduced-motion: reduce)");
    this.renderer.domElement.addEventListener("pointermove", this.onPointerMove);
    this.renderer.domElement.addEventListener("click", this.onClick);
    this.resize();
    this.frame = requestAnimationFrame(this.tick);
  }

  setGraph(graph: CompanyGraph) {
    this.graph = graph;
    const ids = graph.nodes.map((node) => node.id).join("|");
    const signature = `${ids}|${graph.edges.filter((e) => e.relationship === "reports_to").map((e) => e.id).join(",")}`;
    if (signature !== this.nodeIds()) {
      this.rebuild(graph);
    }
    this.nodeCount = graph.nodes.length;
  }

  focus(id: string) {
    this.selectedId = id;
    const mesh = this.nodes.get(id);
    if (!mesh) return;
    this.controls.autoRotate = false;
    this.idleRotateUntil = performance.now() + 12000;
    const dest = mesh.rest.clone().add(new THREE.Vector3(0.4, 1.2, 2.8));
    this.animateCamera(dest, mesh.rest.clone());
  }

  resetView() {
    this.selectedId = "sam";
    this.animateCamera(new THREE.Vector3(0.8, 2.4, 7.4), new THREE.Vector3(0, 0.1, 0));
    this.idleRotateUntil = performance.now() + 400;
  }

  dispose() {
    this.disposed = true;
    cancelAnimationFrame(this.frame);
    this.renderer.domElement.removeEventListener("pointermove", this.onPointerMove);
    this.renderer.domElement.removeEventListener("click", this.onClick);
    this.controls.dispose();
    this.clearScene();
    this.orbGeo.dispose();
    this.glowGeo.dispose();
    this.synapseGeo.dispose();
    this.pulseGeo.dispose();
    this.pulseMat.dispose();
    this.synapseMat.dispose();
    this.lineMat.dispose();
    this.activeLineMat.dispose();
    this.handoffLineMat.dispose();
    this.dust?.geometry.dispose();
    (this.dust?.material as THREE.Material | undefined)?.dispose();
    this.renderer.dispose();
    this.renderer.domElement.remove();
  }

  private nodeIds() {
    return [...this.nodes.keys()].sort().join("|") + `|${this.pathways.filter((p) => p.relationship === "reports_to").map((p) => p.id).join(",")}`;
  }

  private rebuild(graph: CompanyGraph) {
    this.clearScene(false);
    this.positions = layoutWorkforce(graph.nodes);
    const count = graph.nodes.length;
    graph.nodes.forEach((node) => {
      const pos = this.positions.get(node.id) || [0, 0, 0];
      const radius = nodeRadius(node, count);
      const group = new THREE.Group();
      group.position.set(pos[0], pos[1], pos[2]);
      group.userData.id = node.id;
      const material = new THREE.ShaderMaterial({
        vertexShader: organicVertex,
        fragmentShader: membraneFragment,
        uniforms: {
          time: { value: 0 },
          energy: { value: 0 },
          radius: { value: radius },
          color: { value: new THREE.Color(node.role === "human_authority" ? "#7bf0d6" : "#b7ecff") },
          blocked: { value: 0 },
          waiting: { value: 0 },
          authority: { value: node.role === "human_authority" ? 1 : 0 },
        },
        transparent: true,
        depthWrite: true,
      });
      const orb = new THREE.Mesh(this.orbGeo, material);
      orb.scale.setScalar(radius);
      orb.userData.id = node.id;
      const glowMaterial = new THREE.ShaderMaterial({
        vertexShader: glowVertex,
        fragmentShader: glowFragment,
        uniforms: { color: { value: new THREE.Color("#8ceee0") }, energy: { value: 0 } },
        transparent: true,
        depthWrite: false,
        blending: THREE.AdditiveBlending,
      });
      const glow = new THREE.Mesh(this.glowGeo, glowMaterial);
      glow.scale.setScalar(radius * (node.role === "human_authority" ? 6.4 : 4.6));
      group.add(glow, orb);
      const inner = new THREE.Mesh(
        this.synapseGeo,
        new THREE.MeshBasicMaterial({ color: node.role === "human_authority" ? 0xeafffb : 0xb8f4ff, transparent: true, opacity: 0.85 }),
      );
      inner.scale.setScalar(radius * (node.role === "human_authority" ? 0.32 : 0.38));
      group.add(inner);
      this.scene.add(group);
      this.nodes.set(node.id, {
        group,
        orb,
        glow,
        material,
        glowMaterial,
        rest: new THREE.Vector3(pos[0], pos[1], pos[2]),
        radius,
        seed: hash(node.id) / 1000,
      });
    });
    this.buildPathways(graph);
  }

  private buildPathways(graph: CompanyGraph) {
    const orgEdges = graph.edges.filter((edge) => edge.relationship === "reports_to");
    const allowTwigs = graph.nodes.length <= 36;
    orgEdges.forEach((edge) => {
      const a = this.nodes.get(edge.source);
      const b = this.nodes.get(edge.target);
      if (!a || !b) return;
      const curve = axonCurve(a.rest, b.rest, hash(edge.id));
      const tube = new THREE.TubeGeometry(curve, 48, edge.source === "sam" ? 0.028 : 0.014, 6, false);
      const tubeMat = new THREE.MeshBasicMaterial({ color: 0x7ecfe0, transparent: true, opacity: 0.32 });
      const mesh = new THREE.Mesh(tube, tubeMat);
      this.scene.add(mesh);
      const synapses = [0.32, 0.57, 0.82].map((t) => {
        const mesh = new THREE.Mesh(this.synapseGeo, this.synapseMat);
        mesh.position.copy(curve.getPoint(t));
        mesh.scale.setScalar(0.028);
        this.scene.add(mesh);
        return mesh;
      });
      const twigs: THREE.Line[] = [];
      if (allowTwigs) {
        [0.22, 0.74].forEach((t, i) => {
          const origin = curve.getPoint(t);
          const tangent = curve.getTangent(t);
          const side = tmp.crossVectors(tangent, new THREE.Vector3(0, 1, 0)).normalize();
          if (side.lengthSq() < 1e-4) side.set(1, 0, 0);
          const end = origin.clone().addScaledVector(side, (i % 2 ? -1 : 1) * 0.38).add(new THREE.Vector3(0, 0.18, 0));
          const twigGeo = new THREE.BufferGeometry().setFromPoints([origin, origin.clone().lerp(end, 0.45), end]);
          const twig = new THREE.Line(twigGeo, this.lineMat);
          this.scene.add(twig);
          twigs.push(twig);
        });
      }
      this.pathways.push({ id: edge.id, source: edge.source, target: edge.target, curve, mesh, synapses, twigs, relationship: "reports_to", restColor: 0x7ecfe0 });
    });
  }

  private addDust() {
    const positions = new Float32Array(MAX_DUST * 3);
    for (let i = 0; i < MAX_DUST; i++) {
      const r = 2.2 + Math.random() * 5.5;
      const t = Math.random() * Math.PI * 2;
      const y = (Math.random() - 0.45) * 3.2;
      positions[i * 3] = Math.cos(t) * r;
      positions[i * 3 + 1] = y;
      positions[i * 3 + 2] = Math.sin(t) * r * 0.9;
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    const mat = new THREE.PointsMaterial({ color: 0x8fd4e4, size: 0.018, transparent: true, opacity: 0.22, depthWrite: false });
    this.dust = new THREE.Points(geo, mat);
    this.scene.add(this.dust);
  }

  private clearScene(full = true) {
    this.nodes.forEach((node) => {
      node.material.dispose();
      node.glowMaterial.dispose();
      this.scene.remove(node.group);
    });
    this.nodes.clear();
    this.pathways.forEach((path) => {
      this.scene.remove(path.mesh);
      path.mesh.geometry.dispose();
      (path.mesh.material as THREE.Material).dispose();
      path.synapses.forEach((s) => this.scene.remove(s));
      path.twigs.forEach((t) => {
        this.scene.remove(t);
        t.geometry.dispose();
      });
    });
    this.pathways = [];
    if (full && this.dust) this.scene.remove(this.dust);
  }

  private onPointerMove = (event: PointerEvent) => {
    const rect = this.renderer.domElement.getBoundingClientRect();
    this.pointer.x = ((event.clientX - rect.left) / rect.width) * 2 - 1;
    this.pointer.y = -((event.clientY - rect.top) / rect.height) * 2 + 1;
    this.raycaster.setFromCamera(this.pointer, this.camera);
    const hits = this.raycaster.intersectObjects([...this.nodes.values()].map((n) => n.orb), false);
    const id = hits[0]?.object.userData.id as string | undefined;
    if (id !== this.hoveredId) {
      this.hoveredId = id || null;
      this.onHover?.(this.hoveredId);
    }
    this.renderer.domElement.style.cursor = id ? "pointer" : "grab";
  };

  private onClick = () => {
    if (!this.hoveredId) return;
    this.selectedId = this.hoveredId;
    this.focus(this.hoveredId);
    this.onSelect?.(this.hoveredId);
  };

  private animateCamera(position: THREE.Vector3, target: THREE.Vector3) {
    const startP = this.camera.position.clone();
    const startT = this.controls.target.clone();
    const t0 = performance.now();
    const step = () => {
      if (this.disposed) return;
      const t = Math.min(1, (performance.now() - t0) / 700);
      const e = 1 - Math.pow(1 - t, 3);
      this.camera.position.lerpVectors(startP, position, e);
      this.controls.target.lerpVectors(startT, target, e);
      if (t < 1) requestAnimationFrame(step);
    };
    requestAnimationFrame(step);
  }

  resize = () => {
    const w = Math.max(1, this.element.clientWidth);
    const h = Math.max(1, this.element.clientHeight);
    this.camera.aspect = w / h;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(w, h);
  };

  private tick = (ms: number) => {
    if (this.disposed) return;
    this.frame = requestAnimationFrame(this.tick);
    if (document.hidden) return;
    const dt = Math.min(0.1, this.last ? (ms - this.last) / 1000 : 0.016);
    this.last = ms;
    if (this.reduced.matches) {
      this.controls.autoRotate = false;
    } else if (!this.controls.autoRotate && ms > this.idleRotateUntil) {
      this.controls.autoRotate = true;
    }
    this.controls.update();
    const time = this.reduced.matches ? 0 : ms / 1000;
    const byId = new Map(this.graph.nodes.map((node) => [node.id, node]));
    this.nodes.forEach((mesh, id) => {
      const node = byId.get(id);
      const status = node?.status || "IDLE";
      const energy = status === "WORKING" ? 1 : status === "WAITING_FOR_SAM" ? 0.55 : status === "BLOCKED" ? 0.12 : 0.22;
      mesh.material.uniforms.time.value = time + mesh.seed;
      mesh.material.uniforms.energy.value = energy;
      mesh.material.uniforms.blocked.value = status === "BLOCKED" ? 1 : 0;
      mesh.material.uniforms.waiting.value = status === "WAITING_FOR_SAM" ? 1 : 0;
      mesh.glowMaterial.uniforms.energy.value = energy;
      const tint = status === "BLOCKED" ? "#ffad84" : status === "WAITING_FOR_SAM" ? "#ffd89a" : status === "WORKING" ? "#8fffe2" : id === "sam" ? "#7bf0d6" : "#b7ecff";
      mesh.material.uniforms.color.value.set(tint);
      mesh.glowMaterial.uniforms.color.value.set(tint);
      const breath = this.reduced.matches ? 0 : Math.sin(time * (status === "WORKING" ? 2.4 : 1.1) + mesh.seed) * (status === "WORKING" ? 0.045 : 0.018);
      const still = status === "BLOCKED" ? 0.35 : 1;
      mesh.group.position.y = mesh.rest.y + breath * still;
      mesh.glow.quaternion.copy(this.camera.quaternion);
      const pulseScale = 1 + energy * 0.12 + (this.selectedId === id ? 0.08 : 0);
      mesh.orb.scale.setScalar(mesh.radius * pulseScale);
    });
    this.pathways.forEach((path) => {
      const src = byId.get(path.source);
      const dst = byId.get(path.target);
      const live = src?.status === "WORKING" || dst?.status === "WORKING" || src?.status === "WAITING_FOR_SAM" || dst?.status === "WAITING_FOR_SAM";
      const mat = path.mesh.material as THREE.MeshBasicMaterial;
      mat.color.setHex(live ? 0xb7fff0 : path.restColor);
      mat.opacity = live ? 0.55 : 0.32;
      path.synapses.forEach((s) => {
        s.scale.setScalar(live ? 0.04 : 0.024);
      });
    });
    this.updatePulses(ms);
    this.updateLabels();
    if (this.dust && !this.reduced.matches) this.dust.rotation.y += dt * 0.02;
    this.renderer.render(this.scene, this.camera);
  };

  private updatePulses(ms: number) {
    const now = Date.now();
    this.pulses.forEach((mesh) => { mesh.visible = false; });
    if (this.reduced.matches) return;
    this.graph.pulses.forEach((pulse, i) => {
      const mesh = this.pulses[i];
      if (!mesh) return;
      const path = this.pathways.find((p) => (p.source === pulse.source && p.target === pulse.target) || (p.source === pulse.target && p.target === pulse.source))
        || this.ensureHandoffPath(pulse.source, pulse.target);
      if (!path) return;
      const t = (now - pulse.startedAt) / pulse.durationMs;
      if (t < 0 || t > 1) return;
      mesh.visible = true;
      path.curve.getPoint(t, mesh.position);
      mesh.scale.setScalar(0.05 + Math.sin(t * Math.PI) * 0.04);
      const mat = path.mesh.material as THREE.MeshBasicMaterial;
      mat.color.setHex(0xe7ffb0);
      mat.opacity = 0.7;
      path.synapses.forEach((synapse, index) => {
        const at = 0.32 + index * 0.25;
        const near = Math.abs(t - at) < 0.12;
        synapse.scale.setScalar(near ? 0.07 : 0.03);
      });
    });
    void ms;
  }

  private ensureHandoffPath(source: string, target: string): Pathway | undefined {
    const existing = this.pathways.find((p) => p.source === source && p.target === target);
    if (existing) return existing;
    const a = this.nodes.get(source);
    const b = this.nodes.get(target);
    if (!a || !b) return;
    const curve = axonCurve(a.rest, b.rest, hash(source + target));
    const tube = new THREE.TubeGeometry(curve, 40, 0.016, 5, false);
    const tubeMat = new THREE.MeshBasicMaterial({ color: 0xe7ffb0, transparent: true, opacity: 0.55 });
    const mesh = new THREE.Mesh(tube, tubeMat);
    this.scene.add(mesh);
    const synapses = [0.35, 0.65].map((t) => {
      const mesh = new THREE.Mesh(this.synapseGeo, this.synapseMat);
      mesh.position.copy(curve.getPoint(t));
      mesh.scale.setScalar(0.03);
      this.scene.add(mesh);
      return mesh;
    });
    const path: Pathway = { id: `live:${source}:${target}`, source, target, curve, mesh, synapses, twigs: [], relationship: "handoff", restColor: 0xe7ffb0 };
    this.pathways.push(path);
    return path;
  }

  private updateLabels() {
    const w = this.element.clientWidth;
    const h = this.element.clientHeight;
    this.labels = this.graph.nodes.map((node) => {
      const mesh = this.nodes.get(node.id);
      const label: LabelState = {
        id: node.id,
        name: node.name,
        specialty: node.shortSpecialty,
        x: 0,
        y: 0,
        depth: 0,
        visible: false,
        status: node.status,
        hovered: this.hoveredId === node.id,
        selected: this.selectedId === node.id,
        authority: node.role === "human_authority",
      };
      if (!mesh) return label;
      tmp.copy(mesh.group.position);
      tmp.y -= mesh.radius + 0.12;
      tmp.project(this.camera);
      label.x = (tmp.x * 0.5 + 0.5) * w;
      label.y = (-tmp.y * 0.5 + 0.5) * h;
      label.depth = tmp.z;
      label.visible = tmp.z < 1 && tmp.z > -1 && label.x > -80 && label.x < w + 80 && label.y > -40 && label.y < h + 40;
      return label;
    });
  }
}
