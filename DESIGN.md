# Design Revision — Gestalt / Wasp / Possum

*A language for agents and people. Revised concept — draft, 2026-06-07.*

> **Lineage.** This began as Speckle — AI-implemented specifications expressed as
> `#[speckle]` annotations on Rust items, with the implementation filled in by an
> agent and substituted by a proc-macro. It passed through Possum 0.1 (where the
> source form *was* XML and the program *was* the XML tree) and Possum 0.2 (where
> the program became a value in a typed store and XML became one of several
> projections). This revision keeps the 0.2 substrate but splits the concept into
> a **three-layer tower sharing one type system (WIT)**. The proc-macro
> awkwardness of the original was a symptom of hosting the idea on Rust; the tower
> is what it wanted to be.

> **Naming note.** What the 0.2 specification called *Possum* — the store, the
> builder, comptime, Cranelift lowering — is now **Wasp**, the canonical
> implementation language. *Possum* is repositioned as a **faculty of Wasp**: the
> library for defining XML-like languages. *Gestalt* is the specification
> language, and the first client of that faculty.

> **The keystone.** *Only valid programs are representable.* The author — agent,
> human, or layperson — never holds a broken program and never receives an error
> after the fact. When an edit would produce something invalid, including an
> ill-typed one, the system surfaces a **clarification of intent** *before* the
> edit is committed, and the store moves directly from one valid state to another.
> "Type error" is not a concept the author meets; "what did you mean here?" is.
> This guarantee is scoped to *validity*, not *correctness* (§5).

---

## 1. The three layers

The concept resolves into three things that share one type system and one
mechanism, but serve three different jobs.

- **Gestalt — the specification language.** WIT types + intent-prose +
  declarative **snapshots**. Spec-only, with a plugin interface for
  implementation specializations. It is the agent- and human-facing front door:
  an agent reads a gestalt and produces a Wasp implementation; the snapshots gate
  the result. A gestalt is itself a *Possum language* (§9), so it gets its XML
  face, its type discipline, and its snapshot machinery from the same mechanism
  every other Possum language uses.

- **Wasp — the canonical implementation language.** Cranelift-native, with the
  typed store (§3), the validity-as-clarification builder (§4), comptime
  (in-process JIT, §8), reflection, and the REPL (§9.3 of the substrate). A
  superset of what gestalts express, so it also holds ordinary **imperative unit
  tests**. Possum and Gestalt are Wasp libraries.

- **Possum — a faculty of Wasp.** Because Wasp has reflection and procedural AST
  editing, Possum is the library for defining XML-like languages as bundles of
  *(node kinds + per-kind render + per-kind lower-to-target)*. Gestalt, HTML,
  JavaScript-binding emission, and native-wasm output are all Possum languages
  **differing only in their lowering target** — a contract, markup, glue, or
  bytes. The same per-kind discipline that gives the store its projections (§7)
  gives Possum its languages.

The unifying thread is that **a language is just a set of node kinds plus a
render arm and a lower arm per kind.** Wasp-proper lowers to native code; HTML
lowers to markup; Gestalt lowers to a *contract* (WIT types plus snapshot
obligations) rather than to code. Only the bottom of the fold changes.

---

## 2. Design principles

These are invariants. Every later section is a consequence.

- **The program is a value, not text.** It lives in a typed store of nodes
  addressed by handles. Data-oriented representation is the default, not an
  optimization applied later.
- **Only valid programs are representable.** The store has no encoding for an
  ill-formed, unresolved, or ill-typed program. The construction API is the
  *only* way to mutate the store, and it is total over valid programs (§4).
- **Invalidity is a question, not an error.** When an edit cannot be completed
  validly, the builder returns a set of **clarifications** — concrete, typed
  choices that each lead to a valid program — instead of committing a broken
  state or emitting a diagnostic.
- **Correctness is a separate loop.** The keystone abolishes *representational*
  invalidity, not *behavioral* wrongness, which is undecidable and not in the
  type lattice. Behavioral feedback lives in the REPL and in snapshots (§5).
- **References are edges, not names.** A use-site stores a handle to the binding
  node, not the string of the name. "Reference to an undeclared name" is
  unrepresentable, rename is a no-op, and macro hygiene is automatic (§3.3).
- **One closed set of node kinds; every operation is a fold over it.** Type
  checking, lowering, dropping, tracing, rendering a projection, and clarifying
  an edit are each *one function per kind*, selected by the node's type tag.
  Adding a projection, a language, or a pass means adding an arm, not threading
  new control flow.
- **Types are WIT's types, and types are the validity lattice.** What may connect
  to what — an operand to a slot, a block to a hole, an argument to a parameter,
  a value to a snapshot — is decided by WIT type compatibility (§6).
- **Surfaces are projections.** The store is canonical; XML, Rust-like text, and
  blocks are derived, bidirectional views, and so is a derived gestalt. No
  projection has authority the others lack (§7).

**Identifiers** are lowercase kebab-case (WIT convention): `player`,
`tic-tac-toe`, `arm-count`. Names are display metadata on binding nodes; the
program's structure does not depend on them.

---

## 3. The canonical representation (Wasp's substrate)

### 3.1 The store

The store is a set of **arenas, one per node kind**, each laid out in
structure-of-arrays form. A node is identified by a **handle**: a
`(kind, index, generation)` triple, not a pointer. The `kind` selects the arena
(and the jump-table entry for every per-kind operation); the `index` locates the
node within it; the `generation` validates the handle.

Because handles are indices rather than pointers, the store keeps each arena
densely packed: removing a node swaps the last element into the gap, and the
moved node's handle stays valid because handles resolve through a small stable
indirection slot that the move updates. A stale handle is caught by a generation
mismatch rather than dereferencing freed memory. Stable identity, dense storage,
and O(1) edits coexist — which is what later lets Salsa key incremental work on
node identity (§10) and lets the editor relocate and compact freely.

The kind set is **closed**. This is what makes dispatch a jump table indexed by
`kind`, and it is what makes "add a projection" or "add a Possum language" mean
"add a render/lower arm per kind."

### 3.2 What a node holds

A node holds typed slots, each either a **named operand** (one child playing a
role — `lhs`, `count`, `value`, `self`), a **homogeneous sequence** (list items,
match arms, variant cases, statement bodies), or a **scalar payload** (a
literal's value). Operands and sequence members are handles; the store is a graph
of handles, not a string tree. Every node carries the handle of its **result
type** (itself a node in a type arena), so any node's type is available without
inference at read time.

### 3.3 References are edges

A `get`, a branch target, a `use` of an imported type, a method's `self` — none
store the *name* they mention; each stores a **handle to the binding** it
resolves to. Three properties follow with no extra machinery:

- **Scope validity is intrinsic.** You cannot construct a reference to a binding
  that does not exist; there is nothing to point the edge at.
- **Rename is free and total.** Changing a binding's display name changes one
  field; every reference already points at the binding, not its spelling.
- **Macro hygiene is free.** A binding a macro introduces has its own handle
  identity and cannot collide with a name elsewhere (§8).

The `key="…"` strings in the XML projection and the identifiers in the Rust-like
projection are *renderings* of these edges, resolved per projection at display
time. (This has a consequence for serialization — see §11.)

---

## 4. Validity as intent clarification

The only way to change the store is to submit an **edit intent** to the builder.
The builder is a **total function**:

```
build(valid_store, intent) -> Committed(valid_store') | Clarify(choices)
```

It never returns an invalid store and never returns an error. Either the intent
determines exactly one valid result, which is committed, or it is underdetermined
or conflicting, in which case the builder returns a set of **choices**, each a
concrete intent that *would* commit to a valid store. The author picks one (or
refines), and the loop repeats. An invalid program is never instantiated, not
even briefly.

**Types are clarified, not enforced.** A type mismatch is a fork, not an error:

- A `u8`-valued node in a slot that expects `s64` yields choices such as *insert
  an `s64.from` conversion around the value*, *change the slot to `u8`*, or
  *change the binding's type to `s64`* — each a valid program.
- A `player` where a `cell` (`option<player>`) is expected yields *wrap in
  `option.some`* as the leading choice.
- An omitted required record field yields *supply field `x`* with a typed hole to
  fill, never a "missing field" error.

There is exactly one way for an author to experience types: as the system
proposing the small, explicit step that makes their intent fit. Conversions are
always explicit nodes in the store (§6.5) and always offered rather than assumed.

**The protocol is uniform across surfaces and macros.** The same `build` backs
every projection. An agent's structural replace, a human's keystroke that
reparses a region, and a block dragged toward a hole all become edit intents and
get back a commit or the same choices, rendered in the surface's vocabulary.
Macros build through `build` too, so a macro cannot emit an invalid program; if a
macro's construction is underdetermined, the clarification surfaces at the
**call site** (§8.3).

---

## 5. The validity/correctness boundary

The keystone guarantees you never hold a program that *won't build*. It does not
and cannot guarantee you never hold a program that is *wrong* — behavioral
correctness is undecidable and outside the type lattice. A perfectly valid,
fully-typed program can compute the wrong thing. So the system has **two dual
feedback mechanisms, and neither is ever an "error":**

| | Validity | Correctness |
|---|---|---|
| **Object** | clarification | snapshot diff |
| **When** | pre-commit | post-eval |
| **Where** | the type lattice (§4) | the REPL / comptime run (§8) |
| **Author sees** | "which valid completion did you mean?" | "behavior changed — accept or reject?" |

"Accept the REPL output and proceed, or reject and retry" is the behavioral twin
of clarification — the same *show-a-fork-not-an-error* philosophy, one layer
down. Snapshots are how that behavioral loop is **persisted and
regression-checked** instead of evaporating after each REPL poke (§10).

There is also a quiet third state: a program with an unfilled **typed hole**
(§4) is *valid and representable* but *not runnable*. So the agent's loop needs a
"complete enough to `eval`" predicate distinct from "valid" — it may probe only
once the holes on the path to its query are filled.

---

## 6. Type system

Possum's types are WIT's types. They additionally serve as the lattice the
builder consults in §4 and the connector rules the block editor enforces in §7.4.

- **6.1 Primitives.** `bool`, `char`, `string`; signed `s8 s16 s32 s64`; unsigned
  `u8 u16 u32 u64`; floats `f32 f64`.
- **6.2 Built-in generics.** `list<t>`, `option<t>`, `result<t, e>` (either
  parameter omittable, e.g. `result<_, string>`), `tuple<a, b, …>`.
- **6.3 User-defined types.** **Record** (product of named fields), **variant**
  (sum with optional per-case payload), **enum** (payload-free sum, kept distinct
  to match WIT), **flags** (set of named booleans), **alias** (a name for a type
  expression). Each owns a namespace of constructors and operations under its
  dotted name (`octopus.new`, `outcome.won`, `player.x`).
- **6.4 Resources.** A handle to a stateful entity, with an optional constructor,
  methods (receiving the handle as a `self` operand), and statics. Handles pass
  as `own<r>` or `borrow<r>`. Resource lifetime is governed by ownership,
  independent of the runtime collector (§10): an `own<r>` is released
  deterministically when consumed or out of scope — how non-memory resources
  (files, sockets, host objects) get prompt teardown.
- **6.5 Conversions.** No implicit coercions. Every numeric or representational
  conversion is an explicit per-type node (`s64.from`, …). The author rarely
  *writes* one; §4 offers it as the resolution when a slot and a value disagree.

---

## 7. Projections

### 7.1 The mechanism

A projection is a pair of per-kind functions, dispatched on the node's `kind`:

- **render** (store → surface): each kind knows how to display itself. Adding a
  projection is adding one `render` arm per kind.
- **edit** (surface → store): a surface action becomes an edit intent submitted
  to `build` (§4). Validity, including types, is resolved by clarification at
  this boundary, in the surface's vocabulary.

No projection is canonical and none can express anything the store cannot hold.
This is the *same* mechanism Possum uses to define languages (§9); a projection
is a Possum language whose lowering target is "back into this store."

### 7.2 XML projection — for agents

Explicit, structural, addressable: element kind = node kind, `key`/`type` =
naming and type metadata, `<field key="role">` = a named operand, repeated
`<item>`/`<arm>`/`<case>` = sequence members, text content = a scalar literal.

```xml
<func key="add">
  <input  key="a"   type="s64"/>
  <input  key="b"   type="s64"/>
  <output key="sum" type="s64"/>
  <s64.add key="sum">
    <field key="lhs"><get key="a"/></field>
    <field key="rhs"><get key="b"/></field>
  </s64.add>
</func>
```

The agent's structural edit API (query/replace over the tree, addressed by dotted
keys) is the XML face of §4: a replace either commits or returns clarification
objects naming the fork and its typed choices. The agent's loop is therefore
*write → commit-or-clarify → choose*; it never parses an error or repairs a
broken tree.

### 7.3 Rust-like projection — for humans

A terse surface with sugar that desugars to the same nodes: infix operators,
`let`, method-call syntax, literal shorthands, and real pattern matching.

```rust
fn add(a: s64, b: s64) -> s64 {
    a + b
}

fn play(b: board, at: u8, who: player) -> board {
    b.set(at, some(who))            // list.set(list: b, index: at, value: some(who))
}
```

`+` is `s64.add`, `b.set(…)` is `list.set`, `some(…)` is the namespaced
constructor with the namespace elided where the expected type makes it
unambiguous. All sugar is projection-level; the store holds the explicit nodes.
An edit that does not fit (e.g. `a + b` with `a: u8`, `b: s64`) raises the §4
clarification inline rather than underlining an error.

### 7.4 Block projection — for non-programmers

Each node kind renders as a **block**; each named operand and sequence slot
renders as a **typed hole**. WIT types become connector shapes and colors: a hole
expecting `s64` accepts only blocks whose result type is `s64`-compatible — so
**a block only snaps where it would type-check**, and a layperson cannot assemble
an ill-typed program at all. When a block is dragged toward a hole it does not
fit, the §4 clarification appears as the snap affordance itself (drop a `player`
block on a `cell` hole, get *wrap in `some`?*). There are no syntax or type
errors here; only blocks that fit and gentle questions where they could be made
to.

---

## 8. Comptime and macros

- **8.1 Comptime is JIT evaluation.** A comptime computation is an ordinary Wasp
  computation evaluated *during compilation* by JIT-compiling it with Cranelift
  and running it in-process against the live store; its result is folded back as
  constant nodes. This is the reason for targeting Cranelift rather than
  WebAssembly: a native target lets the compiler compile and run fragments
  interleaved with compilation, sharing its own data structures with no
  marshalling boundary.
- **8.2 Macros are functions over node handles.** At a call site the compiler
  gathers argument handles, JIT-compiles the macro (memoized by Salsa, §10), runs
  it, and splices the returned handles into the store.
- **8.3 Macros cannot produce invalid programs.** Because a macro builds its
  result *through `build`* (§4), every node it creates is validity- and
  type-checked as created. If its construction is underdetermined, the
  clarification surfaces at the **call site**, attributed to the macro. Combined
  with edges-not-names (§3.3), macros are hygienic and validity-preserving by
  construction.
- **8.4 Comptime purity (the lever).** Comptime is restricted to a pure,
  deterministic subset — no host imports, no ambient effects — so a build is
  reproducible and a macro is referentially transparent. The alternative
  (effectful comptime) is a one-section change, deferred (§12).

---

## 9. The Possum faculty in depth

Possum makes "a language" a first-class library value: a set of node kinds with a
`render` arm and a `lower` arm per kind, reusing Wasp's store, builder, and
reflection. The consequences:

- **Every Possum language gets the keystone for free.** Because all construction
  goes through `build` (§4), a Possum language cannot represent an invalid
  document of itself any more than Wasp can.
- **The lowering target is the only thing that varies.** Wasp-proper lowers to
  CLIF → native. HTML lowers to markup (and its `lower` for a text node holding a
  Wasp value-handle drives a DOM update — fine-grained reactivity *is* the handle
  graph). A wasm language lowers to bytes. A bindings language lowers to glue.
  **Gestalt lowers to a contract** rather than to code (§1, §5).
- **Multiple languages compose in one document.** "Wasm-in-html" is one Possum
  document mixing HTML nodes and Wasp computation nodes, wired by the edges of
  §3.3: an HTML node holds a handle to a Wasp value node; when the value
  recomputes, the edge drives the update. No extra mechanism — the reactivity is
  the store.
- **Gestalt is the proof-of-concept.** It is not hardcoded; it is the first and
  canonical Possum language, which is what demonstrates the faculty works.

---

## 10. Snapshots and testing

Testing splits along the layer boundary, by analogy to doctests vs. unit tests:

- **Snapshots = doctests = the gestalt's public contract.** Declarative,
  I/O-shaped, they *define* the behavior and travel with the spec. Authoritative
  once blessed.
- **Imperative tests = unit tests = the Wasp impl's private mechanism.** Caches,
  buffers, internal resources. They stay in Wasp, the superset.

**The line is typed, not conventional.** A behavioral check expressible purely in
terms of a public WIT interface type is snapshot-eligible; one that must mention
an internal resource or helper is a unit test. The interface surface (§6, the
`world`) is what decides — so `greet("")` (public `string -> string`) wants to be
a snapshot, while "the memo cache evicts after N" is a Wasp unit test.

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

**Snapshots are a fourth projection — a behavioral one.** XML / Rust-like /
blocks render the store's *shape*; a derived gestalt renders the impl's
*behavior*. Both are derived; both have a fork-resolution loop instead of an
error — structural edits go through *build → commit-or-clarify*, behavioral
captures through *eval → bless-or-reject* (§5).

**Production is symmetric.** A snapshot is born either direction, and the REPL is
the shared pivot of both:

- *Spec-first* — author snapshots, an agent writes Wasp to satisfy them, the
  snapshots gate the result (the original Speckle loop).
- *Impl-first* — write Wasp, author snapshot inputs, see the output in the REPL,
  accept and proceed or reject and retry. The blessed gestalt falls out as a
  **useful, independent, derived artifact** for cross-language compatibility or
  human review.

Either way the snapshot *is* the contract once blessed, and a later impl change
that diffs a blessed snapshot is a **regression surfaced as a diff to review**,
not an error — the doctest property, with no separate machinery.

**The oracle, and why the contract is readable-but-incomplete.** With a canonical
Wasp impl *plus* plugin specializations there are again multiple implementations
of one gestalt, so conformance has two layers. Snapshots gate every
implementation — legible, finite, human-reviewable I/O — but finite: a plugin can
pass every snapshot and still diverge off-sample. The canonical Wasp impl is the
**oracle** that fills the gap (differentially test a plugin against it on
arbitrary inputs; record each disagreement as a new snapshot). The contract is
deliberately readable-but-incomplete, backstopped by the oracle.

**Capabilities make effectful snapshots possible.** Pure functions snapshot as
bare I/O. Effectful or stateful behavior does not — and the `world`/capability
mechanism (§11.5 of the substrate; kept for composition, not security) is exactly
what rescues it: *pin the imports* to fixed values and an otherwise
non-deterministic function becomes deterministic and recordable. This exposes a
grammar question (§12): snapshots come in at least three shapes —

1. **pure** — `<input>` → `<output>`;
2. **effectful** — a pinned-imports clause plus I/O;
3. **stateful** — an interaction *transcript* of method calls on a handle and
   their successive outputs.

---

## 11. Compilation, execution, and serialization

- **11.1 Pipeline.** Lowering walks the store and emits **CLIF**: each function
  becomes a CLIF function in SSA form; bound `key`s become SSA values; `get`
  becomes the read; structured `block`/`loop`/`if` and keyed branches become CLIF
  blocks and jumps; `match` becomes a switch on the variant discriminant.
  Cranelift lowers CLIF to native code. One backend, one engine — the same
  Cranelift JIT serves comptime, the REPL, and final codegen.
- **11.2 Incrementality with Salsa.** The compiler is a graph of Salsa queries
  (`resolve`, `type_of`, `lower`, `jit`, `eval`) keyed on node handles. Because
  handles are stable identities (§3.1), an edit invalidates only the queries that
  read the changed node; identity comparison is O(1) and there is no tree-diffing.
  The data-oriented store is precisely what makes incrementality granular.
- **11.3 On-demand REPL.** "Evaluate this expression" is `eval(e)` depending on
  `jit(deps(e))`. Comptime, the REPL, and incremental rebuild are **one**
  demand-driven JIT-eval mechanism under three sets of roots: the program
  (comptime), an agent's expression (REPL), a changed input (rebuild). This is
  the agent's fast probe-and-observe loop over a live program (§5).
- **11.4 Runtime memory.** Values use the data-oriented representation by default.
  The collector is a **stop-the-world mark-sweep that slides into mark-compact**,
  because the architecture pre-pays its two costs: precise tracing is the per-kind
  `trace` arm (same shape as `drop` and `render`), and relocation is cheap
  (compaction updates the indirection slot, not every reference). Resources are
  handled by `own`/`borrow` (§6.4); the collector owns only the plain value graph.
- **11.5 Capabilities and composition (no sandbox).** A `world` declares the
  interfaces a unit imports and exports. This is a **composition and clarity**
  mechanism — it makes dependencies explicit, lets capabilities be swapped or
  mocked (which is what powers effectful snapshots, §10) — but it is **not** an
  isolation boundary. The tower runs native and makes no sandboxing claim.
- **11.6 Serialization and version control.** The store is canonical and
  in-memory, but a durable, diffable form is needed for VCS — and the XML
  projection is the natural candidate. Two consequences follow:
  - Because `key="…"` *renders an edge* (§3.3), the durable form must encode the
    edge's **identity**, not only the binding's current display name — otherwise
    a round-trip through text cannot distinguish "same edge, renamed binding"
    from "edge re-pointed elsewhere," and diffs/merges lose the graph. The agent
    surface shows `key="player"`; the durable form also carries a stable
    identity token.
  - A merge can produce text that corresponds to no valid store. Since the *only*
    way into the store is `build` (§4), importing merged XML goes through the
    validity protocol — so a conflicting merge surfaces as **clarification
    choices, never a committed broken state.** Even VCS conflicts cannot
    instantiate an invalid program.

---

## 12. Status and open questions

**Settled.**

- The store-first canonical representation (§3) and handles as stable identity.
- Validity-as-clarification construction, including types (§4); the
  validity/correctness boundary and its two dual feedback loops (§5).
- References as resolved edges (§3.3), with rename and hygiene consequences.
- WIT as the type system and validity lattice (§6).
- The per-kind render/edit/lower mechanism; the three projections (§7); Possum as
  the faculty that generalizes it into languages by lowering target (§9).
- The three-layer tower: Gestalt (spec), Wasp (canonical impl), Possum (faculty);
  WIT as shared connective tissue (§1).
- Declarative snapshots as the gestalt's contract; the doctest/unit split typed
  by the interface surface; bidirectional production; the oracle (§10).
- Cranelift-only native compilation, Salsa incrementality, the unified
  comptime/REPL/rebuild mechanism, the tracing-collector default, capabilities as
  composition with no sandbox (§11).

**Open levers and deferred work.**

1. **Snapshot grammar scope.** Pure I/O only (minimal, maximally legible), or
   also pinned-imports and stateful **transcripts** (a complete behavioral
   contract, but the grammar grows toward a scripting language)?
2. **Snapshot/unit boundary enforcement.** Enforced by interface types (a
   checkable invariant that keeps gestalts from leaking implementation detail) or
   a soft convention?
3. **Possum as language vs. faculty.** Keep the three-language framing for
   identity, or present it as "one language (Wasp) + one metaprogramming library
   (Possum) + one DSL built with it (Gestalt)"?
4. **Comptime trust.** The pure/deterministic subset (§8.4) vs. effectful
   comptime — reproducibility traded for power.
5. **Clarification ranking.** When `build` returns several choices (§4), the
   order and defaulting that make the common case a single tap.
6. **Pattern richness in `match`.** Literal and wildcard arms are defined;
   nested/destructuring patterns and their renderings are a candidate extension.
7. **Multi-value loop carry.** Values threaded through `loop` iterations, now
   lowered to CLIF block parameters, to be pinned down with examples.
