# Alignment Summary

*Read this in 30 seconds and gut-check before starting cycles.*

---

## Who This Serves

- **William as FLS conformance researcher** — testing whether the Ferrocene Language Specification is independently implementable, milestone by milestone, with every decision traceable to a `FLS §X.Y` citation and no rustc-internal knowledge.
- **William as cache-aware codegen researcher** — testing what a compiler looks like when cache-line alignment is a first-class structural constraint from the start, not an optimization pass at the end.
- **The Sunday contributor** — someone who finds this interesting and wants to add a FLS section; needs `cargo build` to work and clear testing patterns to follow.
- **CI/validation infrastructure** — the trust substrate. Currently only two security workflows exist (no build CI). The lathe's work is only as trustworthy as the CI that validates it.

---

## Key Tensions

- **Bootstrapping velocity vs. design discipline:** Getting to milestone 1 quickly vs. not making early decisions that paint the project into a corner. Resolved by following the proven architecture from `.lathe/skills/architecture.md` rather than inventing a new one.

- **FLS fidelity vs. convenience:** The whole value of the project is spec-only discipline (no rustc cheating). The agent must never take a convenient path that papers over a spec ambiguity. See `.lathe/refs/fls-constraints.md`.

- **Cache-line documentation vs. enforcement:** `size_of` assertions enforce the structural promise; doc comments don't. Add assertions only for types that actually have a budget claim. Aspirational comments are fine but don't make them claims.

---

## Load-Bearing Claims

These are the promises encoded in `.lathe/claims.md` and checked every cycle by `falsify.sh`:

**Active (checkable now):**
1. **PR-rejection workflow is intact** — `.github/workflows/close-prs.yml` exists, uses `pull_request_target`, and does not check out PR head code. This is a safety invariant — the workflow runs with base-repo write tokens.
2. **Author-signature verification is active** — `.github/workflows/verify-author-signature.yml` exists and contains `git verify-commit` logic with the maintainer's SSH key.

**Pending (activate as source code is built):**
3. **Build integrity** — `cargo build` and `cargo clippy -- -D warnings` succeed. *(activate when Cargo.toml + src/ exist)*
4. **Test suite passes** — `cargo test` exits 0. *(activate when tests/ exists)*
5. **Token is 8 bytes** — `size_of::<Token>() == 8`. *(activate when src/lexer.rs has Token)*
6. **No unsafe in library source** — grep check on `src/`. *(activate when library code exists)*
7. **Runtime instruction emission** — `fn main() -> i32 { 1 + 2 }` emits `add`, not `mov x0, #3`. *(activate when tests/e2e.rs has the assembly inspection test)*
8. **CLI handles adversarial inputs** — no panic on empty file, binary garbage, etc. *(activate when galvanic binary exists)*

---

## Current Focus

The project is at **Stage 0: blank slate**. No `Cargo.toml`, no source files, no tests, no build CI.

The agent's first priority cycle should be creating the `Cargo.toml` and `src/` scaffold. The second should be creating a minimal GitHub Actions `build` job (`cargo build && cargo test`). Everything else follows.

The proven architecture to build toward is documented in `.lathe/skills/architecture.md`. Build it exactly — don't redesign it.

---

## Repository Security

- This repo uses `pull_request_target` in `close-prs.yml`. The workflow is written safely (no checkout of PR head), and Claim 1 verifies this every cycle.
- The repo is at `libliflin/galvanic2`. If it's public, the injection surface is higher (external contributors can file issues with injected text) — but the lathe engine only reads structured data from GitHub, not free-text fields.
- Branch protection status was not verified during init. Before starting cycles, check that the `main` branch requires commit signatures (the `verify-author-signature.yml` workflow enforces this at the CI level, but the ruleset setting should also be on).

---

## What Could Be Wrong

**`falsify.sh` was not executed during init.** The init environment blocked `chmod` and `bash` execution. The script's logic was reviewed manually:
- All `grep` calls are inside `if grep -q` patterns, which safely capture exit codes and cannot trigger `pipefail`.
- No `grep` calls appear in pipelines.
- The summary line (`=== Summary === passed: N  failed: M`) is at the end.
- The script should run cleanly, but you should verify before starting cycles:
  ```bash
  chmod +x .lathe/falsify.sh
  bash .lathe/falsify.sh
  ```
  Confirm the output ends with `=== Summary === passed: 5  failed: 0` (or close to it).

**This is a reinit of a compromised repo.** The `.lathe-bku/` directory contains the old lathe files from the original `galvanic` repo (at milestone 197, with 1700+ tests and a full pipeline). Those files describe the target end state, not the current state. The agent should NOT assume that code described in `.lathe-bku/` exists in this repo — it doesn't. Read the actual file system.

**No source code means no research is happening yet.** The first ~5 cycles are pure infrastructure (Cargo.toml, src/ scaffold, CI). This is necessary but does not advance the FLS conformance or cache research questions directly. William may want to speed through this phase.

**Workflow files lack a CI job for the project itself.** The only CI today is the two security workflows. Adding `ci.yml` with a build job is high priority but must be done as a signed commit directly to main (the repo rejects PRs).

**The `snapshot.sh` now runs `cargo build` and `cargo test` on every cycle.** These will print "skipped — no Cargo.toml" until the project is initialized, then they will run on every snapshot. If the full test suite grows large, the snapshot will slow down. Consider switching `cargo test` in snapshot.sh to `cargo test --lib` (fast unit tests only) and leaving the full test run to CI once the suite is substantial.

**The Sunday contributor stakeholder is aspirational.** There's no `CONTRIBUTING.md` and no "what's next" map. Once the first few milestones are done, adding a brief "current milestone / what comes next" section to the README would substantially improve this stakeholder's journey.
