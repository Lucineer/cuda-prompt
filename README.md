# cuda-prompt

Prompt primitives — templates, chain-of-thought, few-shot management, versioning, compression (Rust)

Part of the Cocapn fleet — a Lucineer vessel component.

## What It Does

### Key Types

- `PromptTemplate` — core data structure
- `CotStep` — core data structure
- `CotScaffold` — core data structure
- `FewShotExample` — core data structure
- `PromptVersion` — core data structure
- `PromptManager` — core data structure

## Quick Start

```bash
# Clone
git clone https://github.com/Lucineer/cuda-prompt.git
cd cuda-prompt

# Build
cargo build

# Run tests
cargo test
```

## Usage

```rust
use cuda_prompt::*;

// See src/lib.rs for full API
// 10 unit tests included
```

### Available Implementations

- `PromptTemplate` — see source for methods
- `CotScaffold` — see source for methods
- `PromptVersion` — see source for methods
- `PromptManager` — see source for methods

## Testing

```bash
cargo test
```

10 unit tests covering core functionality.

## Architecture

This crate is part of the **Cocapn Fleet** — a git-native multi-agent ecosystem.

- **Category**: other
- **Language**: Rust
- **Dependencies**: See `Cargo.toml`
- **Status**: Active development

## Related Crates


## Fleet Position

```
Casey (Captain)
├── JetsonClaw1 (Lucineer realm — hardware, low-level systems, fleet infrastructure)
├── Oracle1 (SuperInstance — lighthouse, architecture, consensus)
└── Babel (SuperInstance — multilingual scout)
```

## Contributing

This is a fleet vessel component. Fork it, improve it, push a bottle to `message-in-a-bottle/for-jetsonclaw1/`.

## License

MIT

---

*Built by JetsonClaw1 — part of the Cocapn fleet*
*See [cocapn-fleet-readme](https://github.com/Lucineer/cocapn-fleet-readme) for the full fleet roadmap*
