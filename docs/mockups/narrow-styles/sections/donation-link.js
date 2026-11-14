// Section Donation link (18b): a discreet link to the Ko-fi page of the project (`ko-fi.com/sylentdev`), chosen
// on 2026-11-14 (step 18a: the README and the sponsor button of GitHub). A donation is voluntary and unlocks
// nothing: the link never stands in the way of a price, never comes back as a bubble, and opens the page in the
// browser. Three places drawn: A, in the status bar of the settings, after the author and the site, written like
// the link of sylent.art; B, a line of the settings above the sentence on Grinding Gear Games, telling that the
// application is free; C, a heart in the title bar, left of the bug, always in sight (the least discreet).
// B chosen by the owner on 2026-11-14, without the heart, the link white under the pointer and never underlined;
// sylent.art removed from the status bar of the settings at the same time.

const KOFI_URL = "https://ko-fi.com/sylentdev";
// The "heart" icon of Lucide (ISC license), on its 24 px grid.
const KOFI_HEART = size => `<svg class="ico" width="${size}" height="${size}" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true"><path d="M19 14c1.49-1.46 3-3.21 3-5.5A5.5 5.5 0 0 0 16.5 3c-1.76 0-3 .5-4.5 2-1.5-1.5-2.74-2-4.5-2A5.5 5.5 0 0 0 2 8.5c0 2.3 1.5 4.05 3 5.5l7 7Z"/></svg>`;
// `heart` the small heart before the text (A and B drawn with it, B chosen without it), `hover` the pointer over it.
const kofiLink = (text, { hover = false, heart = true } = {}) =>
  `<a class="app-kofi-link ${heart ? "" : "is-chosen"} ${hover ? "is-hovered" : ""}" href="${KOFI_URL}" target="_blank" rel="noopener" title="${KOFI_URL}">${heart ? KOFI_HEART(10) : ""}${text}</a>`;
// The status bar of the settings without the site of the author.
const kofiNoSite = html => html.replace(/ · <a href="https:\/\/sylent\.art"[^>]*>sylent\.art<\/a>/, "");

// A: the link last in the status bar of the settings; `update` with a newer version told before it.
const kofiA = ({ fr = false, update = false, hover = false } = {}) => {
  let html = (update ? upSettings({ fr }) : hlWin({ fr }));
  return html.replace(/(sylent\.art<\/a>)/, `$1 · ${kofiLink("Ko-fi", { hover })}`);
};

// B: a line above the sentence on Grinding Gear Games.
// `chosen` the drawing chosen: no heart, and no sylent.art in the status bar (`update` with a newer version told).
const kofiB = ({ fr = false, hover = false, chosen = false, update = false } = {}) => {
  const html = update ? upSettings({ fr }) : hlWin({ fr });
  return (chosen ? kofiNoSite(html) : html).replace(`<div class="disclaimer">`,
  `<div class="app-kofi-line">${fr ? "Exile Market est gratuit." : "Exile Market is free."} ${kofiLink(fr ? "Le soutenir sur Ko-fi" : "Support it on Ko-fi", { hover, heart: !chosen })}</div>
    <div class="disclaimer">`);
};

// C: a heart left of the bug, its tooltip under the pointer.
const kofiTopBar = ({ hover = false, fr = false } = {}) => {
  const label = fr ? "Soutenir sur Ko-fi" : "Support on Ko-fi";
  const tip = hover ? `<span class="app-tip">${label}</span>` : "";
  const heart = `<div class="app-icon-button app-kofi-heart ${hover ? "is-hovered" : ""}" title="${label}">${KOFI_HEART(15)}${tip}</div>`;
  return rpTopBar().replace(/<div class="app-icon-button app-report-bug/, `${heart}$&`);
};
const KOFI_ITEM = [item(HH), price(HH_PRICE), offers({ state: "idle" }), foot()];

SECTIONS.unshift(["Donation link (18b): B chosen, a line of the settings without the heart, and no sylent.art", [
  ["Chosen · B · a line of the settings: free, and the link", kofiB({ chosen: true })],
  ["Chosen · under the pointer: white, never underlined", kofiB({ chosen: true, hover: true })],
  ["Chosen · with a newer version told", kofiB({ chosen: true, update: true })],
  ["Chosen · en français, avec une nouvelle version", kofiB({ chosen: true, fr: true, update: true })],
  ["Rejected · A · last in the status bar of the settings, like sylent.art", kofiA()],
  ["Rejected · A · under the pointer", kofiA({ hover: true })],
  ["Rejected · A · with a newer version told", kofiA({ update: true })],
  ["Rejected · A · en français, avec une nouvelle version", kofiA({ fr: true, update: true })],
  ["Before · B · with the heart", kofiB()],
  ["Before · B · under the pointer", kofiB({ hover: true })],
  ["Before · B · en français", kofiB({ fr: true })],
  ["Rejected · C · a heart left of the bug, always in sight", win(kofiTopBar(), ...KOFI_ITEM)],
  ["Rejected · C · under the pointer: its tooltip", win(kofiTopBar({ hover: true }), ...KOFI_ITEM)],
]]);
