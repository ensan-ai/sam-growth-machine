export const organicVertex = /* glsl */ `
uniform float time;
uniform float energy;
uniform float radius;
varying vec3 vNormal;
varying vec3 vView;

void main() {
  float wave = sin(position.x * 3.2 + time * 0.42) + sin(position.y * 4.1 - time * 0.31) + sin(position.z * 3.6 + time * 0.37);
  vec3 displaced = position + normal * wave * (0.035 + energy * 0.05) * radius;
  vec4 mv = modelViewMatrix * vec4(displaced, 1.0);
  vNormal = normalize(normalMatrix * normal);
  vView = -mv.xyz;
  gl_Position = projectionMatrix * mv;
}
`;

export const membraneFragment = /* glsl */ `
uniform vec3 color;
uniform float energy;
uniform float blocked;
uniform float waiting;
uniform float authority;
varying vec3 vNormal;
varying vec3 vView;

void main() {
  vec3 N = normalize(vNormal);
  vec3 V = normalize(vView);
  float fresnel = pow(1.0 - max(dot(N, V), 0.0), 2.2);
  float core = pow(max(dot(N, V), 0.0), 1.35);
  vec3 membrane = mix(vec3(0.05, 0.16, 0.22), color, 0.72 + authority * 0.18);
  vec3 light = color * (0.45 + energy * 0.7 + authority * 0.25);
  vec3 rgb = membrane * (0.45 + core * 0.55) + light * fresnel;
  rgb += color * (0.12 + energy * 0.22);
  rgb = mix(rgb, vec3(0.95, 0.52, 0.36), blocked * 0.5);
  rgb = mix(rgb, vec3(1.0, 0.84, 0.52), waiting * 0.42);
  float alpha = 0.62 + fresnel * 0.32 + energy * 0.12 + authority * 0.1;
  gl_FragColor = vec4(rgb, alpha);
}
`;

export const glowFragment = /* glsl */ `
uniform vec3 color;
uniform float energy;
varying vec2 vUv;
void main() {
  vec2 p = vUv * 2.0 - 1.0;
  float d = length(p);
  float a = pow(max(0.0, 1.0 - d), 2.8) * (0.45 + energy * 0.4);
  gl_FragColor = vec4(color, a);
}
`;

export const glowVertex = /* glsl */ `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;
