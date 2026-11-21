// Section Call to estimate (22n): before the search, the price block of a rare, magic or normal item, or of a skill
// gem, opened with "No market price for rare items." then, in gray, "Search similar offers to estimate it.": a refusal
// at the moment the player waits for a price, the action second (owner, 2026-11-21: "frustrating as soon as one
// starts using the application"). The search stays on a click (the guardrails of the trade site: a Ctrl+D isn't
// always meant as a search). The search block is already the white of the main action, so the leads only change the
// price block, or draw the eye to that block:
// A. The title "Estimated price" from the opening, faint dashes where the amounts will go, one line that leads with
//    the action and points to the block below.
// B. A, with a button in the price block that runs the same search: the strongest call, but two buttons for one
//    search, and two white blocks.
// C. A, with the search block beating twice at the opening of the item, before its first search only (the owner's
//    idea): a ring that spreads and fades, or the magnifier that swells. A click on the window plays it again.
// The owner chose A and C, the ring, on 2026-11-21: the line says what to do, the beat shows where. Rejected: B, two
// buttons for one search; the magnifier that swells, less seen than the ring.

const EC_TEXT = {
  en: { title: "Estimated price", call: "Search the offers below to estimate its price.", button: "Estimate the price",
    check: "Check a stat that sets this item apart to estimate it." },
  fr: { title: "Prix estimé", call: "Cherchez les offres ci-dessous pour estimer son prix.", button: "Estimer le prix",
    check: "Cochez une statistique qui distingue cet objet pour l'estimer." },
};

// Faint dashes in the three columns of the amounts, the orbs faint too.
const ecPending = () => `<div class="prices app-pending">${["div", "chaos", "ex"].map(k => `<div class="price">${orb(k)}—</div>`).join("")}</div>`;

// The price block of A (and of C), or of B with its button.
const ecPrice = (lang, { line = "call", button = false } = {}) => {
  const t = EC_TEXT[lang];
  const action = button ? `<div class="app-estimate-button">${ICON.search(16)}<span>${t.button}</span></div>` : "";
  // B: the button says the action, so no line points to the block below.
  const text = button ? "" : `<div class="line muted">${t[line]}</div>`;
  return `<div class="band a"><div class="head"><span class="label">${t.title}</span></div>${ecPending()}${text}${action}</div>`;
};

const EC_BOW_FR = { ...BOW_RARE, cls: "Arcs", tags: ["niv. 81", "Q 20%", "2 châsses"], req: "Niveau requis 65 · 145 Dex",
  statsLabel: "Statistiques (6)", filters: "4 filtres" };
const EC_MAGIC_FR = { ...MAGIC, name: "Anneau de saphir scintillant du lynx", cls: "Anneaux", tags: ["niv. 68"],
  req: "Niveau requis 44", statsLabel: "Statistiques (3)", filters: "2 filtres" };
const EC_FOOT_FR = "poe2scout il y a 12min · GGG trade il y a 2min";

const ecWin = (sample, lang, block, pulse) => {
  const fr = lang === "fr";
  const html = win(topBar(), item(sample), block, offers({ state: "idle" }, lang), foot(fr ? EC_FOOT_FR : undefined));
  return pulse ? html.replace('class="win"', `class="win app-pulse-win is-pulse-${pulse}"`) : html;
};

SECTIONS.unshift(["Call to estimate (22n): the price block of a rare, magic or normal item before its search", [
  ["Today · a refusal first, the action in gray", win(topBar(), item(EC_BOW_FR), price({ lines: ["Pas de prix de marché pour les objets rares.",
    "Cherchez des offres similaires pour l'estimer."] }, "fr"), offers({ state: "idle" }, "fr"), foot(EC_FOOT_FR))],
  ["Chosen · A · the title of the estimate, dashes, the action first", ecWin(EC_BOW_FR, "fr", ecPrice("fr"))],
  ["Chosen · A · in English", ecWin(BOW_RARE, "en", ecPrice("en"))],
  ["Chosen · A · a magic item (the same for a normal item and a skill gem)", ecWin(EC_MAGIC_FR, "fr", ecPrice("fr"))],
  ["Chosen · A · no stat checked: the line of 22a stays", ecWin(EC_BOW_FR, "fr", ecPrice("fr", { line: "check" }))],
  ["Rejected · B · A and a button in the price block, the same search", ecWin(EC_BOW_FR, "fr", ecPrice("fr", { button: true }))],
  ["Chosen · C · A and two beats of the search block: a ring (click the window to play it again)", ecWin(EC_BOW_FR, "fr", ecPrice("fr"), "ring")],
  ["Rejected · C · A and two beats of the search block: the magnifier swells (click to play again)", ecWin(EC_BOW_FR, "fr", ecPrice("fr"), "icon")],
]]);

// A click on a window of C plays its beats again.
document.addEventListener("click", event => {
  const pulsing = event.target.closest(".app-pulse-win");
  if (!pulsing) return;
  const kind = [...pulsing.classList].find(c => c.startsWith("is-pulse-"));
  pulsing.classList.remove(kind);
  void pulsing.offsetWidth;
  pulsing.classList.add(kind);
});
