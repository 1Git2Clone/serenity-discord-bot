# Agent Guidelines

## Priorities

1. User's instructions
2. Consistency with the codebase
3. This file

---

## Architecture

The non-obvious architecture lives in
[docs/architecture.md](./docs/architecture.md) and `handoff/DOCS_KEYNOTES.md`.
Two gotchas up front: `/ai-review` only works against DeepSeek, and the `ai`
Cargo feature implies `redis`.

---

## Core Rules

### KISS — Keep It Simple

Write straightforward code. No `pub(super)` visibility gymnastics, no indirection
for its own sake, no abstractions that exist only to satisfy a module boundary.
If the code is hard to follow, it's wrong.

### YAGNI — Don't Build What Isn't Asked For

- No features beyond what was requested.
- No abstractions for single-use code.
- No "flexibility" or "configurability" that wasn't asked for.
- No error handling for scenarios that can't happen.

**On modules specifically:** if a unit of code is used once, inline it. If it's
used twice, inline it again — two callsites is probably a coincidence. Only
consider extracting at three or more uses, and even then just suggest it rather
than doing it unilaterally.

---

## Think Before Coding

Don't assume. Don't hide confusion. Surface tradeoffs.

- State assumptions explicitly — ask rather than guess when uncertain.
- Present multiple interpretations instead of picking silently.
- Push back when a simpler approach exists.
- Stop when confused — name what's unclear and ask.

---

## Simplicity First

Minimum code that solves the problem. Nothing speculative.

If 200 lines could be 50, write 50. A senior engineer should not look at the
diff and say "this is overcomplicated."

---

## Surgical Changes

Touch only what the task requires.

- Don't improve adjacent code, comments, or formatting.
- Don't refactor things that aren't broken.
- Match existing style even if you'd do it differently.
- If you spot unrelated dead code, mention it — don't delete it.
- Remove only the imports/variables/functions that *your* changes made unused.

Every changed line should trace directly to the user's request.

---

## Comments & Docs Register

Write like a contributor, not a narrator. Plain, terse doc and inline comments.
No markdown bold, no hype, no restating the code in prose. A comment earns its
place by explaining something the code can't say for itself — a why, a gotcha, a
ceiling. If it just paraphrases the line below it, delete it.

---

## Keep Docs In Sync

Treat docs as part of the change, not an afterthought. When a change is
user-facing or otherwise notable, update them in the same commit/PR:

- `CHANGELOG.md` — add an entry under `[Unreleased]` (match the existing
  Added/Changed/Fixed prose register). Pure CI/repo-plumbing changes don't need
  one; user-visible behavior and commands do.
- `README.md` — if the change adds/alters a command, feature flag, env var, or
  setup step described there.
- `docs/` — if it touches architecture or behavior documented under `docs/`.

If you decide a change needs none of these, say so briefly rather than skipping
silently.

---

## Migrations Are Append-Only

Never edit or reorder a migration once it has been applied anywhere. To change
the schema, add a new migration with an additive `ALTER`. Editing an applied
migration desyncs every environment that already ran it. After changing
queries, regenerate `.sqlx/` with `cargo sqlx prepare` so offline builds and CI
stay in sync.

---

## Reuse Over Macros

When you do deduplicate (at three-plus uses — see YAGNI), reach for spec structs
and shared functions, not proc-macro DSLs. LSP support — go-to-definition,
rename, completion — matters more than terseness. The `InteractionSpec` /
`run_interaction` pattern in `embed_commands/spec` is the shape to follow.

---

## Verification Gates

CI runs clippy with `-D warnings` across three feature configs; match it locally
before pushing — code that compiles under one config can break under another,
which is exactly what the matrix catches:

- `cargo fmt --check`
- `cargo clippy --all-targets -- -D warnings` for each of: no features,
  `--features "opentelemetry ai-deepseek"` (the deployed set — see
  `scripts/deploy-features.sh`), and `--all-features`.
- Real tests, not just `cargo check`: feature-gated code needs Postgres and
  Redis. Bring them up with `docker compose up -d db redis`, run
  `sqlx migrate run`, then `cargo test`. `cargo check` alone never exercises the
  gated paths.

---

## Goal-Driven Execution

Define success criteria and loop until they're met.

For non-trivial tasks, state a brief plan with a verifiable check per step:

```
1. [step] → verify: [how to confirm it worked]
2. [step] → verify: [how to confirm it worked]
```

Prefer writing tests first when the task is a bug fix or a well-defined
feature: a failing test that reproduces the problem is a stronger success
criterion than "make it work."

---

## Commit Conventions

Use conventional commits: `type(scope): short description`.

### Format

```
type(scope): short description

Body paragraphs explaining what and why. Keep them concise.

Co-authored-by: Name <email>
```

- **Types**: `feat`, `fix`, `chore`, `docs`, `style`, `refactor`, `test`, etc.
- **Scopes**: scan `git log --oneline -50` before committing and match existing
  naming, casing, and granularity. Don't invent new scopes without checking.
- **Co-author**: append `Co-authored-by: DeepSeek V4 Pro <service@deepseek.com>`
  for tool-assisted commits.

### Before committing

- Run `git diff --cached --check` to catch trailing whitespace and merge
  conflict markers.
- If the commit is an amend, confirm it hasn't been pushed first and verify
  the branch.
