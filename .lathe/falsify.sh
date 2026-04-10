#!/usr/bin/env bash
# falsify.sh — Galvanic claims falsification suite.
#
# Runs every cycle. Checks the load-bearing claims in claims.md.
# Exit 0 if all active claims hold; exit 1 if any fail.
# Prints a summary line at the end regardless of outcome.
#
# IMPORTANT: This suite grows with the project. The source-code claims
# (build integrity, test suite, Token size, no unsafe, runtime emission,
# adversarial input) are listed as PENDING in claims.md and have no checks
# here yet. Activate them as the corresponding code is added.

set -uo pipefail

PASS=0
FAIL=0
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

ok() {
  echo "  ok: $1"
  PASS=$((PASS + 1))
}

fail() {
  echo "  FAIL: $1"
  FAIL=$((FAIL + 1))
}

# ── Claim 1: PR-rejection workflow is intact and safe ────────────────────────
#
# .github/workflows/close-prs.yml must exist. It uses pull_request_target and
# must NOT check out the PR head (no actions/checkout on PR head code, no
# reference to github.event.pull_request.head.ref in a checkout context).
# This is a safety invariant: the workflow runs with base-repo write tokens.

echo "Claim 1: PR-rejection workflow integrity"

CLOSE_PRS="$REPO_ROOT/.github/workflows/close-prs.yml"
if [[ ! -f "$CLOSE_PRS" ]]; then
  fail "close-prs.yml does not exist at $CLOSE_PRS"
else
  # Check file contains pull_request_target trigger
  if grep -q "pull_request_target" "$CLOSE_PRS" 2>/dev/null; then
    ok "close-prs.yml exists and uses pull_request_target"
  else
    fail "close-prs.yml exists but missing pull_request_target trigger"
  fi

  # Check no actions/checkout step exists (would check out PR code with elevated perms)
  if grep -q "actions/checkout" "$CLOSE_PRS" 2>/dev/null; then
    fail "close-prs.yml contains actions/checkout — PR head code could be checked out with write tokens"
  else
    ok "close-prs.yml does not use actions/checkout"
  fi
fi

# ── Claim 2: Author-signature verification workflow is active ────────────────
#
# .github/workflows/verify-author-signature.yml must exist and contain
# git verify-commit logic. If it disappears, unsigned commits could land.

echo "Claim 2: Author-signature verification workflow"

VERIFY_SIG="$REPO_ROOT/.github/workflows/verify-author-signature.yml"
if [[ ! -f "$VERIFY_SIG" ]]; then
  fail "verify-author-signature.yml does not exist at $VERIFY_SIG"
else
  ok "verify-author-signature.yml exists"

  if grep -q "verify-commit" "$VERIFY_SIG" 2>/dev/null; then
    ok "verify-author-signature.yml contains verify-commit logic"
  else
    fail "verify-author-signature.yml exists but missing verify-commit — signature checking may be broken"
  fi

  # Check that the maintainer's key fingerprint is present
  if grep -q "AAAAC3NzaC1lZDI1NTE5" "$VERIFY_SIG" 2>/dev/null; then
    ok "verify-author-signature.yml contains maintainer SSH key"
  else
    fail "verify-author-signature.yml is missing the maintainer SSH key fingerprint"
  fi
fi

# ── Claim 3: Lathe CI-status reads are scoped and structured ─────────────────
#
# Documentary backstop: agent.md must contain the load-bearing rule heading
# that tells the lathe agent never to read attacker-controllable workflow run
# fields. If this heading disappears, the prompt-injection containment story
# is silently broken. When the lathe engine source is available on this
# machine, this check should additionally grep the engine source for forbidden
# references and fail if any are found outside an explicit allowlist.

echo "Claim 3: Lathe CI-status reads are scoped and structured"

AGENT_MD="$REPO_ROOT/.lathe/agent.md"
if [[ ! -f "$AGENT_MD" ]]; then
  fail "agent.md does not exist at $AGENT_MD"
else
  if grep -q "Reading CI status safely — load-bearing rule" "$AGENT_MD" 2>/dev/null; then
    ok "agent.md contains the CI-status load-bearing rule heading"
  else
    fail "agent.md is missing the 'Reading CI status safely — load-bearing rule' heading"
  fi

  if grep -q "Forbidden endpoints and tools" "$AGENT_MD" 2>/dev/null; then
    ok "agent.md contains the forbidden endpoints section"
  else
    fail "agent.md is missing the 'Forbidden endpoints and tools' section"
  fi
fi

# ── Claim 4: Build workflow runs on push to main and uses no third-party actions ──
#
# .github/workflows/build.yml must:
#   (a) exist
#   (b) be triggered on push to branches: [main]
#   (c) declare a job named `build` (the check name lathe polls in direct mode)
#   (d) contain no `uses:` lines (no third-party actions, matching the
#       convention from close-prs.yml and verify-author-signature.yml)
#
# If any of these break, the lathe loop loses its CI signal — it pushes,
# polls /commits/<sha>/check-runs filtered to name == "build", and gets
# nothing back. The agent then has no way to know whether main is healthy.

echo "Claim 4: Build workflow integrity"

BUILD_WF="$REPO_ROOT/.github/workflows/build.yml"
if [[ ! -f "$BUILD_WF" ]]; then
  fail "build.yml does not exist at $BUILD_WF"
else
  ok "build.yml exists"

  # (b) Push trigger on main. Match a `push:` block followed (within a few
  # lines) by `branches:` containing `main`. We can't fully parse YAML in
  # bash, so this is a structural grep — close enough for the documentary
  # backstop, and the agent must keep the file straightforward.
  if awk '
    /^on:/                {in_on=1; next}
    in_on && /^[a-z]/     {in_on=0}
    in_on && /push:/      {in_push=1; next}
    in_push && /branches:/{print; exit}
  ' "$BUILD_WF" | grep -q 'main'; then
    ok "build.yml triggers on push to main"
  else
    fail "build.yml does not trigger on push to branches: [main]"
  fi

  # (c) Job named `build`. The job key must be exactly `build:` at indent 2.
  if grep -q '^  build:' "$BUILD_WF" 2>/dev/null; then
    ok "build.yml declares a job named 'build'"
  else
    fail "build.yml is missing a job named 'build' (lathe polls check name 'build')"
  fi

  # (d) No third-party actions. The convention is to clone via GH_TOKEN.
  if grep -nE '^[[:space:]]*-?[[:space:]]*uses:' "$BUILD_WF" 2>/dev/null; then
    fail "build.yml contains 'uses:' lines — third-party actions are not permitted in this repo"
  else
    ok "build.yml uses no third-party actions"
  fi
fi

# ── Pending claims (not yet active) ─────────────────────────────────────────
#
# The following claims from claims.md have no checks yet because the source
# code doesn't exist. They are listed here as comments so the runtime agent
# knows what to add as the project grows.
#
# Pending Claim 5:  cargo build succeeds (activate when Cargo.toml + src/lib.rs exist)
# Pending Claim 6:  cargo test passes (activate when tests/ exists)
# Pending Claim 7:  size_of::<Token>() == 8 (activate when src/lexer.rs has Token)
# Pending Claim 8:  no unsafe in src/ (activate when src/ has library code)
# Pending Claim 9:  runtime_add_emits_add_instruction (activate when tests/e2e.rs has the test)
# Pending Claim 10: CLI handles adversarial inputs (activate when galvanic binary exists)
#
# To activate a pending claim:
#   1. Move it from "Pending Claims" to "Active Claims" in claims.md
#   2. Uncomment (or add) the check block below
#   3. Verify falsify.sh still runs to completion and prints the summary line

# ── Summary ──────────────────────────────────────────────────────────────────

echo ""
echo "=== Summary === passed: $PASS  failed: $FAIL"

if [[ $FAIL -gt 0 ]]; then
  exit 1
fi
exit 0
