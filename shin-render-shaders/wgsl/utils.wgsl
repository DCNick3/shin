fn map(value: f32, minmax: vec2<f32>) -> f32 {
    let min = minmax.x;
    let max = minmax.y;

	return min + value * (max - min);
}
