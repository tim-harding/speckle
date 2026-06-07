# Design Revision — Gestalt / Wasp / Possum

*A concise summary of the revised concept. Draft — 2026-06-07.*

## The shape of the change

What began as Speckle (AI-implemented specifications bolted onto Rust
proc-macros) and passed through Possum 0.2 (a single typed store with
projections) resolves into a **three-layer tower sharing one type system
(WIT)**. The proc-macro awkwardness was a symptom of hosting the idea on Rust;
the tower is what it wanted to be.

## The three layers

- **Gestalt — the specification language.** WIT types + intent-prose +
  declarative **snapshots**. Spec-only, with a plugin interface for
  implementation specializations. It is the agent- and human-facing front door:
  an agent reads a gestalt and produces a Wasp implementation; the snapshots
  gate the result.

- **Wasp — the canonical implementation language.** Cranelift-native, with the
  typed store, the validity-as-clarification builder, comptime (in-process JIT),
  reflection, and the REPL. A superset of what gestalts express, so it also
  holds ordinary **imperative unit tests**. Possum and Gestalt are Wasp
  libraries.

- **Possum — a faculty of Wasp, not a separate language.** Because Wasp has
  reflection and procedural AST editing, Possum is the library for defining
  XML-like languages as bundles of *(node kinds + per-kind render + per-kind
  lower-to-target)*. Gestalt, HTML, JS-binding emission, and native-wasm output
  are all Possum languages **differing only in their lowering target** — a
  contract, markup, glue, or bytes. "Wasm-in-html" reactive UIs are just one
  Possum document mixing two languages, wired by the handle graph.

## Load-bearing commitments

- **WIT is both the type system and the validity lattice**, shared across all
  three layers.
- **References are edges (handles), not names** — rename is free, hygiene is
  automatic, scope errors are unrepresentable.
- **One closed set of node kinds; every operation is a per-kind fold** —
  type-check, lower, render a projection, trace, drop.
- **Two dual feedback mechanisms, neither ever an "error":**
  - *Clarification* — validity, pre-commit, in the type lattice (the keystone:
    only valid programs are representable).
  - *Snapshot bless/reject* — correctness, post-eval, in the REPL. The
    behavioral twin of clarification.

## Tests

A **doctest/unit-test split lifted to a layer boundary**:

- **Snapshots** = doctests = the gestalt's public contract; declarative,
  I/O-shaped, travel with the spec. Authoritative once blessed.
- **Imperative tests** = unit tests = the Wasp impl's private mechanism.

The line is **typed by the interface surface**: a check expressible in public
WIT types is snapshot-eligible; one touching internal resources is a unit test.

Snapshots are born either direction — hand-authored spec-first, or
captured-and-blessed impl-first via the REPL — and the derived gestalt becomes a
**behavioral projection** of the Wasp code, useful for cross-language
compatibility and human review, and a regression detector when the impl changes.
**Worlds/capabilities** (kept for composition, not security) are what make
effectful code snapshot-testable: pin the imports, and the output becomes
deterministic and recordable.

A snapshot of a pure function:

```xml
<snapshot>
<input id="name">
Jon
</input>

<output>
Jon! Most excellent sir!
</output>
</snapshot>
```

## Bootstrap

Wasp seeds in Rust against Cranelift, reaches self-hosting, after which Possum,
Gestalt, HTML, and the snapshot machinery are all downstream Wasp libraries. The
seed is small: store + builder + Cranelift lowering + reflection.

## Open questions

1. **Snapshot grammar scope** — pure I/O only (minimal, maximally legible), or
   also pinned-imports and stateful **transcripts** (complete, but grows toward
   a scripting language)?
2. **Snapshot/unit boundary** — enforced by interface types, or soft
   convention?
3. **Comptime trust** — the pure/deterministic subset vs. effectful comptime
   (the one deferred lever).
