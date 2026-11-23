---
name: refactoring-pass
description: Run a refactoring pass on the code changed since the last one (the whole repository for the first), with no change of behavior: the review of the zone, a plan the owner accepts, tests first, moves then one commit per improvement, and reviews until no high or medium finding is left. Use it for the refactoring step before each minor version, or when the owner asks for a refactoring pass.
---

# Refactoring pass

Goal: code simple, readable, easy to maintain and to change, with the same behavior. A pass doesn't make the code perfect: it ends when nothing left would cause a silent bug or make the next change cost more than it should.

The pass keeps the code clean between two changes; the rule of each change (`CLAUDE.md`, Development: refactor what the change touches before each commit) keeps it clean day to day. A pass that finds much tells that the daily rule wasn't kept, not that passes must grow.

## Needs

Stop and tell the owner if one is missing:

- the `code-reviewer` agent (`.claude/agents/code-reviewer.md`), with its section Priorities: the levels of a finding, read from its consequence;
- `docs/code-quality.md`: the rules of the code of the project, and its section Refactoring, the checks of this project after each commit;
- `roadmap.md`: the step of the pass, and the line that records the end of the last pass (Reference).

## 1. When

- **Before each minor version** (`0.3.0`, `0.4.0`…): a numbered step of `roadmap.md`, once the features of the version are merged and before its release.
- **The first pass, before the first version**, covers the whole repository: there is no pass before it, so no diff.
- **When the owner asks**, on the zone they name.

## 2. The zone

- **After a first pass**: the code changed since the end of the last one, read from `roadmap.md` (Reference): `git diff --stat <commit>..origin/main`. Leave out what only moved a version or a text.
- **The hot spots first**: the files changed most often since then (`git log --format= --name-only <commit>..origin/main | sort | uniq -c | sort -rn`) and the longest ones. A file stable and readable is left alone, even if not perfect.
- **The first pass**: the whole repository, cut into areas by module (the parser, the interface, the network…), each one a review and its own sub-steps, in the order of what depends on what: first the code most likely to change and that lengthens the reviews most, last the code that puts the rest together.
- Say the zone to the owner, with its size (files, lines changed), before the review.

## 3. The review of the zone

- Run `code-reviewer` in the area mode, one per area, at once, each told what the others read so that none reads twice. Its model follows the rule of `roadmap.md` (Models); say the model and the cost when launching.
- Its findings, each with its consequence and its priority (the section Priorities of the agent), are the input of the plan. Give the owner the count per priority.

## 4. The plan

- **Read the whole code of the zone yourself**, then propose one numbered plan, one commit per improvement: the findings of the review, and what it missed (a word shared by two notions, a boolean parameter, a long function, a clone that only quiets the compiler). The findings are the least to do.
- **Sort it by consequence**, the same levels as the agent: the high first, then the medium. The low are done when a commit already touches their file; the others, the owner decides (now or `backlog.md`).
- **Split it into sub-steps** by the rules of `CLAUDE.md` (Specifications and roadmap): one pull request each, four commits at most, written in the line of the step in `roadmap.md` before any code. Say what the plan leaves out.
- The owner accepts the plan; then it is a closed list.

## 5. Tests first

- Before changing or moving code, check that it is tested. A path without a test gets one first, which pins what the code does today: a behavior that looks wrong is pinned as it is, and its fix is a commit of its own (step 6).
- Where only the network or another program reaches the code, change it only by steps the compiler checks, and say so in the pull request.

## 6. Moves, then improvements

- **The moves first**, in commits of their own, which `git diff --color-moved=dimmed-zebra --color-moved-ws=allow-indentation-change` shows unchanged, but for the paths, the imports, the visibility and the line breaks the formatter puts again. A move of more than a few items is done by a script that can be run again from the commit before, and a second script checks that every line of the old files is in the new ones.
- **Then one commit per improvement**, each kept in mind against:
  - **KISS**: the simplest form that does the work;
  - **YAGNI**: nothing for a need that isn't there yet; easy to change means simple, not ready for every case: a layer is added the day its second case comes;
  - **DRY of the knowledge**: one rule in one place; two pieces of code alike that say two rules stay apart, and a code repeated is shared at its third place, a wrong abstraction costing more than a copy;
  - **SOLID**, as `docs/code-quality.md` reads it for the language of the project;
  - **the names first**: each name says what its thing is for.
- **The behavior stays the same after each commit**, not only at the end, so that a difference names its commit: the tests, and the checks of the section Refactoring of `docs/code-quality.md`. A test whose assertion changes is named in the pull request, with its reason.
- **On the way**: a small refactoring in the files the commit touches is done in the same pull request. Work that needs a commit of its own and that the plan doesn't name: finish the commit under way, then tell the owner what it is and its cost, and the owner decides. A small fix of the behavior goes in a commit of its own (`fix(…)`), with its test, named in the pull request as its change of behavior; one that needs a decision or a mock-up goes to `backlog.md`.

## 7. Reviews until the stop

- Each pull request of the pass gets `code-reviewer` in the branch mode, told that it is a refactoring.
- **The stop**: no high and no medium finding left. Apply the fixes the owner approves, then review again.
- **The owner's way out**: when only medium findings are left, outside the critical code and outside a rule of the domain (a matter of reading, not of logic), give them to the owner with your recommendation; the owner decides to go on without them, and they go to `backlog.md`.
- On critical code (`CLAUDE.md`, Code review), every fix gets its test, and the review runs again until no high or medium finding is left, without the way out.

## 8. The end

- Record the end of the pass in `roadmap.md` (Reference): the commit of `main` that merged its last pull request, and its date. The next pass starts from it.
- When the method of the pass had to change, write the lesson in `docs/lessons.md`; an improvement of this skill goes back to the template of the project too.
