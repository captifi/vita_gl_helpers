//! Textured quad example using the new RAII API
//!
//! This example demonstrates:
//! - Using VitaGlBuilder for initialization
//! - RAII Buffer, Shader, Program, and Texture types
//! - attribute_table! and uniform_table! macros
//! - Texture loading and binding

use vita_gl_helpers::{
    attribute_table,
    buffer::Buffer,
    draw::{draw_arrays, Mode},
    program::Program,
    shader::{Shader, ShaderType},
    texture::Texture,
    uniform_table,
    VitaGlBuilder,
};

// Define uniform table for our shader
uniform_table!(MyUniformTable,
    tex: Sampler2D => "uTexture"
);

// Define attribute table for our shader
attribute_table!(MyAttributeTable,
    pos => "aPos",
    tex_coord => "aTexCoord"
);

// Quad vertices: position (x, y) and texture coordinates (u, v)
const QUAD_VERTICES: &[[f32; 4]] = &[
    // Position      TexCoord
    [-0.5, -0.5,    0.0, 1.0],  // bottom-left
    [ 0.5, -0.5,    1.0, 1.0],  // bottom-right  
    [-0.5,  0.5,    0.0, 0.0],  // top-left
    [ 0.5, -0.5,    1.0, 1.0],  // bottom-right
    [ 0.5,  0.5,    1.0, 0.0],  // top-right
    [-0.5,  0.5,    0.0, 0.0],  // top-left
];

fn main() {
    // Initialize vitaGL with builder pattern
    let ctx = VitaGlBuilder::new()
        .build()
        .expect("Failed to initialize vitaGL");

    // Compile shaders (automatically deleted when dropped)
    let vert_shader = Shader::compile(
        r#"
        void main(
            float2 aPos,
            float2 aTexCoord,
            float4 out gl_Position : POSITION,
            float2 out vTexCoord : TEXCOORD0
        ) {
            gl_Position = float4(aPos, 0.0, 1.0);
            vTexCoord = aTexCoord;
        }
        "#,
        ShaderType::Vertex,
    )
    .expect("Failed to compile vertex shader");

    let frag_shader = Shader::compile(
        r#"
        float4 main(
            uniform sampler2D uTexture,
            float2 vTexCoord : TEXCOORD0
        ) {
            return tex2D(uTexture, vTexCoord);
        }
        "#,
        ShaderType::Fragment,
    )
    .expect("Failed to compile fragment shader");

    // Link program (automatically deleted when dropped)
    let program = Program::link(&vert_shader, &frag_shader)
        .expect("Failed to link program");

    // Get uniform and attribute locations
    let utable = program.get_uniform_table::<MyUniformTable>()
        .expect("Failed to get uniform table");
    let atable = program.get_attribute_table::<MyAttributeTable>()
        .expect("Failed to get attribute table");

    // Create vertex buffer (automatically deleted when dropped)
    let vertex_buffer = Buffer::new();
    vertex_buffer.data(gl::ARRAY_BUFFER, QUAD_VERTICES, gl::STATIC_DRAW);

    // Create a procedural checkerboard texture
    let texture = Texture::new();
    let tex_data = create_checkerboard_texture(8, 8, 1);
    
    // Use bind_then pattern for texture setup
    texture.bind_then(gl::TEXTURE_2D, |bound| {
        bound.image_2d_data(
            0,
            gl::RGBA as i32,
            8,
            8,
            gl::RGBA,
            gl::UNSIGNED_BYTE,
            &tex_data,
        );
        bound.set_nearest_filtering();
        bound.set_wrap(gl::CLAMP_TO_EDGE as i32);
    });

    // Set clear color
    unsafe {
        gl::ClearColor(0.2, 0.3, 0.3, 1.0);
    }

    let stride = std::mem::size_of::<[f32; 4]>() as i32;

    // Main render loop
    loop {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Use program
        program.use_program();

        // Bind texture to unit 0 and set uniform
        texture.bind_to_unit(0, gl::TEXTURE_2D);
        utable.tex.set(0);

        // Enable vertex attributes
        atable.pos.enable();
        atable.tex_coord.enable();

        // Bind buffer and set attribute pointers
        vertex_buffer.bind(gl::ARRAY_BUFFER);
        
        unsafe {
            // Position: offset 0
            gl::VertexAttribPointer(
                atable.pos.0 as u32,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                std::ptr::null(),
            );
            // TexCoord: offset 2 floats = 8 bytes
            gl::VertexAttribPointer(
                atable.tex_coord.0 as u32,
                2,
                gl::FLOAT,
                gl::FALSE,
                stride,
                (2 * std::mem::size_of::<f32>()) as *const _,
            );
        }

        // Draw the quad
        draw_arrays(Mode::Triangles, 0, 6);

        // Swap buffers
        ctx.swap_buffers();
    }
}

/// Create a checkerboard texture pattern.
fn create_checkerboard_texture(width: usize, height: usize, cell_size: usize) -> Vec<u8> {
    let mut data = Vec::with_capacity(width * height * 4);
    
    for y in 0..height {
        for x in 0..width {
            let is_white = ((x / cell_size) + (y / cell_size)) % 2 == 0;
            let (r, g, b, a) = if is_white {
                (255u8, 255, 255, 255)
            } else {
                (100, 100, 100, 255)
            };
            data.extend_from_slice(&[r, g, b, a]);
        }
    }
    
    data
}
