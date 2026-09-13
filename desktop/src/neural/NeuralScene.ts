import * as THREE from "three";
import { OrbitControls } from "three/addons/controls/OrbitControls.js";
import { defaultCamera, labelAnchor, layoutWorkforce, nodeRadius, type LabelAnchor } from "../graph/layout";
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
  anchor: LabelAnchor;
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
  curve: THREE.CatmullRomCurve3;
  mesh: THREE.Mesh;
  synapses: THREE.Mesh[];
  twigs: THREE.Line[];
  relationship: GraphEdge["relationship"];
  restColor: number;
};

const tmp = new THREE.Vector3();
const tmpN = new THREE.Vector3();
const MAX_PULSES = 10;
const MAX_DUST = 70;

function hash(value: string): number {
  let h = 0;
  for (let i = 0; i < value.length; i++) h = (h * 31 + value.charCodeAt(i)) | 0;
  return Math.abs(h);
}

function neuralCurve(from: THREE.Vector3, to: THREE.Vector3, rFrom: number, rTo: number, seed: number): THREE.CatmullRomCurve3 {
  const dir = to.clone().sub(from);
  const dist = Math.max(0.04, dir.length());
  dir.multiplyScalar(1 / dist);
  const a = from.clone().addScaledVector(dir, rFrom * 1.12);
  const b = to.clone().addScaledVector(dir, -rTo * 1.12);
  const up = new THREE.Vector3(0, 0, 1);
  let side = new THREE.Vector3().crossVectors(dir, up);
  if (side.lengthSq() < 1e-6) side.crossVectors(dir, new THREE.Vector3(0, 1, 0));
  side.normalize();
  const bulge = Math.min(0.13, dist * 0.16) * (seed % 2 ? 1 : -1);
  const synapse = a.clone().lerp(b, 0.5).addScaledVector(side, bulge).add(new THREE.Vector3(0, dist * 0.035, 0));
  const dendrite = a.clone().lerp(synapse, 0.42).addScaledVector(side, bulge * 0.2);
  const axon = synapse.clone().lerp(b, 0.45).addScaledVector(side, -bulge * 0.12);
  return new THREE.CatmullRomCurve3([a, dendrite, synapse, axon, b]);
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
  private raycaster = new THREE.Raycaster();
  private pointer = new THREE.Vector2();
  private frame = 0;
  private last = 0;
  private idleRotateUntil = Number.POSITIVE_INFINITY;
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
    this.camera = new THREE.PerspectiveCamera(40, 1, 0.08, 40);
    this.camera.position.set(0, 0.22, 2.9);
    this.scene.fog = new THREE.FogExp2(0x05121d, 0.012);
    this.controls = new OrbitControls(this.camera, this.renderer.domElement);
    this.controls.enableDamping = true;
    this.controls.dampingFactor = 0.08;
    this.controls.minDistance = 1.9;
    this.controls.maxDistance = 6.2;
    this.controls.target.set(0, 0.08, 0);
    this.controls.minPolarAngle = Math.PI * 0.34;
    this.controls.maxPolarAngle = Math.PI * 0.64;
    this.controls.autoRotate = false;
    this.controls.autoRotateSpeed = 0.12;
    this.controls.addEventListener("start", () => {
      this.controls.autoRotate = false;
      this.idleRotateUntil = performance.now() + 14000;
    });
    this.orbGeo = new THREE.IcosahedronGeometry(1, 2);
    this.glowGeo = new THREE.PlaneGeometry(1, 1);
    this.synapseGeo = new THREE.SphereGeometry(1, 8, 6);
    this.pulseGeo = new THREE.SphereGeometry(1, 10, 8);
    this.pulseMat = new THREE.MeshBasicMaterial({ color: 0xc9e6ff, transparent: true, opacity: 0.82, blending: THREE.AdditiveBlending, depthWrite: false });
    this.synapseMat = new THREE.MeshBasicMaterial({ color: 0x8aaec8, transparent: true, opacity: 0.5, blending: THREE.AdditiveBlending, depthWrite: false });
    this.lineMat = new THREE.LineBasicMaterial({ color: 0x6a879e, transparent: true, opacity: 0.22 });
    this.pulses = Array.from({ length: MAX_PULSES }, () => {
      const mesh = new THREE.Mesh(this.pulseGeo, this.pulseMat);
      mesh.visible = false;
      mesh.scale.setScalar(0.03);
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
    if (signature !== this.nodeIds()) this.rebuild(graph);
    this.nodeCount = graph.nodes.length;
  }

  focus(id: string) {
    this.selectedId = id;
    this.controls.autoRotate = false;
    this.idleRotateUntil = performance.now() + 16000;
  }

  resetView() {
    this.selectedId = "sam";
    this.frameCluster(true);
    this.idleRotateUntil = Number.POSITIVE_INFINITY;
    this.controls.autoRotate = false;
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
          blocked: { value: 0 },
          waiting: { value: 0 },
          authority: { value: node.role === "human_authority" ? 1 : 0 },
        },
        transparent: true,
        depthWrite: false,
      });
      const orb = new THREE.Mesh(this.orbGeo, material);
      orb.scale.setScalar(radius);
      orb.userData.id = node.id;
      const glowMaterial = new THREE.ShaderMaterial({
        vertexShader: glowVertex,
        fragmentShader: glowFragment,
        uniforms: { energy: { value: 0 } },
        transparent: true,
        depthWrite: false,
        blending: THREE.AdditiveBlending,
      });
      const glow = new THREE.Mesh(this.glowGeo, glowMaterial);
      glow.scale.setScalar(radius * (node.role === "human_authority" ? 1.55 : 1.45));
      group.add(glow, orb);
      const inner = new THREE.Mesh(
        this.synapseGeo,
        new THREE.MeshBasicMaterial({
          color: node.role === "human_authority" ? 0xd4e8ff : 0x8fb4d4,
          transparent: true,
          opacity: 0.55,
        }),
      );
      inner.scale.setScalar(radius * 0.26);
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
    this.frameCluster(false);
  }

  private frameCluster(animate: boolean) {
    const cam = defaultCamera(this.positions, Math.max(0.8, this.element.clientWidth / Math.max(1, this.element.clientHeight)));
    const position = new THREE.Vector3(...cam.position);
    const target = new THREE.Vector3(...cam.target);
    if (animate) this.animateCamera(position, target);
    else {
      this.camera.position.copy(position);
      this.controls.target.copy(target);
      this.controls.update();
    }
  }

  private buildPathways(graph: CompanyGraph) {
    const orgEdges = graph.edges.filter((edge) => edge.relationship === "reports_to");
    const allowTwigs = graph.nodes.length <= 28;
    orgEdges.forEach((edge) => {
      const a = this.nodes.get(edge.target);
      const b = this.nodes.get(edge.source);
      if (!a || !b) return;
      const curve = neuralCurve(a.rest, b.rest, a.radius, b.radius, hash(edge.id));
      const tube = new THREE.TubeGeometry(curve, 36, edge.source === "sam" ? 0.006 : 0.0042, 4, false);
      const tubeMat = new THREE.MeshBasicMaterial({ color: 0x5a7388, transparent: true, opacity: 0.42 });
      const mesh = new THREE.Mesh(tube, tubeMat);
      this.scene.add(mesh);
      const synapse = new THREE.Mesh(this.synapseGeo, this.synapseMat);
      synapse.position.copy(curve.getPoint(0.5));
      synapse.scale.setScalar(0.016);
      this.scene.add(synapse);
      const twigs: THREE.Line[] = [];
      if (allowTwigs) {
        const origin = curve.getPoint(0.5);
        const tangent = curve.getTangent(0.5);
        const side = tmpN.crossVectors(tangent, new THREE.Vector3(0, 0, 1)).normalize();
        if (side.lengthSq() < 1e-4) side.set(1, 0, 0);
        const end = origin.clone().addScaledVector(side, 0.07).add(new THREE.Vector3(0, 0.03, 0));
        const twig = new THREE.Line(new THREE.BufferGeometry().setFromPoints([origin, end]), this.lineMat);
        this.scene.add(twig);
        twigs.push(twig);
      }
      this.pathways.push({
        id: edge.id,
        source: edge.source,
        target: edge.target,
        curve,
        mesh,
        synapses: [synapse],
        twigs,
        relationship: "reports_to",
        restColor: 0x5a7388,
      });
    });
  }

  private addDust() {
    const positions = new Float32Array(MAX_DUST * 3);
    for (let i = 0; i < MAX_DUST; i++) {
      const r = 0.5 + Math.random() * 1.4;
      const t = Math.random() * Math.PI * 2;
      positions[i * 3] = Math.cos(t) * r;
      positions[i * 3 + 1] = (Math.random() - 0.45) * 1.5;
      positions[i * 3 + 2] = Math.sin(t) * r * 0.55;
    }
    const geo = new THREE.BufferGeometry();
    geo.setAttribute("position", new THREE.BufferAttribute(positions, 3));
    const mat = new THREE.PointsMaterial({ color: 0x6a8298, size: 0.01, transparent: true, opacity: 0.14, depthWrite: false });
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
    if (this.reduced.matches) this.controls.autoRotate = false;
    else if (!this.controls.autoRotate && ms > this.idleRotateUntil) this.controls.autoRotate = true;
    this.controls.update();
    const time = this.reduced.matches ? 0 : ms / 1000;
    const byId = new Map(this.graph.nodes.map((node) => [node.id, node]));
    this.nodes.forEach((mesh, id) => {
      const node = byId.get(id);
      const status = node?.status || "IDLE";
      const energy = status === "WORKING" ? 1 : status === "WAITING_FOR_SAM" ? 0.4 : status === "BLOCKED" ? 0.08 : 0.08;
      mesh.material.uniforms.time.value = time + mesh.seed;
      mesh.material.uniforms.energy.value = energy;
      mesh.material.uniforms.blocked.value = status === "BLOCKED" ? 1 : 0;
      mesh.material.uniforms.waiting.value = status === "WAITING_FOR_SAM" ? 1 : 0;
      mesh.glowMaterial.uniforms.energy.value = energy;
      const breath = this.reduced.matches ? 0 : Math.sin(time * (status === "WORKING" ? 2.0 : 1.0) + mesh.seed) * 0.006;
      mesh.group.position.y = mesh.rest.y + breath * (status === "BLOCKED" ? 0.3 : 1);
      mesh.glow.quaternion.copy(this.camera.quaternion);
      mesh.orb.scale.setScalar(mesh.radius * (1 + energy * 0.06 + (this.selectedId === id ? 0.05 : 0)));
    });
    this.pathways.forEach((path) => {
      const src = byId.get(path.source);
      const dst = byId.get(path.target);
      const live = src?.status === "WORKING" || dst?.status === "WORKING" || src?.status === "WAITING_FOR_SAM" || dst?.status === "WAITING_FOR_SAM";
      const mat = path.mesh.material as THREE.MeshBasicMaterial;
      mat.color.setHex(live ? 0x88aec6 : path.restColor);
      mat.opacity = live ? 0.58 : 0.42;
      path.synapses.forEach((s) => s.scale.setScalar(live ? 0.022 : 0.016));
    });
    this.updatePulses(ms);
    this.updateLabels();
    if (this.dust && !this.reduced.matches) this.dust.rotation.y += dt * 0.015;
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
      mesh.scale.setScalar(0.028 + Math.sin(t * Math.PI) * 0.02);
      const mat = path.mesh.material as THREE.MeshBasicMaterial;
      mat.color.setHex(0xc5def2);
      mat.opacity = 0.7;
      path.synapses.forEach((synapse) => synapse.scale.setScalar(Math.abs(t - 0.5) < 0.12 ? 0.03 : 0.016));
    });
    void ms;
  }

  private ensureHandoffPath(source: string, target: string): Pathway | undefined {
    const existing = this.pathways.find((p) => p.source === source && p.target === target);
    if (existing) return existing;
    const a = this.nodes.get(source);
    const b = this.nodes.get(target);
    if (!a || !b) return;
    const curve = neuralCurve(a.rest, b.rest, a.radius, b.radius, hash(source + target));
    const tube = new THREE.TubeGeometry(curve, 28, 0.0048, 4, false);
    const tubeMat = new THREE.MeshBasicMaterial({ color: 0x8fb4cc, transparent: true, opacity: 0.5 });
    const mesh = new THREE.Mesh(tube, tubeMat);
    this.scene.add(mesh);
    const synapse = new THREE.Mesh(this.synapseGeo, this.synapseMat);
    synapse.position.copy(curve.getPoint(0.5));
    synapse.scale.setScalar(0.016);
    this.scene.add(synapse);
    const path: Pathway = { id: `live:${source}:${target}`, source, target, curve, mesh, synapses: [synapse], twigs: [], relationship: "handoff", restColor: 0x8fb4cc };
    this.pathways.push(path);
    return path;
  }

  private updateLabels() {
    const w = this.element.clientWidth;
    const h = this.element.clientHeight;
    const boxes = this.graph.nodes.map((node) => {
      const mesh = this.nodes.get(node.id);
      const pos = this.positions.get(node.id) || [0, 0, 0];
      const anchor = labelAnchor(node.id, pos);
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
        anchor,
      };
      if (!mesh) return { label, width: 0, height: 0, nx: 0, ny: 0, nr: 0 };
      tmp.copy(mesh.group.position);
      tmp.project(this.camera);
      const nx = (tmp.x * 0.5 + 0.5) * w;
      const ny = (-tmp.y * 0.5 + 0.5) * h;
      const nr = Math.max(10, mesh.radius * (h / (2 * Math.tan((this.camera.fov * Math.PI) / 360) * mesh.group.position.distanceTo(this.camera.position))));
      const width = node.name.length * 7.4 + (label.authority ? 18 : 10);
      const height = label.authority ? 22 : 16;
      let x = nx;
      let y = ny;
      if (anchor === "left") { x = nx - nr - width * 0.5 - 10; y = ny - 4; }
      else if (anchor === "right") { x = nx + nr + width * 0.5 + 10; y = ny - 4; }
      else if (anchor === "above") { x = nx; y = ny - nr - height - 6; }
      else { x = nx; y = ny + nr + 8; }
      label.x = x;
      label.y = y;
      label.depth = tmp.z;
      label.visible = tmp.z < 1 && tmp.z > -1 && x > 8 && x < w - 8 && y > 8 && y < h - 8;
      return { label, width, height, nx, ny, nr };
    });

    for (let iter = 0; iter < 18; iter++) {
      for (let i = 0; i < boxes.length; i++) {
        for (let j = i + 1; j < boxes.length; j++) {
          const a = boxes[i];
          const b = boxes[j];
          const dx = b.label.x - a.label.x;
          const dy = b.label.y - a.label.y;
          const ox = (a.width + b.width) * 0.5 + 8 - Math.abs(dx);
          const oy = (a.height + b.height) * 0.5 + 4 - Math.abs(dy);
          if (ox <= 0 || oy <= 0) continue;
          const pushX = (dx === 0 ? 1 : Math.sign(dx)) * ox * 0.5;
          const pushY = (dy === 0 ? 1 : Math.sign(dy)) * oy * 0.5;
          if (ox < oy) {
            a.label.x -= pushX * 0.5;
            b.label.x += pushX * 0.5;
          } else {
            a.label.y -= pushY * 0.5;
            b.label.y += pushY * 0.5;
          }
        }
        const a = boxes[i];
        boxes.forEach((n) => {
          const dx = a.label.x - n.nx;
          const dy = a.label.y - n.ny;
          const need = n.nr + Math.max(a.width, a.height) * 0.35 + 6;
          const dist = Math.hypot(dx, dy) || 0.001;
          if (dist >= need) return;
          const s = (need - dist) / dist;
          a.label.x += dx * s;
          a.label.y += dy * s;
        });
        a.label.x = Math.max(a.width * 0.5 + 8, Math.min(w - a.width * 0.5 - 8, a.label.x));
        a.label.y = Math.max(14, Math.min(h - 28, a.label.y));
      }
    }

    this.labels = boxes.map((b) => b.label);
  }
}
