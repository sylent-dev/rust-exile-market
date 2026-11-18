# Roadmap

Implementation steps of the features described in [specs.md](specs.md). Each step is delivered in its own pull request, which checks its box.

## Reference

Read only by the steps that need them:

- [Technical choices](docs/technical-choices.md): the libraries, the services and the main designs, with their reasons.
- [Code quality](docs/code-quality.md): the rules of the code (names, functions, types, ownership, modules, patterns, tests, refactoring), read at the start of each step that changes code.
- [Releases](docs/releases.md): how to publish a version, and what its archives hold.
- [Images](docs/images.md): the game images the application may show, and the rules of Grinding Gear Games.
- [poe2scout usage](docs/poe2scout-usage.md): the limits of the price database and the rules the application follows.
- [Trade site usage](docs/trade-site-usage.md): the risk accepted by the owner, the limits and the guardrails.
- [Game data](docs/game-data.md): the stat ids and the modifier tiers, their sources and how they fit together.
- [Backlog](backlog.md): what is noted and not being done; read when the next work is chosen (steps 20 and 22).
- [Lessons](docs/lessons.md): what working on the project taught, kept for the retrospectives (steps 19 and 23).
- [Roadmap archive](docs/roadmap-archive.md): the detail of the steps done.

## Models

Each step to come names the Claude model recommended for its session (owner, 2026-11-10), so that the owner switches with `/model` before starting it; a new step or sub-step gets its model when it is written here. A judgment of the difficulty, not a measure: Fable 5.2 where a quiet mistake costs most (the matching of the game data, the guardrails of the trade site, what installs code on a player's machine); Opus 5.5 for every other step (mock-up, interface, refactoring, release, documentation), never Sonnet 5.5 (owner, 2026-11-15). The subagents (`code-reviewer`, `security-auditor`, `dependency-reviewer`) name no model and run on the model of the session, but a code review, whose model follows what it reads, not the session (owner, 2026-11-12, after the review of 17e3d4b4 and 17e3d4b5, tests alone, took 18 minutes and 271k tokens on Fable 5.2): Opus 5.5 for a pull request of tests alone or of moves alone, also in a session on Fable 5.2, a miss there leaving a weaker guard or a move to look at again, not a bug; Fable 5.2 for critical code (the matching of the game data, the guardrails of the trade site, what installs code on a player's machine); the model of the session otherwise. Claude Code gives `code-reviewer` its model when it launches it, and says it with the cost of the review: one agent, rather than a copy of it per model or a switch of the session by the owner (owner's choice on 2026-11-12); the notes of a review name at their top the model that wrote them. What Opus 5.5 misses that Fable 5.2 would find isn't measured: the line of a step says which model reviewed it. If a step proves harder than its model (a review that keeps finding medium recommendations, a design that doesn't settle), say so and propose to switch.

## Path of Exile 2 1.0

Path of Exile 2 leaves early access with its version 1.0 on 2026-12-11. The modifiers, their tiers and ranges, the trade site data and maybe the format of the copied text can change. After this release (model: Fable 5.2, the game data generated again and their matching checked):

- copy real items again (English and French) and rerun the parser tests;
- check the stat ids, the pseudo stats and the filters of the trade site, and the categories of poe2scout;
- see whether a macOS client comes with 1.0 (none on 2026-11-01: `"mac": false` on Steam for the app 2694490, while Path of Exile 1 has one; the players of a Mac run the Windows client through CrossOver or Whisky). If one does, the price-check hotkey on macOS (left out of step 14a2 by the owner on 2026-11-01): `global-hotkey` (Carbon) and the copy sent with CGEvent through `objc2-core-graphics`, already a dependency of `arboard`, whose functions are safe (`CGEvent::new_keyboard_event`, `set_flags`, `post`, Ctrl read with `CGEventSource::flags_state`); the copy of the game is Ctrl+C there too (`Ctrl + C` on every system in Exiled Exchange 2, which presses Ctrl again on a Mac); the Accessibility permission, needed to send keys, asked at the first Ctrl+D only (`CGPreflightPostEventAccess`, `CGRequestPostEventAccess`);
- check the names of the properties of a waystone (`waystone_properties` in `src/item/labels.rs`; French `Efficacité des Monstres` from Exiled Exchange 2, not seen in a copy yet, but written so by the French trade site on 2026-10-31), the same names among the `properties` of `/fetch` on both trade sites (read for the tooltip of an offer, 13k5c2) and their ids among the `map_filters` of the trade site (`/data/filters`; `map_rare_monsters` is "Monster Rarity" and `map_magic_monsters` "Monster Effectiveness", checked by two searches on 2026-10-31): an unknown name is logged (`ParseWarning::UnknownWaystoneProperty`);
- check the words a Time-Lost jewel writes before its lines (`radius_skills` in `src/item/labels.rs`, English and French): reworded, a line shows them whole, without its tag of the passive skills;
- check the properties of a gem in `/fetch` (`Level` type 5, the quality type 6, `gemSockets`) and the types of its `requirements` (62 to 65; the strength, type 63, not seen yet), read for the tooltip of an offer (13k5d2);
- check the tiers and ranges given by the advanced copy, and generate the game data again (stat ids and modifier tables; the checklist of the section "Generator" of [docs/game-data.md](docs/game-data.md)), then record again the tests on the tables of the game (the `game_tests.rs` files of `src/game_data/modifiers/`, the draw of `random_items.rs` among them; the `tests.rs` files, on tables written by hand, must pass as they are); then decide on the `divisor` of a line without a number, which no table holds (parser review of 17e3, recommendation 14);
- check the bases of the modifiers of the essences and of the alloys (`ESSENCES` and `ALLOYS` in `tools/game-data/src/modifiers/currencies.rs`) against Craft of Exile, poe2db and the trade site, and add the new ones;
- compare the crafts of the game (the crafted modifiers that aren't an alloy's, e.g. the `Motivating` of the owner's sceptre) with the lines of the alloys: crafts have no table, so a line marked crafted from the in-game trade that only an alloy holds is named "Alloy" (`Table::may_hold` in `src/game_data/modifiers/tables.rs`; no case known on 2026-10-31, second code review of 13k3d2); if a craft shares a line and a range with an alloy on a base, name no currency for that line.
- then a second refactoring pass (model: Opus 5.5), with the method of 17e3d: the code that changed for 1.0, what the reports of the first players pointed at (owner, 2026-11-11), and the section Code of [backlog.md](backlog.md).

## Steps

- [x] **0. Mock-ups** (#4)
- [x] **1. Application window** (#5)
- [x] **2. Clipboard watching** (#7)
- [x] **3. Item parser** (#9)
- [x] **4. Interface language** (#10)
- [x] **5. Settings** (#11)
- [x] **6. Title bar** (#12, #13, #14)
- [x] **7. Price data** (#15, #16)
- [x] **8. French item names** (#17)
- [x] **9. Price display** (#18, #19)
- [x] **10. Item stats and filters** (#20, #23, #25, #26)
- [x] **11. Trade site client** (#28)
- [x] **12. Offers display** (#29, #30)
- [x] **13. Estimated prices** (fifty-four pull requests, #31 to #148)
- [x] **14. Overlay POC** (fifteen pull requests, #150 to #165)
- [x] **15. Update notice** (#166)
- [x] **16. Bug report** (#167, #168, #169)
- [x] **17. First release** (#170 to #233; v0.1.0 published on 2026-11-14)
- [x] **18. Donation method** (#234, #236)
- [x] **Urgent fix: the `<<set:…>>` tags in the names of the offers** (#241, v0.1.1): the tooltip of an offer showed the tags the trade site writes before the name of a rare item, taken for font sizes.
- [x] **Urgent fix: the grammar of the names of the offers** (#244, #245, v0.1.2; model: Opus 5.5): the `<<set:…>>` tags of #241 are the forms of the first word of a name (gender, number), which choose its next words, `<if:MS>{ courroucé}<elif:FS>{ courroucée}`, on the French trade site: the choices showed in the tooltip, after a space. Resolved in the name and in the whisper, checked on real answers of `/fetch` (French and English sites). One code review on Opus 5.5, 6 minutes and 49,000 tokens: one medium and four low, all applied. The security audit of the diff (Opus 5.5, 6 minutes, 64,000 tokens) found the reading quadratic on a tag left open (a megabyte in 21 s, from the trade site alone) and a tag closing far away hiding the text between, the price of a whisper: each tag given up at its first unexpected character, and the forms set kept once each (#245; one code review on Opus 5.5, 7 minutes and 61,000 tokens: one high, the forms set again and again still quadratic, and four low, applied but `split_at`, which `clippy.toml` forbids).
- [x] **19. First retrospective** (#237, #238, #240, #243, #247, #248; rust-desktop-template#1 to #9, rust-cli-template#29 to #34, ts-astro-static-website-template#18, #19)
- [ ] **20. More settings** (model: Opus 5.5): no player has asked for anything yet (no issue three days after v0.1.0, and most players are expected with Path of Exile 2 1.0), so in two parts (owner, 2026-11-17): first the settings that need no player report, then the polish of 22a to 22d below, then the pass of 1.0; then the entries of [backlog.md](backlog.md) (Features, Settings) the players ask for.
  - [x] **20a, 20b, the first part** (#249 to #252): closing with the game, the pseudo totals shown or hidden.
- [ ] **21. Automatic updates** (model: Fable 5.2): after the first release, if the application finds its players: the new version downloaded in the background and installed at the next launch, like Discord, with a signature checked before installing anything; Velopack (a real installer for Windows and macOS) or the `self_update` crate (the executable replaced), and Flathub for Linux, whose system updates it; to decide with the installers of the first release.
- [ ] **22. Polish** (model: Opus 5.5): the last touches, picked with the owner from [backlog.md](backlog.md): its sections Polish, Edge cases and Waiting on others, and the features of step 22. 22a to 22d come before the pass of 1.0, right after 20b (owner, 2026-11-17: what needs no player report nor the data of 1.0, and surely brings value); each one mock-up first:
  - [x] **22a, the small texts of the item block** (#253)
  - [x] **22b, the uncertain prices** (#254)
  - [ ] **22c, the error texts** (model: Opus 5.5): the banner « The price database can't be reached » also shown when poe2scout answers but its answer can't be read (`src/session.rs`): a text of its own, or none (network review of 17e3); a refusal of the trade site that isn't a limit (HTTP 403) read "Trade site limit" in the waiting block (`src/ui/offers_block.rs`; `TradeError::Refused` carries no status): a text of its own, e.g. "The trade site refused the search" (security audit of 2026-11-10); the log of the bug report cut at its last 60,000 bytes (`LOG_BUDGET_BYTES` in `src/bug_report.rs`) told by a first line such as `… (first lines left out)` (code review of 17e3d11c). Texts in both languages. Two sub-steps in one pull request, with a stop for the owner's approval of the mock-up between them (Claude Code's split, agreed by the owner on 2026-11-18); one code review on Opus 5.5; no test by hand (neither a refusal nor an answer the application can't read can be brought about by hand):
    - [x] **22c1, the mock-up and the specifications** (commits: the archive of 22b and this split; the mock-up section "Error texts (22c)", two or three leads for each text, the owner's choices of 2026-11-18: 1 B, its own words and "(unreadable)" in the status bar, with one sentence per line in a band too narrow; 2 B, "Refused by the trade site" and "If it happens again, report it."; 3, the first line of "Copy the log" only; `specs.md`).
    - [ ] **22c2, the code** (commits: the identifiers of the messages moved out of `src/i18n.rs`, 1,164 lines, to `src/i18n/ids.rs` ("key" names the keys of the keyboard), moves only; the answer of the price database that can't be read, told apart from no answer, and a warning of the band that doesn't fit on one line written one sentence per line (owner, 2026-11-18); the refusal of the trade site that isn't a limit, told apart from a limit, the status carried by `TradeError::Refused`; the cut log told by the first line of "Copy the log", the preview and the address never reaching the cut; each with its texts in both languages and its tests).
  - [ ] **22d, why a line copied from the in-game trade has no tier** (model: Fable 5.2, the matching of the game data; owner, 2026-11-11: "important"): a line the application can't tell apart shows no tier, no bar and no word (the owner's choice of 2026-10-29), and the only note is "The tiers are found from the values." under the stats; since 17e3e, 7.6 % of the lines of the draw are without tier (4.6 % before), which a player may take for a bug. A word in gray in place of the tier, a tooltip, or a second sentence in the note; the causes are several: pure modifiers or their hybrid, a hybrid whose lines don't follow each other, a line that adds up modifiers in several ways, two readings as good, a sanctified item (no tier at all, no note), a search that stopped at its bound. Mock-up first (the best 2 or 3 options); `doubts::doubts` would then keep why a line is in doubt (`PartRead` knows it for each part of the item).
- [ ] **23. Second retrospective** (model: Opus 5.5): much later, after the polish and the players' reports (owner, 2026-11-17), the same sorting on the lessons written since the first one, with the players' reports as well (the automatic updates and the installers of step 21 above all, for the template).
- [ ] **24. Price fallback** (model: Fable 5.2): a plan B, after the polish, for when poe2scout can't be reached: the price of a currency or a unique from the trade site (its currency exchange, or the median of its cheapest offers), on a click only, with a search block of its own in the price block; the cached prices stay shown with their age until then. Within the [Trade site usage](docs/trade-site-usage.md) guardrails: never when an item is copied, never in the background, never a local copy of poe2scout (poe2scout reads the currency exchange API of Grinding Gear Games and the trade site from one server; every player doing it would be far more requests). Mock-up first.
- [ ] **25. Responsive layout and price history** (model: Opus 5.5): at the very end, after the price fallback (owner's choice on 2026-10-31): wireframes then mock-up of the medium and wide layouts with the visual style, then these layouts in the application, with the 7-day history chart in the wide layout.
