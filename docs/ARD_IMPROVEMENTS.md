# Architecture Requirements Document: vita_gl_helpers v0.2.0

## Overview

This document describes the architecture decisions and technical implementation for the v0.2.0 improvements to `vita_gl_helpers`.

---

## Architecture Decisions

### AD-1: RAII via Generic Handle Wrapper ✅ IMPLEMENTED

**Decision:** Use a generic `Handle<T>` wrapper with a `GpuResource` trait for RAII.

**Rationale:**
- Single implementation handles all GPU resource types
- Type safety via marker types
- Consistent API across Buffer, Shader, Program, Texture

**Implementation:**
```rust
// src/handle.rs
pub trait GpuResource {
    type Id: Copy + Default + Eq;
    unsafe fn delete(id: Self::Id);
    fn is_null(id: Self::Id) -> bool;
    fn resource_name() -> &'static str;
}

pub struct Handle<T: GpuResource> {
    id: Option<T::Id>,
    _marker: PhantomData<T>,
}

impl<T: GpuResource> Drop for Handle<T> {
    fn drop(&mut self) {
        if let Some(id) = self.id.take() {
            unsafe { T::delete(id); }
        }
    }
}
```

**Files Modified:**
- `src/handle.rs` (new)
- `src/buffer.rs`
- `src/shader.rs`
- `src/program.rs`
- `src/texture.rs`

---

### AD-2: Builder Pattern for Initialization ✅ IMPLEMENTED

**Decision:** Use builder pattern for vitaGL initialization.

**Rationale:**
- Cleaner API than multiple function parameters
- Self-documenting code
- Extensible for future options

**Implementation:**
```rust
// src/lib.rs
pub struct VitaGlBuilder {
    width: i32,
    height: i32,
    msaa: i32,
    ram_threshold: i32,
    // ... more options
}

impl VitaGlBuilder {
    pub fn new() -> Self { /* defaults */ }
    pub fn msaa(mut self, samples: i32) -> Self { self.msaa = samples; self }
    pub fn build(self) -> Result<VitaGlContext, InitError> { /* init vitaGL */ }
}
```

**Files Modified:**
- `src/lib.rs`

---

### AD-3: Deprecation Strategy ✅ IMPLEMENTED

**Decision:** Keep legacy APIs but mark as deprecated.

**Rationale:**
- Allows gradual migration
- No breaking changes for existing users
- Clear migration path via deprecation messages

**Implementation:**
```rust
#[deprecated(since = "0.2.0", note = "Use Shader::compile() instead")]
pub fn load_shader(...) -> Result<LegacyShader, ShaderError> { ... }
```

**Deprecated APIs:**
| Legacy | Replacement |
|--------|-------------|
| `load_shader()` | `Shader::compile()` |
| `link_program()` | `Program::link()` |
| `initialise_default()` | `VitaGlBuilder::new().build()` |
| `GenDelBuffersExt` | `Buffer::new()` (auto-delete) |
| `GenDelTexturesExt` | `Texture::new()` (auto-delete) |
| `LegacyShader` | `Shader` |
| `LegacyProgram` | `Program` |

---

### AD-4: Feature Flags ✅ IMPLEMENTED

**Decision:** Use Cargo feature flags for optional functionality.

**Rationale:**
- Keep core library minimal
- Allow opt-in debug features
- No runtime cost when disabled

**Features:**
```toml
[features]
default = []
debug-gl = []  # Enable GL error checking after operations
logging = ["log"]  # Enable debug logging
```

**Files Modified:**
- `Cargo.toml`
- `src/errors.rs` (conditional debug_check_error)
- `src/handle.rs` (conditional logging)

---

### AD-5: Documentation Strategy ✅ IMPLEMENTED

**Decision:** Comprehensive inline documentation with examples.

**Implementation:**
- Module-level `//!` docs with overview and examples
- Doc comments on all public items
- `# Example` sections with `ignore` (can't run without GL context)

**Files Modified:**
- All `src/*.rs` files

---

## Module Structure ✅ IMPLEMENTED

```
vita_gl_helpers/
├── src/
│   ├── lib.rs          # VitaGlBuilder, VitaGlContext, prelude, re-exports
│   ├── handle.rs       # Handle<T>, GpuResource trait
│   ├── buffer.rs       # Buffer (RAII)
│   ├── shader.rs       # Shader (RAII), ShaderType, ShaderError
│   ├── program.rs      # Program (RAII), LinkError
│   ├── texture.rs      # Texture (RAII), BoundTexture
│   ├── attribute.rs    # Attribute, AttributeFormat, attribute_table!
│   ├── uniforms.rs     # Uniform types, Sampler2D, uniform_table!
│   ├── draw.rs         # Mode, draw_arrays, Elements trait
│   └── errors.rs       # GlError, error utilities
├── examples/
│   ├── triangle.rs           # Basic rendering example
│   ├── textured_quad.rs      # Texture example
│   └── instanced_colorful_grid/  # Instanced rendering example
├── docs/
│   ├── PRD_IMPROVEMENTS.md   # Product requirements
│   └── ARD_IMPROVEMENTS.md   # Architecture requirements (this file)
└── .github/
    └── workflows/
        └── ci.yml            # GitHub Actions CI
```

---

## Resource Lifecycle ✅ IMPLEMENTED

### Buffer Lifecycle
```
Buffer::new() → glGenBuffers(1)
buffer.data() → glBindBuffer + glBufferData
buffer.bind() → glBindBuffer
<drop> → glDeleteBuffers(1)
```

### Shader Lifecycle
```
Shader::compile() → glCreateShader + glShaderSource + glCompileShader
shader.id() → access raw ID
<drop> → glDeleteShader
```

### Program Lifecycle
```
Program::link() → glCreateProgram + glAttachShader + glLinkProgram
program.use_program() → glUseProgram
program.get_uniform_location() → glGetUniformLocation
program.get_attrib_location() → glGetAttribLocation
<drop> → glDeleteProgram
```

### Texture Lifecycle
```
Texture::new() → glGenTextures(1)
texture.bind() → glBindTexture
texture.bind_then(f) → bind + f(BoundTexture) 
<drop> → glDeleteTextures(1)
```

---

## Error Handling ✅ IMPLEMENTED

| Error Type | Used For |
|------------|----------|
| `ShaderError` | Shader compilation failures |
| `LinkError` | Program linking failures |
| `InitError` | vitaGL initialization failures |
| `GlError` | Runtime GL errors |
| `MissingAttributes` | Missing attribute locations |
| `MissingUniforms` | Missing uniform locations |

All error types implement `std::error::Error` and `Display`.

---

## API Design Principles ✅ IMPLEMENTED

1. **RAII by default** - All GPU resources auto-cleanup
2. **Type safety** - Strongly typed enums instead of raw GLenums
3. **Builder pattern** - For complex initialization
4. **Method chaining** - Fluent APIs where appropriate
5. **Zero-cost abstractions** - No runtime overhead vs raw GL
6. **Backward compatibility** - Legacy APIs deprecated, not removed

---

## Testing Strategy ✅ IMPLEMENTED

### Unit Tests
- Type conversion tests
- API surface tests (compile-time verification)

### Integration Tests
- Requires GL context (run on device only)
- Examples serve as integration tests

### CI Pipeline
- `cargo check` - Compilation
- `cargo test` - Unit tests (no GL context)
- `cargo fmt --check` - Formatting
- `cargo clippy` - Lints
- `cargo doc` - Documentation builds

---

## Performance Considerations

### Implemented ✅
- Zero-cost RAII (only cleanup code on drop)
- Inline hints on hot paths
- Batch creation for buffers/textures

### Future Optimizations
- [ ] VAO caching for attribute setup
- [ ] State tracking to reduce redundant GL calls
- [ ] Memory pooling for dynamic buffers

---

## Security Considerations ✅ ADDRESSED

1. **Unsafe code minimization** - Only at GL FFI boundary
2. **Handle validation** - Null checks before operations
3. **No undefined behavior** - Safe Rust wrapper over unsafe GL

---

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `gl` | 0.14 | OpenGL bindings |
| `derive_more` | 2.1 | Derive macros |
| `log` | 0.4 | Logging (optional) |

---

## Version Compatibility

| Component | Requirement |
|-----------|-------------|
| Rust | 1.70+ (edition 2021) |
| vitaGL | Latest |
| Vita SDK | Compatible with vitaGL |

---

## Implementation Status

| Component | Status |
|-----------|--------|
| RAII Handle | ✅ Complete |
| Buffer RAII | ✅ Complete |
| Shader RAII | ✅ Complete |
| Program RAII | ✅ Complete |
| Texture RAII | ✅ Complete |
| VitaGlBuilder | ✅ Complete |
| Documentation | ✅ Complete |
| CI/CD | ✅ Complete |
| Examples | ✅ Complete (3) |
| License files | ✅ Complete |
| Changelog | ✅ Complete |
