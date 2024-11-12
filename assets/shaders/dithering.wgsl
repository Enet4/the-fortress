// This shader computes a form of ordered dithering.
// The R channel is computed separately from the G and B channels,
// yielding a 4-color dithered output.
// This is then intermixed with the original input
// based on the intensity setting.

// Since post processing is a fullscreen effect, we use the fullscreen vertex shader provided by bevy.
// This will import a vertex shader that renders a single fullscreen triangle.
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
struct PostProcessSettings {
    /// the intensity of the dithering effect (0 = no dithering, 1 = only dithered output)
    intensity: f32,
    oscillate: f32,
#ifdef SIXTEEN_BYTE_ALIGNMENT
    // WebGL2 structs must be 16 byte aligned.
    _webgl2_padding: vec3<f32>
#endif
}
@group(0) @binding(2) var<uniform> settings: PostProcessSettings;

const BAYER_MATRIX = array<f32, 64>(
    0f, 32, 8,  40, 2,  34, 10, 42,
    48, 16, 56, 24, 50, 18, 58, 26,
    12, 44, 4,  36, 14, 46, 6,  38,
    60, 28, 52, 20, 62, 30, 54, 22,
    3,  35, 11, 43, 1,  33, 9,  41,
    51, 19, 59, 27, 49, 17, 57, 25,
    15, 47, 7,  39, 13, 45, 5,  37,
    63, 31, 55, 23, 61, 29, 53, 21
) / 64.f;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    var r = textureSample(screen_texture, texture_sampler, in.uv).r;
    var g = textureSample(screen_texture, texture_sampler, in.uv).g;
    var b = textureSample(screen_texture, texture_sampler, in.uv).b;

    let intensity = settings.intensity;
    // bypass dithering
    if intensity == 0.0 {
        return vec4<f32>(
            r, g, b, 1.0
        );
    }

    // average intensity
    let v = (r + g + b) / 3.0;

    // get the bayer matrix index
    let x = i32(in.position.x) % 8;
    let y = i32(in.position.y) % 8;
    let i = y * 8 + x;

    // define the color filtering array
    var bayerMatrix = BAYER_MATRIX;

    // get the threshold
    let threshold: f32 = bayerMatrix[i];

    let inv_intensity = 1.0 - intensity;
    if v > threshold {
        g = intensity + g * inv_intensity;
        b = intensity + b * inv_intensity;
    } else {
        g = g * inv_intensity;
        b = b * inv_intensity;
    }
    if r > threshold {
        r = intensity + r * inv_intensity;
    } else {
        r = r * inv_intensity;
    }

    // apply dithering
    return vec4<f32>(
        r, g, b, 1.0
    );
}
