# Contributing to OPURE OS

Thank you for your interest in contributing to **OPURE OS**, the high-performance, agent-first operating system cognitive substrate.

## Core Architectural Invariants

All contributions must strictly adhere to the following non-negotiable systems constraints:

1. **Zero-Allocation Execution (`0 Bytes` Heap Allocation):**
   - No `Vec`, `Box`, `String`, or dynamic heap-allocated collections in runtime fast paths.
   - Use `Aligned64`, `StaticSlab`, fixed-capacity ring buffers (`EventRingBuffer`), and stack buffers.
   - Any PR introducing dynamic heap allocations or hidden allocations will be rejected.

2. **Strict Compiler Invariant (`#![deny(warnings)]`):**
   - Every crate in the workspace compiles under `#![deny(warnings)]`.
   - Code must build cleanly across both host targets and the bare-metal UEFI target (`x86_64-unknown-uefi`).

3. **Sub-Microsecond Latency Standards:**
   - Cognitive NCM routing must remain under 50 ns per pass.
   - OZTAE capability firewall checks must remain under 5 ns.
   - Machine-code trampoline emissions must remain under 5 ns.

4. **Cryptographic & Hardware Telemetry:**
   - Any new security event or system transition must commit a 64-byte `OztaeAuditFrame` to the Merkle-chained audit ledger.
   - Unit tests must be provided for all new mathematical operators, SIMD routines, and hardware state transitions.

## Testing & Verification Workflow

Before submitting a pull request, run the workspace test and benchmark suite:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\init_workspace.ps1 -Release
```

Ensure all 48+ unit tests pass and benchmarks confirm zero heap allocation.
