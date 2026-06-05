# Virtuous Laziness

A compact agent skill for making smaller, cleaner, and more maintainable changes.

AI agents are very good at producing more: more code, more files, more abstractions, more dependencies, more output. Good engineering often requires the opposite. This skill teaches agents to reduce unnecessary work by choosing the smallest durable change that solves the real problem.

## Why this exists

As agents become more capable, they gain more power to change systems. Without restraint, that power can easily turn into bloated diffs, unnecessary rewrites, and codebases that become harder to understand.

Virtuous laziness is not about avoiding useful work. It is about avoiding future waste.

Before adding anything, an agent should ask:

> What can be avoided, reused, deleted, or simplified?

The goal is to leave the system easier to reason about than it was before.

## Inspiration

This skill was inspired by Bryan Cantrill’s post [The peril of laziness lost](https://bcantrill.dtrace.org/2026/04/12/the-peril-of-laziness-lost/), which argues that laziness is a programming virtue when it leads to better abstractions, simpler systems, and lower long-term maintenance cost.

It was also inspired by Theo Browne’s [video](https://youtu.be/iN_9aH3VuzU), which discusses the post and how this idea applies to modern AI-assisted development.

## Install

Using the Vercel Skills CLI:

```bash
npx skills add omouta/virtuous-laziness
```

## What it does

The skill guides an agent to:

* understand the existing project before changing it
* reuse current patterns
* avoid unnecessary dependencies or abstractions
* prefer deletion and simplification
* keep changes easy to review
* explain why the chosen approach is small and useful

## Principle

> Before adding anything, ask: what can be avoided, reused, deleted, or simplified?

## License

MIT
