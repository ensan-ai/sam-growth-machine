export const organicVertex = /* glsl */ `
uniform float time;
uniform float energy;
uniform float radius;
varying vec3 vNormal;
varying vec3 vView;
varying vec3 vObj;

void main() {
  vObj = position;
  float wave = sin(position.x * 5.6 + time * 0.55) + sin(position.y * 6.4 - time * 0.41) + sin(position.z * 5.9 + time * 0.37);
  vec3 displaced = position + normal * wave * (0.022 + energy * 0.03) * radius;
  vec4 mv = modelViewMatrix * vec4(displaced, 1.0);
  vNormal = normalize(normalMatrix * normal);
  vView = -mv.xyz;
  gl_Position = projectionMatrix * mv;
}
`;

export const membraneFragment = /* glsl */ `
uniform float time;
uniform float energy;
uniform float blocked;
uniform float waiting;
uniform float authority;
varying vec3 vNormal;
varying vec3 vView;
varying vec3 vObj;

void main() {
  vec3 N = normalize(vNormal);
  vec3 V = normalize(vView);
  float ndv = max(dot(N, V), 0.0);
  float fresnel = pow(1.0 - ndv, 3.6);
  float inner = pow(ndv, 2.6);
  vec3 glass = vec3(0.04, 0.055, 0.08);
  vec3 pearl = mix(vec3(0.16, 0.22, 0.32), vec3(0.24, 0.18, 0.36), 0.4 + 0.15 * sin(vObj.y * 8.0 + time * 0.55));
  vec3 rgb = mix(glass, pearl, inner * (0.28 + authority * 0.22));
  vec3 ice = mix(vec3(0.28, 0.52, 0.72), vec3(0.82, 0.93, 1.0), energy);
  rgb += ice * pow(inner, 3.0) * (0.14 + energy * 0.7 + authority * 0.18);
  rgb += vec3(0.34, 0.24, 0.58) * pow(inner, 4.0) * 0.12;
  rgb += vec3(0.94, 0.46, 0.24) * blocked * pow(inner, 2.4) * (0.22 + 0.2 * sin(time * 3.8));
  rgb += vec3(0.88, 0.72, 0.36) * waiting * pow(inner, 2.2) * 0.26;
  rgb += vec3(0.7, 0.86, 0.98) * fresnel * (0.22 + energy * 0.1 + authority * 0.16);
  float alpha = 0.18 + inner * 0.16 + fresnel * 0.42 + energy * 0.05 + authority * 0.1;
  gl_FragColor = vec4(rgb, alpha);
}
`;

export const glowFragment = /* glsl */ `
uniform float energy;
varying vec2 vUv;
void main() {
  vec2 p = vUv * 2.0 - 1.0;
  float d = length(p);
  float a = pow(max(0.0, 1.0 - d), 3.8) * (0.1 + energy * 0.14);
  gl_FragColor = vec4(0.52, 0.7, 0.92, a);
}
`;

export const glowVertex = /* glsl */ `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;
