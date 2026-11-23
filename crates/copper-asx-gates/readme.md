# copper-asx-gates

MCP Gate Layer for Copper ASX Fork — Phase I implementation of the Retrieval Contract runtime gates.

## What it does

`copper-asx-gates` enforces access control boundaries for MCP tool calls in Copper for Linux, preventing unauthorized or incomplete evidence retrieval. This is **Phase 1 of 5** in the Copper ASX fork roadmap.

### Phase 1: MCP Gate Layer (current)

- **GateExecutor**: Checks clearance + open gaps before tool execution
- **EvidenceRecorder**: SHA256-chain ledger of every gate decision (allow/reject/blocked)
- **EvidenceLedger**: JSONL audit trail with cryptographic integrity verification
- **Z3Stub**: Mocked constraint solver (placeholder for Phase 3)

**Default Phase 1 behavior:** All tool calls are blocked (clearance always false). Every decision is logged to the Evidence Ledger with an unbroken SHA256 hash chain.

## Architecture

```
MCP Tool Call
     ↓
GateExecutor (check clearance + gaps)
     ↓
✓ Allowed / ✗ Blocked / ✗ Rejected
     ↓
EvidenceRecorder (log to chain)
     ↓
EvidenceLedger (append JSONL + verify hash)
```

## Usage

### Basic gating

```rust
use copper_asx_gates::{Clearance, GapSet, GateExecutor};

let clearance = Clearance::denied("phase_i_disabled");
let gaps = GapSet::new();
let executor = GateExecutor::new(clearance, gaps);

let (decision, reason) = executor.get_decision_for_tool("fetch_web");
// decision = GateDecision::Blocked
// reason = "clearance=phase_i_disabled"
```

### Evidence recording

```rust
use copper_asx_gates::EvidenceRecorder;
use copper_asx_gates::GateDecision;

let mut recorder = EvidenceRecorder::new(".copper/evidence_ledger.jsonl")?;
let packet_id = recorder.record_gate_decision(
    GateDecision::Blocked,
    "fetch_web",
    "clearance=false"
)?;
// packet_id = "pkt-000001"

let valid = recorder.verify_ledger_integrity()?;
// Checks entire SHA256 chain
```

## Evidence Ledger Format

JSONL file (`.copper/evidence_ledger.jsonl`) with entries:

```json
{
  "timestamp": "2026-11-22T17:36:06.705Z",
  "event_type": "gate_decision",
  "tool_name": "fetch_web",
  "decision": "blocked",
  "reason": "clearance=phase_i_clearance_disabled",
  "evidence_hash": "sha256:...(64 hex chars)...",
  "prev_hash": "sha256:...(64 hex chars)...",
  "packet_id": "pkt-000001"
}
```

**Invariants:**
- Every `evidence_hash` = SHA256(timestamp|tool_name|decision|reason|prev_hash|packet_id)
- Each entry's `prev_hash` = previous entry's `evidence_hash`
- Chain starts with `prev_hash = "0000...0000"` (genesis block)
- Ledger is append-only; no rewrites

## Roadmap

| Phase | Timeline | Deliverable |
|-------|----------|-------------|
| I.1 | Current | MCP gate layer, evidence ledger, Phase 1 blocking |
| I.2 | +2-3 weeks | Profile schema, multi-user model, operator profiles |
| I.3 | +2-3 weeks | Real Z3 integration, constraint solving |
| II–V | +1-2 months | Advanced evidence analysis, RUNTIME_CONTRACT enforcement, operator dashboard |

## Testing

```bash
cargo test -p copper-asx-gates
```

All 23 tests pass, covering:
- Clearance logic (allowed/denied)
- Gap set operations
- Evidence recording and chain verification
- Executor state mutations
- Z3 stub behavior

## Integration with copper-mcp

The MCP server (`crates/copper-mcp`) will integrate gates like this:

```rust
let mut executor = GateExecutor::new_with_phase_one_config();
let mut recorder = EvidenceRecorder::new(".copper/evidence_ledger.jsonl")?;

// Before executing any tool call:
let (decision, reason) = executor.get_decision_for_tool(tool_name);

if decision != GateDecision::Allow {
    recorder.record_gate_decision(decision, tool_name, &reason)?;
    return Err(format!("Gate: {}", reason));
}

recorder.record_gate_decision(GateDecision::Allow, tool_name, "gate_permit")?;
// proceed with tool execution
```

## References

- **RUNTIME_CONTRACT.md**: Frozen specification of gate invariants (escalation boundary, C1–C7)
- **FORMAL_METHODS_TLA_PLUS.md**: RetrievalContract model; gates implement its safety property
- **Phase H audit**: Evidence Ledger structure locked; 6/6 scenarios rejected in H.4

## License

MIT (same as Copper)
