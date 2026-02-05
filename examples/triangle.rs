//! Triangle rendering example using the new RAII API
//!
//! This example demonstrates:
//! - Using VitaGlBuilder for initialization
//! - RAII Buffer, Shader, and Program types
//! - attribute_table! macro for vertex attributes

use vita_gl_helpers::{
    attribute::{AttributeFormat, AttributeSize, AttributeType},
    attribute_table,
    buffer::Buffer,
    draw::{Elements, Mode},
    errors::eprintln_errors,
    program::Program,
    shader::{Shader, ShaderType},
    VitaGlBuilder,
};

// Define attribute table for our shader
attribute_table!(MyAttributeTable,
    pos => "aPos",
    color => "aColor"
);

const VERTEX_POS: &[f32; 6] = &[0.0, 0.5, 0.5, -0.5, -0.5, -0.5];
const VERTEX_COLOR: &[u32; 3] = &[0xFF0000FFu32, 0xFF00FF00, 0xFFFF0000];

fn main() {
    // Initialize vitaGL with builder pattern
    let ctx = VitaGlBuilder::new()
        .build()
        .expect("Failed to initialize vitaGL");

    // Compile shaders (automatically deleted when dropped)
    let vert_shader = Shader::compile(
        r#"
        void main(float2 aPos, float4 aColor, float4 out gl_Position : POSITION, float4 out vColor: COLOR0) {
            gl_Position = float4(aPos, 0.0, 1.0);
            vColor = aColor;
        }
        "#,
        ShaderType::Vertex,
    )
    .expect("Failed to compile vertex shader");

    let frag_shader = Shader::compile(
        r#"
        float4 main(float4 vColor: COLOR0) {
            return vColor;
        }
        "#,
        ShaderType::Fragment,
    )
    .expect("Failed to compile fragment shader");

    // Link program (automatically deleted when dropped)
    let program = Program::link(&vert_shader, &frag_shader)
        .expect("Failed to link program");

    // Get attribute locations
    let atable = program.get_attribute_table::<MyAttributeTable>()
        .expect("Failed to get attribute table");

    // Create buffers using the new API (automatically deleted when dropped)
    let pos_buffer = Buffer::new();
    let color_buffer = Buffer::new();
    let index_buffer = Buffer::new();

    // Upload data to buffers
    pos_buffer.data(gl::ARRAY_BUFFER, VERTEX_POS, gl::STATIC_DRAW);
    color_buffer.data(gl::ARRAY_BUFFER, VERTEX_COLOR, gl::STATIC_DRAW);
    index_buffer.data(gl::ELEMENT_ARRAY_BUFFER, &[0u32, 1, 2], gl::STATIC_DRAW);

    // Set clear color
    unsafe {
        gl::ClearColor(1.0, 1.0, 1.0, 1.0);
    }

    // Define attribute formats
    let pos_format = AttributeFormat {
        normalized: false,
        size: AttributeSize::Two,
        type_: AttributeType::Float,
    };
    let color_format = AttributeFormat {
        normalized: true,
        size: AttributeSize::Four,
        type_: AttributeType::UnsignedByte,
    };

    // Main render loop
    loop {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        // Use program
        program.use_program();

        // Enable vertex attributes
        atable.pos.enable();
        atable.color.enable();

        // Bind buffers to attributes
        pos_buffer.bind_to(atable.pos, pos_format, 0, 0);
        color_buffer.bind_to(atable.color, color_format, 0, 0);

        // Draw using index buffer
        use vita_gl_helpers::draw::ElementsBufIdU32;
        ElementsBufIdU32 {
            buffer_id: index_buffer.id_unwrap(),
            len: 3,
        }
        .draw(Mode::Triangles);

        // Swap buffers
        ctx.swap_buffers();

        // Check for errors
        eprintln_errors();
    }
}
