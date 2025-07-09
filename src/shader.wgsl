// Vertex shader

struct VertexInput {
    @location(0) position: vec2<f32>,
};

struct PixelColor {
    @location(1) col: f32,
}

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) instance_idx: u32,
    @location(1) instance_col: f32,
};

@vertex
fn vs_main(
    @builtin(instance_index) instance_idx: u32,
    in_pixel: VertexInput,
    in_col : PixelColor
) -> VertexOutput 
{
    var out: VertexOutput;
    out.instance_idx = instance_idx;
    out.instance_col = in_col.col;
    // To get the horizontal and vertical offset we use bitshifts because they are more efficient,
    // not sure if the compiler optimizes
    out.clip_position = vec4<f32>(in_pixel.position.x + 0.03125*(f32(instance_idx & 63)), in_pixel.position.y - 0.0625*(f32(instance_idx >> 6)), 0.0, 1.0);
    return out;
}

// Fragment shader

@fragment
fn fs_main(
    in: VertexOutput
) -> @location(0) vec4<f32> 
{
    return vec4<f32>(in.instance_col, in.instance_col, in.instance_col, 1.0);
}
