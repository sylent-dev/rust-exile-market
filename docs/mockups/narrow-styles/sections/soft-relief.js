// Section Soft relief (22i): the unchecked checkboxes and the min–max fields in soft relief, neumorphism, on the
// color of the title bar and of the status bar (owner, 2026-11-19, a test of style): every checkbox (the filters,
// the settings) while unchecked and every min–max field (the filters, the tier of a waystone); the checked box
// stays as it is, already very visible. Six leads, each one egui can draw: 1 today; 2 flat, the fill of the bars
// and the line of today; 3 raised, cast outside; 4 pressed in; 5 a bevel of 1 px; 6 pressed in with the line of
// today. Then four signs of a field being typed in, on the leads 2 to 6, the line of the accent dropped (owner):
// 7 nothing; 8 the lighter fill of the controls; 9 a gray line under it; 10 a faint gray line around it.
// Chosen by the owner on 2026-11-19: 4 and 8, depth 1 px, softness 2.5 px, dark 70 %, light 3 %, now the style
// of the whole page (css/soft-relief.css); the radios of the settings follow (Claude Code's recommendation, the
// question left unanswered).

const SR_LEADS = [
  [1, "Rejected · 1 today"], [2, "Rejected · 2 flat"], [3, "Rejected · 3 raised"], [4, "Chosen · 4 pressed in"],
  [5, "Rejected · 5 bevel"], [6, "Rejected · 6 pressed, line"],
];
const srField = (min, max = "", focused = false) =>
  `<span class="input app-range${focused ? " is-focused" : ""}"><input value="${min}" inputmode="decimal" aria-label="Minimum">` +
  `<i>–</i><input value="${max}" inputmode="decimal" aria-label="Maximum"></span>`;
const srBoard = () => `<div class="win"><div class="app-soft-board">
  <div class="app-soft-row app-soft-head"><b></b><span>Off</span><span>On</span><span>Empty</span><span>Filled</span><span>Typing</span></div>
  ${SR_LEADS.map(([lead, title]) => `<div class="app-soft app-soft-row" data-lead="${lead}"><b>${title}</b>` +
    `<div><span class="check"></span></div><div><span class="check on">${ICON.check(11)}</span></div>` +
    `<div>${srField("")}</div><div>${srField(80)}</div><div>${srField(11, 13, true)}</div></div>`).join("")}
</div></div>`;

SECTIONS.unshift(["Soft relief (22i): 4 chosen, the unchecked controls and the fields pressed into their block", [
  ["The leads and the states · typing: 8, the lighter fill", srBoard()],
  ["Chosen · Ancestral Mail, its filters", win(topBar(), item({ ...MAIL, stats: mailStats() }, true), price(RARE_ESTIMATE_BEFORE), foot())],
  ["Chosen · a waystone, its tier 11 – 13", wtrWin(wtrStatsB(11, 13))],
  ["Chosen · the settings, the radios too", ptSettingsD()],
]]);
