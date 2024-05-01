use shin_core::format::mask::MaskVertex;

fn make_svg(vertices: &[MaskVertex], color: &str, view_box: (f32, f32, f32, f32)) -> String {
    use std::fmt::Write;

    let mut result = String::new();
    writeln!(
        result,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">"#,
        view_box.0,
        view_box.1,
        view_box.2 - view_box.0,
        view_box.3 - view_box.1
    )
    .unwrap();

    let mut rect = |x: f32, y: f32, width: f32, height: f32| {
        writeln!(
            result,
            r#"  <rect x="{}" y="{}" width="{}" height="{}" fill="none" stroke="{}" />"#,
            x, y, width, height, color
        )
        .unwrap();
    };

    for vertex in vertices {
        rect(
            vertex.from_x as f32,
            vertex.from_y as f32,
            vertex.to_x as f32 - vertex.from_x as f32,
            vertex.to_y as f32 - vertex.from_y as f32,
        )
    }

    result.push_str("</svg>\n");

    result
}

fn compute_area(vertices: &[MaskVertex]) -> u32 {
    vertices
        .iter()
        .map(|v| (v.to_x - v.from_x) as u32 * (v.to_y - v.from_y) as u32)
        .sum()
}

pub fn main(msk_path: String) {
    let mask = std::fs::read(msk_path).unwrap();
    let mask = shin_core::format::mask::read_mask(&mask).unwrap();

    let black_range = 0..mask.vertices.black_regions.vertex_count as usize;
    let white_range =
        black_range.end..black_range.end + mask.vertices.white_regions.vertex_count as usize;
    let transparent_range =
        white_range.end..white_range.end + mask.vertices.transparent_regions.vertex_count as usize;

    assert!(mask
        .vertices
        .vertices
        .iter()
        .all(|v| v.from_x % 4 == 0 && v.from_y % 4 == 0 && v.to_x % 4 == 0 && v.to_y % 4 == 0));

    let black_vertices = &mask.vertices.vertices[black_range];
    let white_vertices = &mask.vertices.vertices[white_range];
    let transparent_vertices = &mask.vertices.vertices[transparent_range];

    // the areas are computed in 4x4 blocks
    println!("Black area: {}", compute_area(black_vertices) / 16);
    println!("White area: {}", compute_area(white_vertices) / 16);
    println!(
        "Transparent area: {}",
        compute_area(transparent_vertices) / 16
    );

    let black_svg = make_svg(
        black_vertices,
        "red",
        (
            0.0,
            0.0,
            mask.texels.width() as f32,
            mask.texels.height() as f32,
        ),
    );
    let white_svg = make_svg(
        &white_vertices,
        "green",
        (
            0.0,
            0.0,
            mask.texels.width() as f32,
            mask.texels.height() as f32,
        ),
    );
    let transparent_svg = make_svg(
        &transparent_vertices,
        "blue",
        (
            0.0,
            0.0,
            mask.texels.width() as f32,
            mask.texels.height() as f32,
        ),
    );

    std::fs::write("mask_black.svg", black_svg).unwrap();
    std::fs::write("mask_white.svg", white_svg).unwrap();
    std::fs::write("mask_transparent.svg", transparent_svg).unwrap();
}
