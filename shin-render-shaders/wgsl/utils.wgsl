fn map(value: f32, minmax: vec2<f32>) -> f32 {
    let min = minmax.x;
    let max = minmax.y;

	return min + value * (max - min);
}

fn evaluate_fragment_shader(color: vec3<f32>, operation: u32, param: vec4<f32>) -> vec3<f32> {
    if operation == 0 {
        // default
        return color;
    } else if operation == 1 {
        // mono
        let luma = dot(color, vec3(0.299, 0.587, 0.114));
        let mix = mix(color, vec3(luma), param.w);
        return mix * param.xyz;
    } else if operation == 2 {
        // fill
        return mix(color, param.xyz, param.w);
    } else if operation == 3 {
        // fill2
        // I have no idea how it is different from default
        return color;
    } else if operation == 4 {
        // negative
        let negated = 1 - color;
        return mix(color, negated, param.w);
    } else if operation == 5 {
        // gamma
        return exp2(log2(color) * 1 / param.xyz);
    } else {
        return color;
    }
}
