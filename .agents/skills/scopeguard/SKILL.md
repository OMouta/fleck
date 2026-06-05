---
name: scopeguard
description: Turn specs into traceable tasks and prevent agents from drifting out of scope.
license: MIT
metadata:
  author: omouta
  version: "1.0.0"
---

# scopeguard

Use this skill when a coding task starts from a spec, product brief, GitHub
issue, PRD, roadmap, or existing task list that needs to remain traceable to
the original request.

The core principle is:

> Task completion is not spec completion. Coverage is.

Treat the spec as the contract. Preserve traceability from spec to requirements
to tasks to implementation evidence. Optimize for preserving intent, not filling
a template.

## Operating Rules

- Silent omission is forbidden. If a requirement is not handled, mark it missing,
  partial, deferred, or blocked.
- Deferred work must be explicit and tied back to the affected requirement.
- Ambiguities must be recorded instead of guessed silently.
- Scope expansion must be recorded as a decision before implementation relies on
  it.
- Preserve user-visible behavior, constraints, edge cases, and non-functional
  requirements. Do not collapse them into vague implementation tasks.
- Prefer the project's existing conventions over a fixed structure.
- Keep artifacts compact and useful. Do not duplicate the full spec in generated
  artifacts.
- Use stable IDs for requirements and tasks so work can survive multiple agent
  sessions.

## Project Artifacts

When useful, create or maintain small project-local artifacts, preferably under
`.scopeguard/`. Adapt filenames to the repo if it already has a better convention.

Useful artifact types:

- requirements: short indexed requirements extracted from the spec
- tasks: implementation work linked to requirement IDs
- coverage: current coverage status and evidence
- decisions: ambiguities, deferrals, and approved scope changes

Prefer short deltas over rewriting large artifacts. Avoid keeping the whole spec
in active context after extraction; preserve references back to original spec
sections, issue comments, line numbers, commit refs, or URLs instead.

## Workflow

### 1. Extract

Read the spec and identify atomic, independently testable requirements. Give
each requirement a stable ID such as `REQ-001`. Keep requirement text short and
preserve a reference to the original spec location.

Do not invent requirements outside the spec or clearly implied behavior. If the
spec is ambiguous, record the ambiguity rather than resolving it silently.

### 2. Plan

Turn requirements into implementation tasks with stable IDs such as `TASK-001`.
Every active requirement should be covered by at least one task, and every task
should trace back to the requirement IDs it supports.

Include just enough acceptance criteria for the agent to know when the task is
actually complete. Enabling or maintenance tasks are allowed, but mark them as
such and explain which requirements they unblock or protect.

### 3. Audit

Before implementation, compare the spec, requirements, and tasks. Identify:

- missing coverage
- partial coverage
- ambiguity
- scope creep
- orphan tasks
- deferred work

Before implementation, surface missing or partial coverage. Do not silently
proceed as if coverage is complete when active requirements are still missing.

### 4. Execute

Implement one task at a time. Before editing code, load only the current task,
linked requirements, relevant spec excerpts, coverage notes, and necessary code
context.

Stay within the linked requirements unless a decision is recorded. If new
ambiguity appears during implementation, record it. Ask when the ambiguity
affects product behavior, data, security, or scope. For low-risk implementation
details, proceed with a documented assumption.

### 5. Verify

After implementation, check the completed work against the linked requirements.
Record evidence such as files changed, behavior implemented, tests run, manual
checks, and known gaps.

Do not mark a requirement covered without evidence. If the work is partial, say
so explicitly and keep the affected requirement partial or open.

### 6. Sync

When the spec changes, update requirements, tasks, and coverage. Detect added,
changed, removed, or deferred requirements. Reopen or downgrade coverage when
existing implementation no longer satisfies the changed spec.

## Context Budget

Manage context by retrieval, not repetition:

- Compress the spec into short, indexed requirements.
- Keep stable IDs for requirements and tasks.
- Load only the current task, linked requirements, relevant spec excerpts,
  coverage notes, and necessary code context.
- Prefer incremental artifact updates over rewriting large files.
- Avoid duplicating the full spec inside artifacts.

## Traceability Example

```md
REQ-004: Users can export filtered results.
Covered by: TASK-007
Evidence: Export endpoint applies active filters and includes a regression test.
Status: covered
```
