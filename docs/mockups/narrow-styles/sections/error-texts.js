// Section Error texts (22c): three failures that today read as another one.
// 1. The price database answers, but its answer can't be read (JSON that isn't the expected one, no current league,
//    no rate of a currency, too many categories): the banner says "can't be reached" and the status bar "(offline)",
//    which tells the player to check their network. A, no banner: the status bar gives the age of the prices; B, its
//    own words, the words of the trade site for the same case ("The answer of … can't be read."), and "(unreadable)"
//    in the status bar; C, B without the status bar changed. Recommended: B, the player sees why the prices are old
//    and that nothing on their side would help; no "report it" in the band: when the database changes, every player
//    sees it at once, the owner first.
// 2. A refusal of the trade site that isn't a limit (HTTP 403, with or without a wait) reads "Trade site limit". The
//    wait stays (a minute when the site gives none). A, its own words in yellow, the wait in the same line, built as
//    the line of a limit ("Refused by the trade site: next search in 1min.", which keeps the French on one line); B,
//    A with what to do under the row, "report it" opening the bug report. Not "The trade site refused this search.":
//    the words of a search turned down for its filters (17e3c), whose hint tells to uncheck a filter, which wouldn't
//    help here. Recommended: B, the cause of a 403 isn't known yet, and the log of a report would tell it.
// 3. "Copy the log" copies the whole log read, its last 60,000 bytes: a longer log loses its first lines without a
//    word, and the one who reads it may look for its start. A first line says it, in English as the log. The preview
//    and the address keep only the last lines that fit in the address (about 6,000 characters), which "Its last
//    lines." tells already: they never reach the cut, so the line is in the copy only.

const ET_TEXT = {
  en: { unreadable: "The answer of the price database can't be read. Last updated 3h ago.", unavailable: "Prices can't be read, retrying…",
    refused: "Refused by the trade site: next search in 1min.", hint: `If it happens again, ${REPORT("report it")}.` },
  fr: { unreadable: "La réponse de la base des prix est illisible. Dernière mise à jour il y a 3h.", unavailable: "Prix illisibles, nouvelle tentative…",
    refused: "Refus du site d'échange : prochaine recherche dans 1min.", hint: `Si cela se répète, ${REPORT("signalez-le", true)}.` },
};

const etPricesWin = (bannerText, status) => win(topBar(), ...(bannerText ? [banner(bannerText)] : []), item(HH), price(HH_PRICE),
  offers({ state: "idle" }), foot(`${status} · GGG trade 2min ago`));
const etPricesFr = (bannerText, status) => win(topBar(), banner(bannerText), empty, foot(status));

// The title row of the offers while the wait of the refusal runs, `hint` under it.
const etRefusedRow = (line, hint = "", fr = false) => refusedRow(line, "warn", hint, fr)
  .replace(/<div class="search-block"[^]*?<\/svg><\/div>/, `<div class="again waiting" title="${fr ? "Prochaine recherche dans 1min" : "Next search in 1min"}">1min</div>`);
const etRefusedWin = (row, fr = false) => win(topBar(), item(HH), price(HH_PRICE), row,
  foot(fr ? "poe2scout il y a 12min · GGG trade : prochaine recherche dans 1min" : "poe2scout 12min ago · GGG trade: next search in 1min"));

// What "Copy the log" pastes, its first lines.
const etPasted = first => `<div class="band b settings"><div class="app-report-label"><span class="label">Pasted from "Copy the log"</span></div>
  <div class="app-report-box is-log">${first ? `<span class="is-hidden">${first}</span>\n` : ""}17:12:40 INFO Item copied: Item Class: Belts
17:12:40 INFO Price of Headhunter: 12.4 ex
17:13:02 INFO Trade site: 23 offers found
18:47:59 INFO Settings saved
…</div></div>`;

SECTIONS.unshift(["Error texts (22c): an unreadable price database, a refusal that isn't a limit, a cut log", [
  ["1 · Today · cached prices, the answer unreadable: “can't be reached”", etPricesWin("The price database can't be reached. Last updated 3h ago.", "poe2scout 3h ago (offline)")],
  ["1 · A · no banner, the age in the status bar", etPricesWin("", "poe2scout 3h ago")],
  ["1 · B · its own words, “(unreadable)” in the status bar", etPricesWin(ET_TEXT.en.unreadable, "poe2scout 3h ago (unreadable)")],
  ["1 · B · first launch, no prices yet", win(topBar(), banner(ET_TEXT.en.unavailable), empty, foot("poe2scout: no prices yet"))],
  ["1 · B · en français", etPricesFr(ET_TEXT.fr.unreadable, "poe2scout il y a 3h (illisible)")],
  ["1 · C · its own words, the status bar as today", etPricesWin(ET_TEXT.en.unreadable, "poe2scout 3h ago (offline)")],
  ["2 · Today · a 403 reads as a limit", etRefusedWin(etRefusedRow("Trade site limit: next search in 1min."))],
  ["2 · A · its own words, the wait in the line", etRefusedWin(etRefusedRow(ET_TEXT.en.refused))],
  ["2 · B · A, and what to do under the row", etRefusedWin(etRefusedRow(ET_TEXT.en.refused, ET_TEXT.en.hint))],
  ["2 · B · en français", etRefusedWin(etRefusedRow(ET_TEXT.fr.refused, ET_TEXT.fr.hint, true), true)],
  ["3 · Today · the copy of a cut log starts anywhere", etPasted("")],
  ["3 · A first line says it", etPasted("… (first lines left out)")],
]]);
