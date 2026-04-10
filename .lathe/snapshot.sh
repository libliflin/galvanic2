#!/usr/bin/env bash
# snapshot.sh — Galvanic project snapshot.
# Collects build/test status and project state for the lathe agent.

set -uo pipefail

echo "# Project Snapshot"
echo "Generated: $(date)"
echo ""

echo "## Git Status"
git status --short 2>/dev/null || echo "(not a git repo)"
echo ""

echo "## Recent Commits (last 5)"
git log --oneline -5 2>/dev/null || echo "(no commits)"
echo ""

echo "## Source Layout"
if [[ -f Cargo.toml ]]; then
  find src tests benches -maxdepth 2 -name "*.rs" 2>/dev/null | sort || true
  echo ""
  echo "### Cargo.toml (dependencies)"
  grep -A 5 '^\[dependencies\]' Cargo.toml 2>/dev/null || echo "(no [dependencies] section)"
else
  echo "(no Cargo.toml — project not yet initialized)"
fi
echo ""

echo "## Build Status"
if [[ -f Cargo.toml ]]; then
  if cargo build 2>&1; then
    echo "cargo build: OK"
  else
    echo "cargo build: FAILED"
  fi
else
  echo "(skipped — no Cargo.toml)"
fi
echo ""

echo "## Test Status"
if [[ -f Cargo.toml ]]; then
  if cargo test 2>&1; then
    echo "cargo test: OK"
  else
    echo "cargo test: FAILED"
  fi
else
  echo "(skipped — no Cargo.toml)"
fi
echo ""

echo "## Clippy"
if [[ -f Cargo.toml ]]; then
  if cargo clippy -- -D warnings 2>&1; then
    echo "clippy: OK"
  else
    echo "clippy: FAILED"
  fi
else
  echo "(skipped — no Cargo.toml)"
fi
echo ""

echo "## Milestone Count"
if [[ -f tests/e2e.rs ]]; then
  count=$(grep -c '// ── Milestone' tests/e2e.rs 2>/dev/null || echo 0)
  last=$(grep '// ── Milestone' tests/e2e.rs 2>/dev/null | tail -1 || echo "(none)")
  echo "Milestones: $count"
  echo "Last: $last"
else
  echo "(no tests/e2e.rs yet)"
fi
echo ""

echo "## FLS Coverage Gaps"
echo "### Parse-acceptance fixtures without e2e tests"
if [[ -f tests/fls_fixtures.rs && -f tests/e2e.rs ]]; then
  # This is a heuristic: fixture test names vs e2e milestone references
  echo "(manual review needed — cross-reference fls_fixtures.rs sections with e2e.rs milestones)"
else
  echo "(tests not yet created)"
fi
echo ""

echo "## TODOs"
grep -rn 'TODO\|FIXME\|HACK' --include='*.rs' . 2>/dev/null | head -20 || echo "(none or no .rs files)"
echo ""

echo "## CI Workflows Present"
ls .github/workflows/ 2>/dev/null || echo "(none)"
