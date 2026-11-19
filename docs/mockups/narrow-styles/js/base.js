// The styles and the variants read from the address, the orbs, the icons, and the sections to fill.
const THEMES = {
  forge: { title: "A · Forge", text: "Close to the game: engraved capitals for the item names (Cinzel), a sober text face (Source Sans 3), gold accent, warm near-black grays." },
  slate: { title: "B · Slate", text: "Modern and sharp: one geometric face (Manrope), numbers in a monospace face (JetBrains Mono), white as the main color so that the colors stay for the item rarities, cool near-black grays." },
  compact: { title: "C · Compact", text: "Dense, made for the narrow width: condensed capitals for the names (Barlow Semi Condensed), Barlow for the text, amber accent, neutral near-black grays." },
};
const VARIANTS = {
  theme: { label: "Style", options: Object.fromEntries(Object.entries(THEMES).map(([k, v]) => [k, v.title])), fallback: "slate" },
  league: { label: "League", options: { softcore: "Softcore", hardcore: "Hardcore" }, fallback: "softcore" },
  // Soft relief (22i): the lead drawn on the windows of its section.
  relief: { label: "Relief (22i)", options: { 1: "1 today", 2: "2 flat", 3: "3 raised", 4: "4 pressed in", 5: "5 bevel", 6: "6 pressed in, its line" }, fallback: "4" },
  // Soft relief (22i): a discreet sign of the field being typed in, on the leads 2 to 6 (owner, 2026-11-19).
  typing: { label: "Typing (22i)", options: { 7: "7 nothing", 8: "8 lighter fill", 9: "9 underline", 10: "10 gray line" }, fallback: "7" },
};
// Values tuned by eye, each a slider of the header that sets a CSS variable of the page (`scale` divides the
// value shown, so that a percent sets an opacity); the address keeps them too.
const SLIDERS = {
  depth: { label: "Depth", min: 0, max: 4, step: 0.5, fallback: 1.5, unit: "px", variable: "--soft-depth" },
  softness: { label: "Softness", min: 0, max: 8, step: 0.5, fallback: 3, unit: "px", variable: "--soft-blur" },
  dark: { label: "Dark", min: 0, max: 100, step: 5, fallback: 60, unit: "%", variable: "--soft-dark-a", scale: 100 },
  light: { label: "Light", min: 0, max: 30, step: 1, fallback: 6, unit: "%", variable: "--soft-light-a", scale: 100 },
};
const params = new URLSearchParams(location.search);
const chosen = Object.fromEntries(Object.entries(VARIANTS).map(([key, v]) => [key, params.get(key) in v.options ? params.get(key) : v.fallback]));
for (const [key, v] of Object.entries(SLIDERS)) {
  const value = Number(params.get(key));
  chosen[key] = params.has(key) && value >= v.min && value <= v.max ? value : v.fallback;
}
let theme = chosen.theme;

const ORB = { div: "divine-orb", chaos: "chaos-orb", ex: "exalted-orb" };
// The three orbs of the prices by their short key; any other item of the currency exchange by its id on the
// trade site, from the icons of assets/exchange/ (fix of the prices in other currencies).
const orb = key => `<img src="../../assets/${ORB[key] ? `currencies/${ORB[key]}` : `exchange/${key}`}.png" alt="">`;

// Icons, drawn with the current text color.
const svg = (size, body) => `<svg class="ico" width="${size}" height="${size}" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">${body}</svg>`;
const ICON = {
  search: size => svg(size, '<circle cx="7" cy="7" r="4.6"/><path d="M10.4 10.4 14 14"/>'),
  refresh: size => svg(size, '<path d="M13.5 8a5.5 5.5 0 1 1-1.6-3.9"/><path d="M13.6 2.6v3.3h-3.3"/>'),
  // A circle struck through, "not possible here", at the size of a checkbox.
  forbid: size => `<svg class="ico" width="${size}" height="${size}" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.2" stroke-linecap="round" aria-hidden="true"><circle cx="8" cy="8" r="5.8"/><path d="M3.9 12.1l8.2-8.2"/></svg>`,
  chevron: size => svg(size, '<path d="M6 4l4 4-4 4"/>'),
  down: size => svg(size, '<path d="M4 6l4 4 4-4"/>'),
  downDouble: size => svg(size, '<path d="M4 3.5l4 4 4-4"/><path d="M4 8.5l4 4 4-4"/>'),
  check: size => `<svg class="ico" width="${size}" height="${size}" viewBox="0 0 16 16" fill="none" stroke="currentColor" stroke-width="2.6" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M3.2 8.4l3.1 3.1 6.5-7"/></svg>`,
  // The "settings" icon of Lucide (ISC license), on its 24 px grid.
  gear: size => `<svg class="ico" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/><circle cx="12" cy="12" r="3"/></svg>`,
};

// Filled by the files of sections/, in the order of their script tags.
const SECTIONS = [];
