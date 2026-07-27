# hace-uni-loader

**SMF ID**: `SMF://hace.uni.loader.v1.1`  
**Intent**: DECLARE_SCHEMA  
**Status**: ACTIVE  
**Layer**: L1 — Host / Reality Plane  
**Role**: Universal Artifact Ingestion & Hacetime Authority Binder  
**WASM Ready**: ✅

7-stage artifact ingestion pipeline with authority boundary enforcement.

---

## Overview

`hace-uni-loader` ingests sealed WASM artifacts and prepares them for execution. It implements a strict 7-stage pipeline:

1. **DISCOVER** → 2. **INGEST** → 3. **VERIFY** → 4. **RESOLVE** → 5. **AUTHORIZE** → 6. **BIND** → 7. **HANDOFF**

### Authority Boundary

The loader **MAY** parse, verify, and resolve artifacts — but it **MAY NOT** grant authority, mint execution permits, bypass Hacetime, or directly execute artifacts.

| May | May Not |
|---|---|
| parse_artifact | grant_authority |
| verify_integrity | mint_execution_permit |
| verify_seal | bypass_hacetime |
| inspect_manifest | bypass_atrust |
| validate_compatibility | directly_execute_artifact |
| request_authority | directly_access_host_resource |
| bind_host_io | |
| prepare_instance | |
| handoff_to_runtime | |

### Key Types

| Type | Description |
|---|---|
| `ArtifactDescriptor` | Artifact identity: id, kind, version, source, format |
| `SealedHeader` | C-struct header with HACE magic, FEH hash, offsets |
| `UniManifest` | Manifest with capabilities and authority binding |
| `ArtifactEnvelope` | Parsed envelope: descriptor + header + manifest + payload + seal |
| `VerifiedArtifact` | Post-verification: integrity, seal, FEH all checked |
| `RuntimeResolution` | Compatibility check result: compatible, capability_gaps |
| `ExecutionPermit` | Authority permit from Hacetime (stage 5 output) |
| `HostBinding` | RAC/PEG binding for host I/O (stage 6 output) |
| `InstanceHandle` | Final instance reference (stage 7 output) |
| `RacEndpoint` | RAC endpoint with auto-detection |

### RAC Endpoint Auto-Detection

| Transport | Protocol | Endpoint |
|---|---|---|
| WebSocket | Racin | `ws://localhost:3018/uni` |
| NamedPipe | Raci | `\\.\pipe\hace-hacetime` |
| Default | Racin | `localhost:3018` |

---

## Usage

### Native (Rust)

```rust
use haha_uni_loader::{HaceUniLoader, ArtifactInput};

// Implement the trait for your host/runtime contexts
struct MyLoader;
impl HaceUniLoader for MyLoader {
    type HostCtx = ();
    type RuntimeCtx = ();
    type Binding = ();
    type InstHandle = ();

    fn discover(input: ArtifactInput) -> Result<ArtifactDescriptor, LoaderError> { /* ... */ }
    fn ingest<'a>(input: ArtifactInput<'a>) -> Result<ArtifactEnvelope<'a>, LoaderError> { /* ... */ }
    fn verify(envelope: &ArtifactEnvelope) -> Result<VerifiedArtifact, LoaderError> { /* ... */ }
    fn resolve(artifact: &VerifiedArtifact, host: &Self::HostCtx, runtime: &Self::RuntimeCtx) -> Result<RuntimeResolution, LoaderError> { /* ... */ }
    fn request_authority(artifact: &VerifiedArtifact, resolution: &RuntimeResolution) -> Result<ExecutionPermit, LoaderError> { /* ... */ }
    fn bind(artifact: &VerifiedArtifact, permit: &ExecutionPermit, host: Self::HostCtx) -> Result<Self::Binding, LoaderError> { /* ... */ }
    fn handoff(binding: Self::Binding, runtime: Self::RuntimeCtx) -> Result<Self::InstHandle, LoaderError> { /* ... */ }
}
```

### WASM (Browser)

```javascript
import { UniLoader, WasmHostContext, WasmRuntimeContext } from '@hacex/hace-uni-loader';

const loader = new UniLoader();

// Stage 1: DISCOVER
loader.discover(wasmBytes);

// Stage 2: INGEST
loader.ingest(wasmBytes);

// Stage 3: VERIFY
loader.verify();

// Stage 4: RESOLVE
const host = WasmHostContext.auto();
const runtime = new WasmRuntimeContext("web", "wasm");
loader.resolve(host, runtime);

// Stage 5: AUTHORIZE (requests from Hacetime)
loader.request_authority(1000); // CESI budget

// Stage 6: BIND
loader.bind(host);

// Stage 7: HANDOFF
const handle = loader.handoff(runtime);
```

---

## Build

```bash
cd engine/hace/uni/loader
cargo build --release
cargo build --target wasm32-unknown-unknown --release
```

---

## Features

| Feature | Default | Description |
|---|---|---|
| `std` | ✅ | Standard library mode |
| `wasm` | ❌ | WASM browser bindings |

---

## Dependencies

- `hace-uni-config` (path dependency)
- `hace-uni-resolver` (path dependency)
- `serde` 1.0
- `serde_json` 1.0
- `blake3` 1.5
- `wasm-bindgen` 0.2 (wasm)
- `js-sys` 0.3 (wasm)
- `serde-wasm-bindgen` 0.6 (wasm)

---

## Canonical References

- **Spec**: `SMF://hace.uni.loader.v1.1` — `.know/canon/specs.ail`
- **Blueprint**: `AIL://hace.uni.canon.blueprint.v1` — `.know/canon/blueprint.ail`
- **Hookpoints**: `hok://uni/loader/*` — `.know/canon/hookpoint.ail`
- **FAN**: 12 features — `.know/canon/fan.ail`
- **ASI**: Integration layer — `.know/canon/asi.ail`

**END OF README**
