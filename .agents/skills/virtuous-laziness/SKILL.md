---
name: virtuous-laziness
description: A compact agent skill for making smaller, cleaner, more maintainable changes by avoiding unnecessary code, abstractions, dependencies, and output.
license: MIT
metadata:
  author: omouta
  version: "1.0.0"
---

# Virtuous Laziness

Use this skill when changing code, docs, architecture, workflows, or project structure.

## Goal

Leave the system simpler, smaller, and easier to maintain.

Do not optimize for amount of output. Optimize for the smallest durable change that solves the real problem.

## Core rule

Before adding anything, ask:

> What can be avoided, reused, deleted, or simplified?

Good work often means writing less code, not more.

## Default behavior

1. Understand the existing shape before acting.
2. Reuse current patterns unless they are clearly harmful.
3. Prefer the smallest change that solves the task.
4. Avoid new files, dependencies, abstractions, configs, and frameworks unless clearly justified.
5. Delete dead, duplicated, or unnecessary code when safe.
6. Keep names, APIs, and flows boring and obvious.
7. Leave a short explanation of what changed and why.

## Avoid

- Adding code “just in case”.
- Creating abstractions before repetition proves they are useful.
- Generating large rewrites when a local fix is enough.
- Adding dependencies for small problems.
- Producing extra docs, tests, helpers, or folders only to look thorough.
- Treating lines of code, number of files, or task count as progress.

## Prefer

- Deleting over adding.
- Reusing over inventing.
- Simple functions over clever abstractions.
- Explicit code over hidden magic.
- Local fixes over broad rewrites.
- One clear path over many configurable paths.
- Maintenance ease over impressive output.

## Change checklist

Before finalizing, verify:

- Does this solve the actual request?
- Is this the smallest reasonable change?
- Did I avoid unnecessary new surface area?
- Can a human understand this quickly later?
- Did I remove or simplify anything made obsolete?
- Are the tradeoffs clear?

## When to add abstraction

Add an abstraction only when most are true:

- The same idea appears repeatedly.
- The abstraction makes usage simpler, not just different.
- The name is obvious.
- The abstraction hides real complexity.
- Future changes become easier.
- The abstraction does not create a new mental model for little gain.

## When using LLMs or agents

LLMs are cheap at producing work, so be extra strict.

Use generation for exploration, scaffolding, refactoring, and removing toil. Do not let generated volume decide what should exist.

If the agent creates many files or a large diff, it must justify why a smaller change was not enough.

## Final response format

Report:

- What changed.
- Why this was the smallest useful approach.
- Any risks or follow-up cleanup.
