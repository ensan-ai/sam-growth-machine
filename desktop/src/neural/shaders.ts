export const organicVertex = /* glsl */ `
uniform float time;
uniform float energy;
uniform float radius;
varying vec3 vNormal;
varying vec3 vView;
varying vec3 vObj;

void main() {
  vObj = position;
  float wave = sin(position.x * 4.4 + time * 0.38) + sin(position.y * 5.1 - time * 0.27) + sin(position.z * 4.8 + time * 0.33);
  vec3 displaced = position + normal * wave * (0.028 + energy * 0.035) * radius;
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
  float fresnel = pow(1.0 - ndv, 3.2);
  float inner = pow(ndv, 2.2);
  vec3 glass = vec3(0.05, 0.07, 0.11);
  vec3 pearl = mix(vec3(0.18, 0.24, 0.34), vec3(0.28, 0.22, 0.40), 0.45 + 0.12 * sin(vObj.y * 6.0 + time * 0.4));
  vec3 rgb = mix(glass, pearl, inner * (0.38 + authority * 0.18));
  vec3 ice = mix(vec3(0.32, 0.58, 0.78), vec3(0.86, 0.94, 1.0), energy);
  rgb += ice * pow(inner, 2.6) * (0.18 + energy * 0.72 + authority * 0.2);
  rgb += vec3(0.38, 0.28, 0.62) * pow(inner, 3.4) * 0.16;
  rgb += vec3(0.96, 0.48, 0.26) * blocked * pow(inner, 2.1) * (0.28 + 0.22 * sin(time * 3.6));
  rgb += vec3(0.90, 0.74, 0.38) * waiting * pow(inner, 1.9) * 0.32;
  rgb += vec3(0.62, 0.80, 0.94) * fresnel * (0.28 + energy * 0.12 + authority * 0.18);
  float alpha = 0.30 + inner * 0.22 + fresnel * 0.48 + energy * 0.06 + authority * 0.1;
  gl_FragColor = vec4(rgb, alpha);
}
`;

export const glowFragment = /* glsl */ `
uniform float energy;
varying vec2 vUv;
void main() {
  vec2 p = vUv * 2.0 - 1.0;
  float d = length(p);
  float a = pow(max(0.0, 1.0 - d), 3.4) * (0.16 + energy * 0.18);
  gl_FragColor = vec4(0.55, 0.72, 0.92, a);
}
`;

export const glowVertex = /* glsl */ `
varying vec2 vUv;
void main() {
  vUv = uv;
  gl_Position = projectionMatrix * modelViewMatrix * vec4(position, 1.0);
}
`;
