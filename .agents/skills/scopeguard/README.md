# scopeguard

Turn specs into traceable tasks and prevent agents from drifting out of scope.

`scopeguard` is a lightweight agent skill for coding agents. It helps turn a product spec, feature brief, GitHub issue, or roadmap into implementation tasks that stay connected to the original requirements.

The goal is simple: finishing every task should actually mean the spec was implemented.

## Why scopeguard exists

Coding agents are good at generating task lists, but they often miss small parts of the original spec. Those small misses compound over time.

You can end up with every task marked as done while the product is still incomplete, inconsistent, or out of scope.

`scopeguard` fixes this by making agents work from traceable requirements instead of loose task lists.

## What it does

`scopeguard` helps your agent:

* extract clear requirements from a spec
* turn those requirements into implementation tasks
* link every task back to the original spec
* audit for missing or partial coverage
* verify completed work against the requirements
* keep scope decisions explicit instead of silently dropping details

## Install

```bash
npx skills add OMouta/scopeguard
```

## Use

Ask your coding agent to use `scopeguard` when planning or implementing from a spec.

Example:

```txt
Use scopeguard on this spec. Extract the requirements, create traceable tasks, and audit coverage before implementation.
```

For implementation:

```txt
Use scopeguard and implement the next task. Keep it aligned with the linked requirements and update coverage when done.
```

For review:

```txt
Use scopeguard to audit whether the current implementation satisfies the original spec.
```

## When to use it

Use `scopeguard` when you are working from:

* product specs
* feature specs
* GitHub issues
* PRDs
* implementation plans
* roadmaps
* agent-generated task lists that need review

It is especially useful when the work spans multiple tasks or multiple agent sessions.

## Principle

Task completion is not spec completion.

`scopeguard` keeps the original spec visible through the entire implementation process, so agents can plan, build, and verify against what was actually requested.

## License

[MIT](LICENSE)
