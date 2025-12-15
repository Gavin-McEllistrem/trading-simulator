# ADR 002: Name Core Module "engine-core" Instead of "rust-core"

## Status
Accepted

## Date
2025-12-15

## Context

Need to name the main Rust component of the trading system. Initial plan was `rust-core`, but reconsidered during Day 1 setup.

**Options**:
1. `rust-core` - Language-focused naming
2. `engine-core` - Purpose-focused naming
3. `core` - Minimal naming
4. `trading-core` - Very explicit naming

## Decision

Use **`engine-core`** as the name for the main Rust component.

## Rationale

1. **Purpose over Implementation**: Focuses on *what* it does, not *how*
2. **Future-proof**: If we add more Rust components, "rust-core" becomes ambiguous
3. **Professional**: Sounds like a product component, not just "the Rust part"
4. **Consistency**: Pairs well with future additions like `engine-ui`, `engine-api`
5. **Language-agnostic**: External users don't need to know it's Rust

## Consequences

### Positive
- More maintainable naming as project grows
- Better for external documentation
- Clearer purpose for new contributors
- Room for multiple Rust components without confusion

### Negative
- Slightly less explicit about implementation language
- Need to update any existing references (minimal in Day 1)

## Project Structure

```
trading-simulator/
├── engine-core/          # Main Rust engine ← Our choice
├── ocaml-indicators/     # Indicator library (language-explicit, lower level)
├── lua-strategies/       # Strategy scripts (language-explicit, user-facing)
├── tests/
└── docs/
```

## Alternatives Considered

**`rust-core`**: Rejected - too implementation-focused
**`core`**: Rejected - too generic, unclear
**`trading-core`**: Rejected - redundant (already in trading-simulator)

## Implementation

Directory structure:
```bash
mkdir -p engine-core/src
cargo init --name trading-engine
```

Binary/library names remain `trading-engine` for clarity.
