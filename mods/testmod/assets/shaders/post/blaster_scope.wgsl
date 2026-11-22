fn apply(p: PostPixel) -> vec4<f32> {
    let point = (p.uv - vec2(0.5)) * p.size;
    let radius = min(p.size.x, p.size.y) * 0.43;
    let distance = length(point);
    let lens = 1.0 - smoothstep(radius - 1.0, radius + 1.0, distance);
    let rim = 1.0 - smoothstep(radius - 5.0, radius - 3.0, distance);
    let axis = min(abs(point.x), abs(point.y));
    let along = max(abs(point.x), abs(point.y));
    let line = (1.0 - smoothstep(0.6, 1.6, axis)) * smoothstep(7.0, 9.0, along);
    let color = mix(p.color.rgb * rim, vec3(0.01), line * 0.8);
    return vec4(mix(p.color.rgb, color * lens, p.strength), p.color.a);
}
