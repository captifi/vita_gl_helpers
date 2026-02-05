# vitaGL Helpers

[![CI](https://github.com/LexiBigCheese/vita_gl_helpers/workflows/CI/badge.svg)](https://github.com/LexiBigCheese/vita_gl_helpers/actions)
[![Crates.io](https://img.shields.io/crates/v/vita_gl_helpers.svg)](https://crates.io/crates/vita_gl_helpers)
[![Documentation](https://docs.rs/vita_gl_helpers/badge.svg)](https://docs.rs/vita_gl_helpers)
[![License](https://img.shields.io/crates/l/vita_gl_helpers.svg)](LICENSE)

Safe, ergonomic Rust helpers for [vitaGL](https://github.com/Rinnegatamante/vitaGL) development on PlayStation Vita.

## Features

- 🛡️ **RAII Resource Management** - GPU resources (buffers, shaders, textures) are automatically cleaned up when dropped
- 🔒 **Type-Safe APIs** - Prevents common OpenGL mistakes at compile time
- 📝 **Declarative Macros** - `attribute_table!` and `uniform_table!` reduce shader boilerplate
- 🏗️ **Builder Patterns** - Fluent APIs for configuration
- 📚 **Comprehensive Documentation** - All public APIs documented with examples

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
vita_gl_helpers = "0.2"
gl = "0.14"
```

### Basic Example

```rust
use vita_gl_helpers::prelude::*;

fn main() {
    // Initialize vitaGL with builder pattern
    let ctx = VitaGlBuilder::new()
        .msaa(4)
        .build()
        .expect("Failed to initialize vitaGL");

    // Compile shaders - automatically deleted when dropped
    let vert = Shader::compile(r#"
        void main(float2 aPos, float4 out gl_Position : POSITION) {
            gl_Position = float4(aPos, 0.0, 1.0);
        }
    "#, ShaderType::Vertex).unwrap();

    let frag = Shader::compile(r#"
        float4 main() {
            return float4(1.0, 0.0, 0.0, 1.0);
        }
    "#, ShaderType::Fragment).unwrap();

    // Link program - automatically deleted when dropped
    let program = Program::link(&vert, &frag).unwrap();

    // Create buffer - automatically deleted when dropped
    let buffer = Buffer::new();
    let vertices: [f32; 6] = [0.0, 0.5, 0.5, -0.5, -0.5, -0.5];
    buffer.data(gl::ARRAY_BUFFER, &vertices, gl::STATIC_DRAW);

    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
    }

    loop {
        unsafe { gl::Clear(gl::COLOR_BUFFER_BIT); }
        program.use_program();
        // ... draw ...
        ctx.swap_buffers();
    }
}
```

### Using Attribute Tables

```rust
use vita_gl_helpers::prelude::*;

// Define attribute table for your shader
attribute_table!(MyAttributes,
    pos => "aPosition",
    color => "aColor"
);

fn setup(program: &Program) {
    // Get attribute locations
    let attrs = program.get_attribute_table::<MyAttributes>().unwrap();

    // Enable all attributes
    attrs.enable_all();

    // Configure attribute pointers
    let pos_format = AttributeFormat::FLOAT2;
    let color_format = AttributeFormat::UBYTE4_NORM;

    pos_buffer.bind_to(attrs.pos, pos_format, 0, 0);
    color_buffer.bind_to(attrs.color, color_format, 0, 0);
}
```

### Using Uniform Tables

```rust
use vita_gl_helpers::prelude::*;

// Define uniform table for your shader
uniform_table!(MyUniforms,
    mvp: UniformMatrix4fv => "uMVP",
    color: Uniform4fv => "uColor",
    time: Uniform1fv => "uTime"
);

fn render(program: &Program, uniforms: &MyUniforms) {
    program.use_program();
    
    // Set uniforms
    uniforms.mvp.set(matrix_data, false);
    uniforms.color.set([1.0, 0.5, 0.0, 1.0]);
    uniforms.time.set(elapsed);
}
```

## Modules

| Module | Description |
|--------|-------------|
| [`buffer`](src/buffer.rs) | GPU buffer management with RAII |
| [`shader`](src/shader.rs) | Shader compilation with RAII |
| [`program`](src/program.rs) | Shader program linking with RAII |
| [`texture`](src/texture.rs) | Texture management with RAII |
| [`attribute`](src/attribute.rs) | Vertex attribute configuration |
| [`uniforms`](src/uniforms.rs) | Uniform variable management |
| [`draw`](src/draw.rs) | Draw call helpers |
| [`errors`](src/errors.rs) | GL error handling |
| [`handle`](src/handle.rs) | Generic RAII handle wrapper |

## Feature Flags

| Feature | Description |
|---------|-------------|
| `debug-gl` | Enable GL error checking after operations |
| `logging` | Enable debug logging via `log` crate |

```toml
[dependencies]
vita_gl_helpers = { version = "0.2", features = ["debug-gl", "logging"] }
```

## Migration from v0.1

Version 0.2 introduces breaking changes for RAII support:

| v0.1 | v0.2 | Notes |
|------|------|-------|
| `load_shader()` | `Shader::compile()` | Shaders auto-delete |
| `link_program()` | `Program::link()` | Programs auto-delete |
| `buffers.gen_buffers()` | `Buffer::new()` | Buffers auto-delete |
| `shader.delete()` | (removed) | Automatic cleanup |
| `initialise_default()` | `VitaGlBuilder::new().build()` | Builder pattern |

Legacy APIs are available but deprecated.

## Prerequisites

- [Vita SDK](https://vitasdk.org/)
- [vitaGL](https://github.com/Rinnegatamante/vitaGL)
- Rust toolchain with Vita target

## Contributing

Contributions are welcome! Please see the [docs/PRD_IMPROVEMENTS.md](docs/PRD_IMPROVEMENTS.md) and [docs/ARD_IMPROVEMENTS.md](docs/ARD_IMPROVEMENTS.md) for the roadmap.

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgments

- [vitaGL](https://github.com/Rinnegatamante/vitaGL) by Rinnegatamante
- [Vita SDK](https://vitasdk.org/) community
