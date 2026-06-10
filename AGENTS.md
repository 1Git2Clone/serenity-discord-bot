# Agent Guidelines

## Priorities

1. User's instructions
2. Consistency with the codebase
3. This file

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
