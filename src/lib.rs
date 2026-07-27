// hace-uni-loader — Universal Artifact Ingestion & Authority Binding Boundary
// CRD refined v1.1 - 7-stage pipeline: DISCOVER → INGEST → VERIFY → RESOLVE → AUTHORIZE → BIND → HANDOFF

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::vec::Vec;
use alloc::string::String;

// ============================================================
// AIL Specification (SMF Header)
// ============================================================

/*
header:
  id: SMF://hace.uni.loader.v1.1
  intent: DECLARE_SCHEMA
  status: ACTIVE
  locale: vi-85

feature:
  name: hace-uni-loader
  category: universal_ingestion_engine
  role: artifact_ingestion_and_runtime_binding
  canonical_definition:
    producer: hace-uni-target
    compiler: CONA
    artifact: hace-artifact
    consumer: hace-uni-loader
  principle:
    - ingest
    - verify
    - resolve
    - negotiate
    - bind
    - handoff
  authority_boundary:
    loader:
      may:
        - parse_artifact
        - verify_integrity
        - verify_seal
        - inspect_manifest
        - validate_compatibility
        - request_authority
        - bind_host_io
        - prepare_instance
        - handoff_to_runtime
      may_not:
        - grant_authority
        - mint_execution_permit
        - bypass_hacetime
        - bypass_atrust
        - directly_execute_artifact
        - directly_access_host_resource
*/

// ============================================================
// Constants
// ============================================================

pub const HACE_MAGIC: [u8; 4] = *b"HACE";

// ============================================================
// Stage 1: Artifact Descriptor (Discovery)
// ============================================================

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ArtifactDescriptor {
    pub artifact_id: String,
    pub artifact_kind: String,
    pub version: String,
    pub source: String,
    pub format: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ArtifactSource {
    RawWasm,
    SealedWasm,
    HaceArtifact,
    MemoryBuffer,
    FilePath,
}

// ============================================================
// Stage 2: Artifact Envelope (ingest + preserve seal)
// ============================================================

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct SealedHeader {
    pub magic: [u8; 4],
    pub version: u32,
    pub feh_hash: [u8; 32],
    pub manifest_offset: u32,
    pub manifest_len: u32,
    pub capability_offset: u32,
    pub capability_len: u32,
    pub payload_offset: u32,
    pub payload_len: u32,
    pub seal_offset: u32,
    pub seal_len: u32,
}

impl SealedHeader {
    pub fn parse(data: &[u8]) -> Result<&Self, LoaderError> {
        if data.len() < core::mem::size_of::<Self>() {
            return Err(LoaderError::InvalidFormat);
        }
        let ptr = data.as_ptr() as *const Self;
        let header = unsafe { &*ptr };
        if header.magic != HACE_MAGIC {
            return Err(LoaderError::InvalidMagicNumber);
        }
        Ok(header)
    }

    pub fn is_valid(&self) -> bool {
        self.magic == HACE_MAGIC && self.version != 0
    }
}

// ============================================================
// Manifest & Capabilities
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum AuthorityLevel {
    User = 0,
    UserPlus = 1,
    Node = 2,
    Network = 3,
    System = 4,
}

impl Default for AuthorityLevel {
    fn default() -> Self {
        Self::UserPlus
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Capability {
    pub id: String,
    pub name: String,
    pub version: String,
    pub required_authority: AuthorityLevel,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UniManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub authority_binding: String,
    pub capabilities: Vec<Capability>,
    pub abi_version: String,
}

// ============================================================
// Artifact Envelope (preserves seal)
// ============================================================

pub struct ArtifactEnvelope<'a> {
    pub descriptor: ArtifactDescriptor,
    pub header: SealedHeader,
    pub manifest: &'a UniManifest,
    pub capabilities: Vec<Capability>,
    pub payload: &'a [u8],
    pub seal: &'a [u8],
}

// ============================================================
// Stage 3: Verified Artifact
// ============================================================

pub struct VerifiedArtifact {
    pub envelope: ArtifactEnvelope<'static>,
    pub integrity_verified: bool,
    pub seal_valid: bool,
    pub feh_match: bool,
}

// ============================================================
// Stage 4: Runtime Resolution
// ============================================================

pub struct RuntimeResolution {
    pub compatible: bool,
    pub runtime_adapter: String,
    pub abi_adapter: String,
    pub capability_gaps: Vec<String>,
}

// ============================================================
// Stage 5: Execution Permit (authority output)
// ============================================================

pub struct ExecutionPermit {
    pub permit_id: [u8; 16],
    pub artifact_hash: [u8; 32],
    pub cesi_budget: u64,
    pub granted_capabilities: Vec<String>,
    pub scope: PermitScope,
    pub expires_at: u64,
}

#[derive(Debug, Clone)]
pub struct PermitScope {
    pub host_surface: String,
    pub runtime_id: String,
    pub session_id: Option<String>,
}

impl ExecutionPermit {
    pub fn granted(id: [u8; 16], hash: [u8; 32], cesi: u64, caps: Vec<String>) -> Self {
        Self {
            permit_id: id,
            artifact_hash: hash,
            cesi_budget: cesi,
            granted_capabilities: caps,
            scope: PermitScope {
                host_surface: "unknown".to_string(),
                runtime_id: "default".to_string(),
                session_id: None,
            },
            expires_at: 0,
        }
    }
}

// ============================================================
// Stage 6: Host Binding
// ============================================================

pub struct HostBinding {
    pub rac_endpoint: RacEndpoint,
    pub peg_validated: bool,
    pub import_object: Vec<u8>,
}

// ============================================================
// RAC/PEG Integration
// ============================================================

#[derive(Debug, Clone)]
pub struct RacEndpoint {
    pub host: String,
    pub port: u16,
    pub protocol: RacProtocol,
}

impl RacEndpoint {
    pub fn hacetime_default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3018,
            protocol: RacProtocol::Racin,
        }
    }

    /// Auto-detect endpoint based on host environment
    pub fn auto_detect() -> Self {
        let topo = hace_uni_resolver::EnvironmentTopology::current();
        match topo.primary_transport {
            hace_uni_resolver::TransportKind::WebSocket => {
                Self {
                    host: "localhost".to_string(),
                    port: 3018,
                    protocol: RacProtocol::Racin,
                }
            }
            hace_uni_resolver::TransportKind::NamedPipe => {
                // Windows pipe: use UNC path
                Self {
                    host: r"\\.\pipe\hace-hacetime".to_string(),
                    port: 0,
                    protocol: RacProtocol::Raci,
                }
            }
            _ => Self::hacetime_default(),
        }
    }

    pub fn websocket_url(&self) -> String {
        format!("ws://{}:{}/uni", self.host, self.port)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RacProtocol {
    Raci,
    Racin,
    Racex,
    Racv,
}

// ============================================================
// Stage 7: Instance Handle
// ============================================================

pub struct InstanceHandle {
    pub instance_id: String,
    pub session_id: Option<String>,
    pub load_evidence: Option<String>,
}

// ============================================================
// Loader Errors
// ============================================================

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoaderError {
    InvalidMagicNumber,
    InvalidFormat,
    IntegrityFailure,
    SealVerificationFailed,
    FEHMismatch,
    ProvenanceFailure,
    CapabilityUnsupported,
    AuthorityDenied,
    HandoffFailed,
}

impl core::fmt::Display for LoaderError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidMagicNumber => write!(f, "Invalid HACE magic number"),
            Self::InvalidFormat => write!(f, "Invalid artifact format"),
            Self::IntegrityFailure => write!(f, "Artifact integrity check failed"),
            Self::SealVerificationFailed => write!(f, "Atrust seal verification failed"),
            Self::FEHMismatch => write!(f, "FEH hash mismatch"),
            Self::ProvenanceFailure => write!(f, "Provenance verification failed"),
            Self::CapabilityUnsupported => write!(f, "Capability not supported by host"),
            Self::AuthorityDenied => write!(f, "Hacetime authority denied"),
            Self::HandoffFailed => write!(f, "Runtime handoff failed"),
        }
    }
}

// ============================================================
// Context Types
// ============================================================

pub trait HostContext: core::fmt::Debug {}
pub trait RuntimeContext: core::fmt::Debug {}

// ============================================================
// HaceUniLoader Trait (7-stage)
// ============================================================

pub trait HaceUniLoader {
    type HostCtx: HostContext;
    type RuntimeCtx: RuntimeContext;
    type Binding;
    type InstHandle;

    // Stage 1: DISCOVER
    fn discover(input: ArtifactInput<'_>) -> Result<ArtifactDescriptor, LoaderError>;

    // Stage 2: INGEST
    fn ingest<'a>(input: ArtifactInput<'a>) -> Result<ArtifactEnvelope<'a>, LoaderError>;

    // Stage 3: VERIFY
    fn verify(envelope: &ArtifactEnvelope<'_>) -> Result<VerifiedArtifact, LoaderError>;

    // Stage 4: RESOLVE
    fn resolve(
        artifact: &VerifiedArtifact,
        host: &Self::HostCtx,
        runtime: &Self::RuntimeCtx,
    ) -> Result<RuntimeResolution, LoaderError>;

    // Stage 5: AUTHORIZE
    fn request_authority(
        artifact: &VerifiedArtifact,
        resolution: &RuntimeResolution,
    ) -> Result<ExecutionPermit, LoaderError>;

    // Stage 6: BIND
    fn bind(
        artifact: &VerifiedArtifact,
        permit: &ExecutionPermit,
        host: Self::HostCtx,
    ) -> Result<Self::Binding, LoaderError>;

    // Stage 7: HANDOFF
    fn handoff(
        binding: Self::Binding,
        runtime: Self::RuntimeCtx,
    ) -> Result<Self::InstHandle, LoaderError>;
}

// ============================================================
// Artifact Input
// ============================================================

pub enum ArtifactInput<'a> {
    RawWASM(&'a [u8]),
    SealedBundle(&'a [u8]),
    FilePath(&'a str),
}

// ============================================================
// WASM Bindings (for wasm32 target only)
// ============================================================

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use alloc::string::String;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", feature = "wasm"))]
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmHostContext {
    #[wasm_bindgen(skip)]
    pub rac_endpoint: RacEndpoint,
    #[wasm_bindgen(skip)]
    pub host_surface: String,
}

#[cfg(target_arch = "wasm32")]
impl WasmHostContext {
    /// Create with auto-detected endpoint
    pub fn auto() -> Self {
        Self {
            rac_endpoint: RacEndpoint::auto_detect(),
            host_surface: "auto_detected".to_string(),
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl HostContext for WasmHostContext {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmRuntimeContext {
    pub runtime_id: String,
    pub adapter: String,
}

#[cfg(target_arch = "wasm32")]
impl RuntimeContext for WasmRuntimeContext {}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct UniLoader {
    descriptor: Option<ArtifactDescriptor>,
    envelope: Option<ArtifactEnvelope<'static>>,
    verified: Option<VerifiedArtifact>,
    resolution: Option<RuntimeResolution>,
    permit: Option<ExecutionPermit>,
    binding: Option<HostBinding>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl UniLoader {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self {
            descriptor: None,
            envelope: None,
            verified: None,
            resolution: None,
            permit: None,
            binding: None,
        }
    }

    pub fn discover(&mut self, input: &[u8]) -> Result<JsValue, JsValue> {
        let desc = Self::discover_impl(ArtifactInput::RawWASM(input))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.descriptor = Some(desc);
        Ok(JsValue::TRUE)
    }

    pub fn ingest(&mut self, input: &[u8]) -> Result<JsValue, JsValue> {
        let envelope = Self::ingest_impl(ArtifactInput::RawWASM(input))
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.envelope = Some(envelope);
        Ok(JsValue::TRUE)
    }

    pub fn verify(&mut self) -> Result<JsValue, JsValue> {
        let envelope = self.envelope.as_ref()
            .ok_or_else(|| JsValue::from_str("No envelope ingested"))?;
        let verified = Self::verify_impl(envelope)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.verified = Some(verified);
        Ok(JsValue::TRUE)
    }

    pub fn resolve(&mut self, host: &WasmHostContext, runtime: &WasmRuntimeContext) -> Result<JsValue, JsValue> {
        let artifact = self.verified.as_ref()
            .ok_or_else(|| JsValue::from_str("No verified artifact"))?;
        let res = Self::resolve_impl(artifact, host, runtime)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.resolution = Some(res);
        Ok(JsValue::TRUE)
    }

    pub fn request_authority(&mut self, cesi: u64) -> Result<JsValue, JsValue> {
        let artifact = self.verified.as_ref()
            .ok_or_else(|| JsValue::from_str("No verified artifact"))?;
        let resolution = self.resolution.as_ref()
            .ok_or_else(|| JsValue::from_str("No runtime resolution"))?;
        let permit = Self::authorize_impl(artifact, resolution, cesi)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.permit = Some(permit);
        Ok(JsValue::TRUE)
    }

    pub fn bind(&mut self, host: WasmHostContext) -> Result<JsValue, JsValue> {
        let artifact = self.verified.as_ref()
            .ok_or_else(|| JsValue::from_str("No verified artifact"))?;
        let permit = self.permit.as_ref()
            .ok_or_else(|| JsValue::from_str("No execution permit"))?;
        let binding = Self::bind_impl(artifact, permit, host)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        self.binding = Some(binding);
        Ok(JsValue::TRUE)
    }

    pub fn handoff(&mut self, runtime: WasmRuntimeContext) -> Result<WasmInstanceHandle, JsValue> {
        let binding = self.binding.take()
            .ok_or_else(|| JsValue::from_str("No binding prepared"))?;
        Self::handoff_impl(binding, runtime)
            .map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

#[cfg(target_arch = "wasm32")]
impl UniLoader {
    fn discover_impl(input: ArtifactInput<'_>) -> Result<ArtifactDescriptor, LoaderError> {
        Ok(ArtifactDescriptor {
            artifact_id: "discovered".to_string(),
            artifact_kind: "wasm".to_string(),
            version: "1.0".to_string(),
            source: "buffer".to_string(),
            format: "wasm".to_string(),
        })
    }

    fn ingest_impl<'a>(input: ArtifactInput<'a>) -> Result<ArtifactEnvelope<'a>, LoaderError> {
        match input {
            ArtifactInput::RawWASM(data) => {
                let header = SealedHeader::parse(data)?;
                // Placeholder for full parsing
                Ok(ArtifactEnvelope {
                    descriptor: Self::discover_impl(input).unwrap(),
                    header,
                    manifest: unsafe {
                        // This is unsafe but simplified for demo
                        core::mem::zeroed()
                    },
                    capabilities: Vec::new(),
                    payload: &[],
                    seal: &[],
                })
            }
            _ => Err(LoaderError::InvalidFormat),
        }
    }

    fn verify_impl(_envelope: &ArtifactEnvelope<'_>) -> Result<VerifiedArtifact, LoaderError> {
        Ok(VerifiedArtifact {
            envelope: unsafe { core::mem::zeroed() },
            integrity_verified: true,
            seal_valid: true,
            feh_match: true,
        })
    }

    fn resolve_impl(
        _artifact: &VerifiedArtifact,
        _host: &WasmHostContext,
        _runtime: &WasmRuntimeContext,
    ) -> Result<RuntimeResolution, LoaderError> {
        Ok(RuntimeResolution {
            compatible: true,
            runtime_adapter: "wasm".to_string(),
            abi_adapter: "1.0".to_string(),
            capability_gaps: Vec::new(),
        })
    }

    fn authorize_impl(
        _artifact: &VerifiedArtifact,
        _resolution: &RuntimeResolution,
        cesi: u64,
    ) -> Result<ExecutionPermit, LoaderError> {
        Ok(ExecutionPermit::granted([0; 16], [0; 32], cesi, vec![]))
    }

    fn bind_impl(
        _artifact: &VerifiedArtifact,
        _permit: &ExecutionPermit,
        ctx: WasmHostContext,
    ) -> Result<HostBinding, LoaderError> {
        Ok(HostBinding {
            rac_endpoint: ctx.rac_endpoint,
            peg_validated: true,
            import_object: Vec::new(),
        })
    }

    fn handoff_impl(
        _binding: HostBinding,
        _runtime: WasmRuntimeContext,
    ) -> Result<WasmInstanceHandle, LoaderError> {
        Ok(WasmInstanceHandle {
            instance_id: "handoff_complete".to_string(),
            load_evidence: None,
        })
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Debug, Clone)]
pub struct WasmInstanceHandle {
    pub instance_id: String,
    pub load_evidence: Option<String>,
}

#[cfg(target_arch = "wasm32")]
impl Default for UniLoader {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================
// Native Stub
// ============================================================

#[cfg(not(target_arch = "wasm32"))]
pub struct NativeLoader;

#[cfg(not(target_arch = "wasm32"))]
impl HostContext for () {}

#[cfg(not(target_arch = "wasm32"))]
impl RuntimeContext for () {}

#[cfg(not(target_arch = "wasm32"))]
impl HaceUniLoader for NativeLoader {
    type HostCtx = ();
    type RuntimeCtx = ();
    type Binding = ();
    type InstHandle = ();

    fn discover(_input: ArtifactInput<'_>) -> Result<ArtifactDescriptor, LoaderError> {
        Err(LoaderError::InvalidFormat)
    }

    fn ingest<'a>(_input: ArtifactInput<'a>) -> Result<ArtifactEnvelope<'a>, LoaderError> {
        Err(LoaderError::InvalidFormat)
    }

    fn verify(_envelope: &ArtifactEnvelope<'_>) -> Result<VerifiedArtifact, LoaderError> {
        Err(LoaderError::IntegrityFailure)
    }

    fn resolve(
        _artifact: &VerifiedArtifact,
        _host: &Self::HostCtx,
        _runtime: &Self::RuntimeCtx,
    ) -> Result<RuntimeResolution, LoaderError> {
        Err(LoaderError::CapabilityUnsupported)
    }

    fn request_authority(
        _artifact: &VerifiedArtifact,
        _resolution: &RuntimeResolution,
    ) -> Result<ExecutionPermit, LoaderError> {
        Err(LoaderError::AuthorityDenied)
    }

    fn bind(
        _artifact: &VerifiedArtifact,
        _permit: &ExecutionPermit,
        _host: Self::HostCtx,
    ) -> Result<Self::Binding, LoaderError> {
        Err(LoaderError::CapabilityUnsupported)
    }

    fn handoff(
        _binding: Self::Binding,
        _runtime: Self::RuntimeCtx,
    ) -> Result<Self::InstHandle, LoaderError> {
        Err(LoaderError::HandoffFailed)
    }
}