//! Instanced colorful grid example using the new RAII API
//!
//! This example demonstrates:
//! - Using VitaGlBuilder for initialization
//! - RAII Buffer, Shader, and Program types
//! - attribute_table! and uniform_table! macros
//! - Instanced rendering

use vita_gl_helpers::{
    attribute::{AttributeFormat, AttributeSize, AttributeTable, AttributeType},
    attribute_table,
    buffer::Buffer,
    draw::{Elements, Mode},
    program::Program,
    shader::{Shader, ShaderType},
    uniform_table,
    uniforms::UniformTable,
    VitaGlBuilder,
};

// Define uniform table for our shader
uniform_table!(MyUniformTable,
    rect_dim: Uniform2fv => "rect_dim"
);

// Define attribute table for our shader
attribute_table!(MyAttributeTable,
    pos => "pos",
    color_top => "color_top",
    color_bottom => "color_bottom"
);

pub const POSITIONS: &[[f32; 2]] = &[[-0.5, 0.5], [0.0, 0.5], [-0.5, 0.0], [0.0, 0.0]];
pub const TOP_COLORS: &[u32] = &[0xFFFF0000, 0xFF0000FF, 0xFFA526FF, 0xFFFFFFFF];
pub const BOTTOM_COLORS: &[u32] = &[0xFF0000FF, 0xFFA526FF, 0xFF00FF00, 0xFF000000];
pub const INDICES: &[u16] = &[0, 1, 3, 2];

const COLOR_FORMAT: AttributeFormat = AttributeFormat {
    size: AttributeSize::Four,
    type_: AttributeType::UnsignedByte,
    normalized: true,
};

const POS_FORMAT: AttributeFormat = AttributeFormat {
    size: AttributeSize::Two,
    type_: AttributeType::Float,
    normalized: false,
};

fn main() {
    // Initialize vitaGL with builder pattern
    let ctx = VitaGlBuilder::new()
        .build()
        .expect("Failed to initialize vitaGL");

    // Compile shaders (automatically deleted when dropped)
    let vertex_shader = Shader::compile(include_str!("vert.cg"), ShaderType::Vertex)
        .expect("Failed to compile vertex shader");
    let fragment_shader = Shader::compile(include_str!("frag.cg"), ShaderType::Fragment)
        .expect("Failed to compile fragment shader");

    // Link program (automatically deleted when dropped)
    let program = Program::link(&vertex_shader, &fragment_shader)
        .expect("Failed to link program");

    // Get uniform and attribute locations
    let utable = program.get_uniform_table::<MyUniformTable>()
        .expect("Failed to get uniform table");
    let atable = program.get_attribute_table::<MyAttributeTable>()
        .expect("Failed to get attribute table");

    // Create buffers using the new API (automatically deleted when dropped)
    let pos_buffer = Buffer::new();
    let top_color_buffer = Buffer::new();
    let bottom_color_buffer = Buffer::new();
    let index_buffer = Buffer::new();

    // Upload data to buffers
    pos_buffer.data(gl::ARRAY_BUFFER, POSITIONS, gl::STATIC_DRAW);
    top_color_buffer.data(gl::ARRAY_BUFFER, TOP_COLORS, gl::STATIC_DRAW);
    bottom_color_buffer.data(gl::ARRAY_BUFFER, BOTTOM_COLORS, gl::STATIC_DRAW);
    index_buffer.data(gl::ELEMENT_ARRAY_BUFFER, INDICES, gl::STATIC_DRAW);

    // Set clear color
    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }

    // Main render loop
    loop {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Use program
        program.use_program();

        // Enable vertex attributes
        atable.enable_all();

        // Bind buffers to attributes
        pos_buffer.bind_to(atable.pos, POS_FORMAT, 0, 0);
        top_color_buffer.bind_to(atable.color_top, COLOR_FORMAT, 0, 0);
        bottom_color_buffer.bind_to(atable.color_bottom, COLOR_FORMAT, 0, 0);

        // Set uniforms
        utable.rect_dim.set([0.25, -0.5]);

        // Draw instanced using index buffer
        use vita_gl_helpers::draw::ElementsBufIdU16;
        ElementsBufIdU16 {
            buffer_id: index_buffer.id_unwrap(),
            len: 4,
        }
        .draw_instanced(Mode::Quads, 4);

        // Swap buffers
        ctx.swap_buffers();
    }
}
