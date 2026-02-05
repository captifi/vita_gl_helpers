# Changelog

All notable changes to `vita_gl_helpers` will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.2.0] - 2026-05-02

### Added

#### RAII Resource Management
- **`Handle<T>` generic wrapper** - Automatic GPU resource cleanup via `Drop` trait
- **`Buffer`** - RAII buffer type with `Buffer::new()` and `Buffer::new_batch()`
- **`Shader`** - RAII shader type with `Shader::compile(source, ShaderType)`
- **`Program`** - RAII program type with `Program::link(&vert, &frag)`
- **`Texture`** - RAII texture type with `Texture::new()` and `Texture::new_batch()`

#### Builder Pattern Initialization
- **`VitaGlBuilder`** - Fluent API for vitaGL configuration
  - `.resolution(width, height)` - Set display resolution
  - `.msaa(samples)` - Set MSAA level
  - `.ram_threshold(bytes)` - Set RAM threshold
  - `.shader_optimization(level)` - Set shader compiler optimization
  - `.fast_math(enabled)` / `.fast_precision(enabled)` / `.fast_int(enabled)`
  - `.build()` - Initialize and return `VitaGlContext`
- **`VitaGlContext`** - Represents initialized graphics context
  - `.swap_buffers()` - Swap front/back buffers
  - `.resolution()` / `.width()` / `.height()` - Query display info
  - `.aspect_ratio()` - Get aspect ratio

#### New Features
- **`Sampler2D`** uniform type for texture binding
- **`BoundTexture::set_linear_filtering()`** and `set_nearest_filtering()` helpers
- **`BoundTexture::set_wrap()`** helper for texture wrapping
- **`Texture::bind_to_unit()`** for easy texture unit binding
- **`AttributeFormat` constants** - `FLOAT1`, `FLOAT2`, `FLOAT3`, `FLOAT4`, `UBYTE4_NORM`, `USHORT2_NORM`
- **`GlError::is_error()`** and `GlError::description()` methods
- **`check_error()`** - Returns `Result<(), GlError>` for `?` operator
- **`assert_no_error()`** - Panics on GL error (debugging)
- **`debug_check_error()`** - Conditional error checking with `debug-gl` feature
- **`draw_arrays_instanced()`** function
- **Additional draw modes** - `LineStrip`, `LineLoop`, `TriangleStrip`, `TriangleFan`
- **`ElementsBufIdU16` / `ElementsBufIdU32`** - Draw with raw buffer IDs

#### Documentation
- Comprehensive module-level documentation with examples
- Doc comments on all public types, traits, and functions
- Code examples in doc comments
- Migration guide from v0.1

#### Infrastructure
- **GitHub Actions CI** - Check, test, fmt, clippy, docs
- **Feature flags** - `debug-gl`, `logging`
- **Prelude module** - `use vita_gl_helpers::prelude::*`
- **PRD and ARD documents** - Project roadmap in `docs/`

### Changed
- **Rust edition** - 2024 → 2021 for stability
- **Cargo.toml** - Added metadata (authors, description, repository, license, keywords, categories)
- **`Program::use_program()`** - New preferred name (alias `use_me()` retained)
- **`attribute_table!` macro** - Now supports trailing commas
- **`uniform_table!` macro** - Now supports trailing commas

### Deprecated
- `load_shader()` → Use `Shader::compile()` instead
- `link_program()` → Use `Program::link()` instead
- `initialise_default()` / `initialise_extended()` → Use `VitaGlBuilder`
- `RuntimeShaderCompilerSettings` → Use `VitaGlBuilder`
- `VglInitSettings` → Use `VitaGlBuilder`
- `GenDelBuffersExt` trait → Buffers auto-delete on drop
- `GenDelTexturesExt` trait → Textures auto-delete on drop
- `LegacyShader` / `LegacyProgram` → Use RAII types

### Fixed
- **Memory leaks** - All GPU resources now auto-cleanup via RAII

---

## [0.1.0] - Initial Release

### Added
- Basic vitaGL initialization helpers
- Buffer management with `GenDelBuffersExt` trait
- Shader compilation via `load_shader()`
- Program linking via `link_program()`
- Texture management with `GenDelTexturesExt` trait
- Vertex attribute configuration with `attribute_table!` macro
- Uniform management with `uniform_table!` macro
- Draw call helpers (arrays, elements, instanced)
- GL error handling utilities
- Example: triangle rendering
- Example: instanced colorful grid
