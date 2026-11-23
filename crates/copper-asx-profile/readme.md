# Copper ASX Profile

Implements the Phase I.2 Profile Schema and Multi-User Model for the Copper Linux ASX fork.

## Overview

This crate provides the runtime profile system that governs tool access, escalation policies, and multi-user persona management. Profiles are immutable from page origin and enforce strict access control based on locked constraints from RUNTIME_CONTRACT.md.

## Schema

Each Profile conforms to the locked schema:

```json
{
  "id": "operator-123",
  "name": "Alice Operator",
  "persona": {
    "reasoning": 0.3,
    "caution": 0.4,
    "speed": 0.2,
    "innovation": 0.1
  },
  "tool_surface": ["fetch_web_page", "execute_code", "search_docs"],
  "hop_limit": 3,
  "clearance_required": "moderate",
  "escalation_threshold": "high",
  "allow_guest": true
}
```

### Validation Rules

- Persona weights must sum to exactly 1.0
- Tool surface must not be empty
- Hop limit must be between 1 and 10
- ID and name cannot be empty

## Modules

### profile.rs

Contains the core `Profile` struct and `PersonaWeights`:

- `Profile::new()` - Create a new profile with validation
- `validate_profile()` - Validate an existing profile against schema
- `load_profile()` - Load profile from storage (placeholder)
- `PersonaWeights` - Encapsulates decision-making weights
  - `clarity_score()` - Sum of caution + reasoning
  - `speed_score()` - Standalone speed weight
  - `caution_score()` - Standalone caution weight

### permissions.rs

Tool access control and clearance checking:

- `can_tool_call()` - Check if profile has tool in surface
- `hop_limit_check()` - Verify current hop count under limit
- `clearance_level_check()` - Validate clearance requirements
- `ToolPermission` - Enum: Allow | Deny | Escalate

### guest_model.rs

Read-only guest overlay support:

- `GuestOverlay` - Capability-reduced wrapper around host profile
- `can_escalate_guest()` - Always returns false (guests cannot escalate)
- `validate_guest_mutation()` - Routes guest writes to human review
- `get_host_profile()` - Retrieve parent profile (placeholder)

## Usage

### Creating a Profile

```rust
use copper_asx_profile::{Profile, PersonaWeights};

let weights = PersonaWeights::new(0.3, 0.4, 0.2, 0.1)?;
let profile = Profile::new(
    "operator-123".to_string(),
    "Alice Operator".to_string(),
    weights,
    vec!["fetch_web_page".to_string(), "execute_code".to_string()],
    3,
    "moderate".to_string(),
    "high".to_string(),
    true,
)?;
```

### Checking Tool Access

```rust
use copper_asx_profile::can_tool_call;

let result = can_tool_call(&profile, "fetch_web_page")?;
match result.permission {
    copper_asx_profile::ToolPermission::Allow => { /* allow */ },
    copper_asx_profile::ToolPermission::Deny(_) => { /* reject */ },
    _ => { /* handle escalation */ },
}
```

### Validating Profile

```rust
use copper_asx_profile::validate_profile;

validate_profile(&profile)?;
println!("Profile is valid");
```

## Integration with copper-asx-gates

The profile module integrates with the gate layer to enable selective tool access:

- `GateExecutor::evaluate_with_profile()` - Evaluate tool call against profile constraints
- `MultiUserGateContext` - Multi-profile gate management
- Evidence Ledger tracks profile_id and operator_id for audit

## Design Constraints

As per RUNTIME_CONTRACT.md:

- Profiles live in runtime directory, outside any page origin's DOM/localStorage
- Mutation requires explicit user gesture; otherwise routes to human_review
- Guest profiles are read-only overlays; cannot escalate or write host IR
- Epistemic state and clearance decisions remain immutable from profiles
- Profiles are never modified by the residual model

## Testing

Run tests for this crate:

```bash
cargo test -p copper-asx-profile
```

All 17 tests cover:
- Persona weight validation
- Profile creation and validation
- Permission checks
- Guest overlay constraints
- Hop limit enforcement

## Compatibility

- Rust 2021 edition
- Phase I.2 deliverable
- Compatible with copper-asx-gates for multi-user gate logic
- Supports Phase H Evidence Ledger format with multi-user tracking
