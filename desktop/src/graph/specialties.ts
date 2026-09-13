/** Founding copy lives in the data layer, not the renderer. Future employees use title derivation. */
const FOUNDING_SHORT: Record<string, string> = {
  sam: "Human Authority",
  travis: "Workforce Director",
  saly: "Research",
  adam: "Strategy",
  brain: "Public Writing",
  jax: "Creative",
  maro: "Publishing",
  lara: "Measurement",
};

const ROLE_WORDS = /\b(specialist|director|manager|lead|agent|officer|engineer|writer|analyst|assistant|coordinator)\b/gi;

export function shortSpecialty(id: string, title: string, explicit?: string): string {
  const given = explicit?.trim();
  if (given) return given;
  const founding = FOUNDING_SHORT[id.toLowerCase()];
  if (founding) return founding;
  const stripped = title.replace(ROLE_WORDS, " ").replace(/[/|,].*$/, " ").replace(/\s+/g, " ").trim();
  if (stripped) return stripped;
  return title.split(/\s+/).slice(0, 2).join(" ") || "Specialist";
}
