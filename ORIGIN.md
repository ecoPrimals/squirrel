<!-- SPDX-License-Identifier: CC-BY-SA-4.0 -->
# Origin

**Squirrel** is the AI Coordination Primal of the [ecoPrimals](https://github.com/ecoPrimals) ecosystem.

---

## Genesis

Squirrel began in mid-2025 as a fault-tolerant compute orchestration platform
(gen1: "AI Swarm") — checkpoint/restart, circuit breakers, and ant-model
orchestration across heterogeneous GPU nodes. The initial insight came from
Geoffrey Huntley's [stdlib thesis](https://ghuntley.com/stdlib/): treat Cursor
not as an IDE but as an autonomous agent, and program LLM outcomes rather than
prompt for them.

That insight became the seed for a formal methodology — **constrained
evolution** — developed during the author's Master of Science in Data Science
(MSDS, 2025). The theoretical framework maps evolutionary biology onto
AI-assisted software generation:

| Biological Component | Computational Analog |
|----------------------|----------------------|
| Mutation (DNA replication errors) | AI agent code generation (LLM token sampling) |
| Environmental constraint (thermal, pH) | Rust's type system + borrow checker |
| Natural selection (differential survival) | Compiler rejection of unfit variants |
| Fitness function (reproductive success) | Physics validation suites (do the results reproduce?) |

The framework is grounded in three biological lines of evidence:

- **Taq polymerase** (*Thermus aquaticus*): extreme thermal constraint produced
  the enzyme that enabled PCR. The constraint defines the fitness landscape.
- **Lenski's LTEE** (80,000+ generations of *E. coli*): identical constraints
  on twelve populations produced twelve different solutions, all increasing
  fitness. Constraint drives specialization, not a single answer.
- **Anderson's boundary** (deep-sea subsurface): when population diversity is
  too low for selection to dominate drift, Muller's ratchet degrades quality.
  This defines when constrained evolution fails.

The formal treatment is in `gen3/thesis/03_theoretical_framework.md`
in the ecoPrimals white paper (see the [ecoPrimals](https://github.com/ecoPrimals/ecoPrimals) repository).

---

## From gen1 to gen3

| Generation | Period | Squirrel's Role |
|------------|--------|-----------------|
| gen1 | mid-2025 | HPC job scheduler — GPU workload distribution, checkpoint/restart |
| gen2 | late 2025 | AI orchestrator — multi-provider routing, MCP integration |
| gen3 | 2026 | Sovereign AI Coordination Primal — TRUE PRIMAL, capability-based discovery, zero vendor coupling |

The gen1 checkpoint/restart logic evolved into context management and
conversation state persistence. The circuit breakers evolved into provider
fallback chains. The ant-model orchestration became multi-MCP coordination.
Same codebase, reshaped by changing selective pressure.

---

## Methodology

One developer. 69,000+ Cursor agent invocations. 51 billion tokens consumed.
185 consecutive day streak.

The methodology:

1. **Specification first** — write the constraint (spec, types, traits) before
   generating code.
2. **AI as mutation operator** — the LLM proposes variants; it does not decide
   what ships.
3. **Rust as natural selection** — `#![forbid(unsafe_code)]`, the borrow
   checker, and the type system reject unfit variants at compile time.
4. **Physics as fitness function** — for the science primals (hotSpring,
   wetSpring, etc.), validation suites reproduce published results. For
  infrastructure primals like Squirrel, the fitness function is the test suite
  (5,775 tests), chaos/fault injection, and ecosystem integration.

The Cursor receipt is the evidence for the methodology: the commit history and
agent invocation log show the evolutionary trajectory.

---

## The ecoPrimals Ecosystem

Squirrel is one of 14 primals. The full catalog is in the
ecoPrimals white paper (`PRIMAL_CATALOG.md`). Key relationships:

- **biomeOS** routes all AI requests through Squirrel
- **Songbird** provides HTTP transport for cloud AI providers
- **NestGate** caches model weights
- **BearDog** encrypts and manages API keys
- **ToadStool** provides GPU compute for local inference

The neuralSpring science validation suite is the use-case counterpart to
Squirrel's infrastructure — it validates the neuromorphic and surrogate-learning
workloads that Squirrel coordinates.

---

## License

[scyBorg](LICENSE) — AGPL-3.0-only + ORC + CC-BY-SA 4.0.

Copyright (C) 2026 ecoPrimals Contributors
