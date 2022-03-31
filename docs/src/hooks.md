# Hooks

Hooks are functions that let you store state and perform side-effects.

reView comes with a few pre-defined Hooks. You can also create your own custom hooks.

## Rules of hooks

- A hook function name always has to start with use_
- Hooks can only be used at the following locations:
  - Top level of a function / hook.
  - If condition inside a function / hook, given it's not already branched.
  - Match condition inside a function / hook, given it's not already branched.
  - Blocks inside a function / hook, given it's not already branched.
- Every render must call the hooks in the same order

All these rules are enforced by either compile time or run-time errors.

## Pre-defined Hooks

reView comes with the following predefined Hooks:
- [use_state](state-hook.md)
- [use_effect](effect-hook.md)