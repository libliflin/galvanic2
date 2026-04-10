# Claims Registry

Claims are the load-bearing promises galvanic makes to its stakeholders. The falsification suite (`falsify.sh`) checks these every cycle. A failing claim is top priority — fix it before any new work.

**Current project state:** Seed — no source code, no `Cargo.toml`, no tests. The source-code claims below are listed as *pending* and will become active as the pipeline is built. Two claims about the repo's security posture are active and checkable right now.

---

## Active Claims

### Claim 1: PR-rejection workflow is intact and safe

**Stakeholders:** All — this is the security posture claim for the repo.
**Promise:** `.github/workflows/close-prs.yml` exists, uses `pull_request_target`, and does NOT check out PR head code and does NOT read free-text PR fields (title, body, branch name) in its shell scripts. The workflow only uses server-generated integers (PR number) and hex SHAs.
**Why it's load-bearing:** This repo auto-closes PRs. `pull_request_target` runs with the base repo's token. If this workflow were modified to check out the PR head, an attacker could execute arbitrary code with write access. The safety relies on the workflow never touching attacker-controlled content.
**How it's checked:** `falsify.sh` verifies the file exists and checks for absence of `actions/checkout` or any `checkout` of `${{ github.event.pull_request.head.*` patterns within the workflow.

---

### Claim 2: Author-signature verification workflow is active

**Stakeholders:** All — this is the commit integrity claim.
**Promise:** `.github/workflows/verify-author-signature.yml` exists and contains `git verify-commit` logic and the maintainer's SSH public key fingerprint.
**Why it's load-bearing:** Every change the lathe makes must be signed by the maintainer's key. If this workflow disappears, unsigned commits could land on main silently.
**How it's checked:** `falsify.sh` verifies the file exists and contains `verify-commit`.

---

### Claim 3: Lathe CI-status reads are scoped and structured

**Stakeholders:** All — this is a prompt-injection containment claim for the autonomous agent.
**Promise:** The lathe engine never reads workflow-run-scoped GitHub Actions data. It reads CI status only via commit-scoped check-runs/status endpoints (`/commits/<sha>/check-runs`, `/commits/<sha>/status`) for SHAs it has obtained from the protected `main` branch, and it consumes only structured fields (`status`, `conclusion`, `name`, `head_sha`, `started_at`, `completed_at`). It never reads `output.title`, `output.summary`, `output.text`, `pull_requests`, or any field from `actions/runs`, `gh run list`, `gh run view`, or search endpoints.
**Why it's load-bearing:** The lathe runtime is an LLM agent. Workflow run records on a public repo include attacker-controllable string fields whenever a fork PR is opened (workflow `name`, `display_title`, commit message, branch name). Reading those fields inside the agent loop is a prompt injection vector — the breach is not the workflow running, it is the agent reading attacker text from a "trusted" API response. The repo's lockdown (auto-close PRs via `close-prs.yml`, signed-commit enforcement, PR-spawned run cleanup) reduces the surface but does not eliminate the existence of attacker-controlled run records during the brief window before they are deleted. The structural fix lives in lathe: never query the unsafe endpoints, never read the unsafe fields. See `agent.md` § "Reading CI status safely — load-bearing rule".
**How it's checked:** `falsify.sh` greps `agent.md` for the load-bearing rule heading as a documentary backstop. When the lathe engine source is available on this machine, the check should additionally grep the engine source for forbidden references (`actions/runs`, `gh run list`, `gh run view`, `gh run watch`, `output.summary`, `output.text`, `pull_requests`, `/search/`) and fail if any are found outside an explicit allowlist.

---

### Claim 4: Build workflow runs on push to main and uses no third-party actions

**Stakeholders:** All — this is the trust substrate for every change the lathe pushes.
**Promise:** `.github/workflows/build.yml` exists. It triggers on `push` to `branches: [main]`. It contains a job named `build` (the check name the lathe loop polls in direct mode). It contains no `uses:` lines (no third-party actions), matching the convention set by `close-prs.yml` and `verify-author-signature.yml` — source is fetched via `git clone` with the runner-issued `GITHUB_TOKEN`.
**Why it's load-bearing:** The lathe loop pushes directly to main and decides whether to move on or fix a failure based on this workflow's result. If `build.yml` is missing, mis-triggered (e.g. on `pull_request` instead of `push`), or renames its `build` job, the loop sees no check and the agent has no signal. If a third-party action is added, the "Allow libliflin actions" repo setting can silently block the workflow, also leaving the loop with no signal — and adding supply-chain risk to a workflow that runs with the repo's `GITHUB_TOKEN`.
**How it's checked:** `falsify.sh` verifies (a) the file exists, (b) it has a `push:` trigger with `branches: [main]`, (c) it declares a job named `build`, and (d) it contains no `uses:` lines.

---

## Pending Claims (activate when source code exists)

These claims cannot be checked yet because the source code doesn't exist. The runtime agent should activate each claim in `claims.md` and add the corresponding check to `falsify.sh` when the prerequisite code exists.

### Pending Claim 5: Build integrity
**Activate when:** `Cargo.toml` and `src/lib.rs` exist.
**Promise:** `cargo build` succeeds with no errors and `cargo clippy -- -D warnings` emits no warnings.
**Stakeholders:** All.

---

### Pending Claim 6: Test suite passes
**Activate when:** `tests/` directory and at least one test file exist.
**Promise:** `cargo test` exits 0.
**Stakeholders:** FLS conformance researcher, cache researcher, Sunday contributor.

---

### Pending Claim 7: Token stays 8 bytes
**Activate when:** `src/lexer.rs` exists with a `Token` type.
**Promise:** `size_of::<Token>() == 8` — the lexer's hot-path type fits 8 tokens per 64-byte cache line.
**Why structural, not documentary:** This claim fails when the struct grows, regardless of what the doc comment says. A doc comment update does not satisfy this claim. See `claims.md` "Adding New Claims" for the structural vs. documentary distinction.
**Stakeholders:** Cache-aware codegen researcher.

---

### Pending Claim 8: No unsafe code in library source
**Activate when:** `src/` exists with more than `main.rs`.
**Promise:** No `unsafe` blocks, `unsafe fn`, or `unsafe impl` in `src/` (excluding `src/main.rs`).
**Stakeholders:** Sunday contributor, FLS conformance researcher.

---

### Pending Claim 9: Runtime instruction emission (no const-fold in non-const functions)
**Activate when:** `tests/e2e.rs` exists with an assembly inspection test for basic arithmetic.
**Promise:** A non-const function that evaluates `1 + 2` emits a runtime `add` instruction, not a folded `mov x0, #3`.
**Why it's load-bearing:** FLS §6.1.2:37–45 is the heart of the conformance research question. A compiler that constant-folds non-const code looks correct on exit-code tests but is semantically wrong.
**Stakeholders:** FLS conformance researcher.

---

### Pending Claim 10: CLI handles adversarial inputs without panicking
**Activate when:** `src/main.rs` exists and the project can be built as a binary.
**Promise:** The galvanic binary does not panic (exit > 128) when given: empty files, binary garbage, NUL bytes, deeply nested braces (500 levels), or large inputs (10k let bindings).
**Stakeholders:** CI/validation infrastructure, Sunday contributor.

---

## Retired Claims

*(None yet. Claims are retired here when they no longer fit the project, with the date and reasoning.)*

---

## Adding New Claims

When a new milestone introduces a new structural promise:

1. Add an entry here with: stakeholder, promise, why it's load-bearing, how it's checked, and whether it's **structural** (fails if code changes) or **documentary** (fails if docs change).
2. Add a corresponding check to `falsify.sh`.
3. **Choose structural over documentary whenever possible.** A claim that can be satisfied by editing comments is not a structural claim. Structural claims use `size_of::<T>()` assertions, AST/grep checks against declarations, or running the actual test.
4. The sharp test: if someone could satisfy this claim by only editing comments, it is not a structural claim.
5. Keep the total number of active claims in the 3–10 range. Claims that run slowly defeat the purpose. New claims should replace weaker ones where possible.
6. When activating a pending claim, move it from "Pending" to "Active" and add the `falsify.sh` check.
