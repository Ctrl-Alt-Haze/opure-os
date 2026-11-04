# Security Policy

## Reporting Security Vulnerabilities

The OPURE OS architecture enforces hardware-level zero-trust boundaries via the **Opure Zero-Trust Agent Enclave (OZTAE)**, single-cycle capability bitmasks, 24-bit Morton spatial fencing, and tamper-evident Merkle-chained audit ledgers.

If you discover a security vulnerability, side-channel leak, or sandbox escape:

1. **Do NOT open a public GitHub issue.**
2. Email full technical reproduction details to: **security@opure.org**
3. Include:
   - Target crate(s) affected (e.g. `opure-oztae`, `opure-lifter`, `opure-boot`).
   - Detailed exploit or reproduction code demonstrating an unauthorized memory access, capability spoofing, or Merkle ledger tampering.
   - Host hardware, CPU architecture, and execution flags.

## Security Guarantees & Design Invariants

- **Single-Cycle Capability Gating:** Agent capability bitmasks are validated in 1 CPU cycle and can be atomically revoked in 1 cycle (`cap_rights.store(0)`).
- **Cryptographic Audit Parity:** All access checks and security state transitions are committed into a rolling XOR-fold / SipHash Merkle-chained ring ledger (`OztaeAuditLedger`). Any tampering with past frames immediately invalidates the cryptographic integrity chain.
- **Ternary Memory Isolation:** Unquantized or rogue agent trajectory drift is caught via Mandelbrot escape-time gates and hardware POPCNT distance thresholds.
