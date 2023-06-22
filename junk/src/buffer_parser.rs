use binrw::BinRead;
use derive_more::Add;
use once_cell::sync::Lazy;
use regex::Regex;
use std::borrow::Cow;

static BUFFER_FLOAT_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[a-zA-Z_]\[(\d+)]\s*=\s*([0-9.\-]+);").unwrap());

static BUFFER_UNDEFINED8_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"[a-zA-Z_]._(\d+)_8_ = 0x([0-9a-f]{16});").unwrap());

pub struct Buffer {
    values: Vec<Option<u8>>,
}

impl Buffer {
    pub fn new(size: usize) -> Self {
        Self {
            values: vec![None; size],
        }
    }

    pub fn set(&mut self, offset: usize, value: u8) {
        self.values[offset] = Some(value);
    }

    pub fn set_float(&mut self, offset: usize, value: f32) {
        let bytes = value.to_le_bytes();
        self.set(offset, bytes[0]);
        self.set(offset + 1, bytes[1]);
        self.set(offset + 2, bytes[2]);
        self.set(offset + 3, bytes[3]);
    }

    pub fn set_undefined8(&mut self, offset: usize, value: u64) {
        let bytes = value.to_le_bytes();
        self.set(offset, bytes[0]);
        self.set(offset + 1, bytes[1]);
        self.set(offset + 2, bytes[2]);
        self.set(offset + 3, bytes[3]);
        self.set(offset + 4, bytes[4]);
        self.set(offset + 5, bytes[5]);
        self.set(offset + 6, bytes[6]);
        self.set(offset + 7, bytes[7]);
    }

    #[allow(dead_code)]
    pub fn dump(&self) -> String {
        let mut result = String::new();

        for line in self.values.chunks(16) {
            for byte in line {
                result.push_str(
                    &byte.map_or("?? ".into(), |byte| Cow::Owned(format!("{:02x} ", byte))),
                );
            }
            result.push('\n');
        }

        result
    }

    pub fn into_vec(self) -> Vec<u8> {
        self.values
            .into_iter()
            .enumerate()
            .map(|(i, v)| {
                if let Some(v) = v {
                    v
                } else {
                    panic!("Buffer value at offset {} is undefined", i);
                }
            })
            .collect()
    }
}

pub fn parse_buffer(buffer_size: usize, text: &str) -> Buffer {
    let mut buffer = Buffer::new(buffer_size);

    for line in text.lines().map(|v| v.trim()).filter(|v| !v.is_empty()) {
        if let Some(captures) = BUFFER_FLOAT_REGEX.captures(line) {
            let index = captures[1].parse::<usize>().unwrap();
            let value = captures[2].parse::<f32>().unwrap();
            buffer.set_float(index * 4, value);
        } else if let Some(captures) = BUFFER_UNDEFINED8_REGEX.captures(line) {
            let offset = captures[1].parse::<usize>().unwrap();
            let value = u64::from_str_radix(&captures[2], 16).unwrap();
            buffer.set_undefined8(offset, value);
        } else {
            panic!("Unknown line: {}", line);
        }
    }

    buffer
}

#[derive(BinRead, Debug, Copy, Clone)]
struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(BinRead, Add, Debug, Copy, Clone)]
struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[allow(dead_code)] // this code is not dead!
#[derive(BinRead, Debug, Copy, Clone)]
struct SpriteVertex {
    pub pos: Vector3,
    pub color: u32,
    pub tex_pos: Vector2,
}

#[derive(BinRead, Debug)]
#[brw(little)]
struct SpriteVertices {
    #[br(parse_with = binrw::until_eof)]
    pub vertices: Vec<SpriteVertex>,
}

fn make_svg(triangle_strip: Vec<Vector2>, view_box: (f32, f32, f32, f32)) -> String {
    use std::fmt::Write;

    let mut result = String::new();
    assert!(triangle_strip.len() >= 3);

    writeln!(
        result,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}">"#,
        view_box.0,
        view_box.1,
        view_box.2 - view_box.0,
        view_box.3 - view_box.1
    )
    .unwrap();

    let mut line = |from: Vector2, to: Vector2| {
        writeln!(
            result,
            r#"  <line x1="{}" y1="{}" x2="{}" y2="{}" stroke="black" />"#,
            from.x, from.y, to.x, to.y
        )
        .unwrap();
    };

    for triangle in triangle_strip.windows(3) {
        if let &[a, b, c] = triangle {
            line(a, b);
            line(b, c);
            line(c, a);
        } else {
            unreachable!()
        }
    }

    result.push_str("</svg>\n");

    result
}

fn dump_sprite_vertex_buffer(
    text: &str,
    size: usize,
    index_buffer: Option<&[u16]>,
    tex_size: Vector2,
    translation: Vector2,
    filename_base: &str,
) {
    let buffer = parse_buffer(size, text);

    // println!("{}", buffer.dump());

    let buffer = buffer.into_vec();

    let vertices = SpriteVertices::read(&mut std::io::Cursor::new(&buffer)).unwrap();

    // let tex_size = Vector2 {
    //     x: 1648.0,
    //     y: 288.0,
    // };

    // print legend
    println!(
        "[{:5} {:5} {:3}] [{:5} {:5}]",
        "x", "y", "z", "tex_x", "tex_y"
    );

    fn print_vertex(vertex: &SpriteVertex, tex_size: &Vector2) {
        println!(
            "[{:5} {:5} {:3}] [{:5} {:5}]",
            vertex.pos.x,
            vertex.pos.y,
            vertex.pos.z,
            (vertex.tex_pos.x * tex_size.x).round(),
            (vertex.tex_pos.y * tex_size.y).round()
        );
    }

    let vertices = if let Some(index_buffer) = index_buffer {
        index_buffer
            .iter()
            .map(|&i| vertices.vertices[i as usize])
            .collect::<Vec<_>>()
    } else {
        vertices.vertices
    };

    for vertex in vertices.iter() {
        print_vertex(vertex, &tex_size);
    }

    let screen_svg = make_svg(
        vertices
            .iter()
            .map(|v| {
                Vector2 {
                    x: v.pos.x,
                    y: v.pos.y,
                } + translation
            })
            .collect::<Vec<_>>(),
        // (-960.0, -540.0, 960.0, 540.0),
        (0.0, 0.0, 1920.0, 1080.0),
    );

    let tex_svg = make_svg(
        vertices
            .iter()
            .map(|v| Vector2 {
                x: v.tex_pos.x * tex_size.x,
                y: v.tex_pos.y * tex_size.y,
            })
            .collect::<Vec<_>>(),
        (0.0, 0.0, tex_size.x, tex_size.y),
    );

    std::fs::write(format!("{}_screen.svg", filename_base), screen_svg).unwrap();
    std::fs::write(format!("{}_tex.svg", filename_base), tex_svg).unwrap();
}

pub fn main() {
    // vertices for messagebox header
    let text = r#"
        buffer[3] = -107374176.0;
        buffer[9] = -107374176.0;
        buffer[15] = -107374176.0;
        buffer[21] = -107374176.0;
        buffer[27] = -107374176.0;
        buffer[33] = -107374176.0;
        buffer[39] = -107374176.0;
        buffer[45] = -107374176.0;
    
        buffer[0] = 130.0;
        buffer[1] = -32.0;
        buffer[6] = 130.0;
        buffer[7] = 80.0;
        buffer._40_8_ = 0x3f638e3900000000;
        buffer[12] = 178.0;
        buffer[13] = -32.0;
        buffer[18] = 178.0;
        buffer[19] = 80.0;
        buffer._16_8_ = 0x3f00000000000000;
        buffer._64_8_ = 0x3f0000003cee9a19;
        buffer[24] = 1742.0;
        buffer[25] = -32.0;
        buffer._88_8_ = 0x3f638e393cee9a19;
        buffer[2] = 1.0;
        buffer[8] = 1.0;
        buffer[14] = 1.0;
        buffer[20] = 1.0;
        buffer[26] = 1.0;
        buffer._112_8_ = 0x3f0000003d1f1166;
        buffer[30] = 1742.0;
        buffer[31] = 80.0;
        buffer[32] = 1.0;
        buffer._136_8_ = 0x3f638e393d1f1166;
        buffer[36] = 1790.0;
        buffer[37] = -32.0;
        buffer[38] = 1.0;
        buffer._160_8_ = 0x3f0000003d8b2f39;
        buffer[42] = 1790.0;
        buffer[43] = 80.0;
        buffer[44] = 1.0;
        buffer._184_8_ = 0x3f638e393d8b2f39;
    "#;

    let tex_size = Vector2 {
        x: 1648.0,
        y: 288.0,
    };

    let translation = Vector2 {
        x: 0.0,
        y: 1080.0 - 1024.0,
    };

    println!("Messagebox header:");
    dump_sprite_vertex_buffer(text, 0xc0, None, tex_size, translation, "messagebox_header");
    println!();

    // messagebox body vertices
    let text = r#"
      buffer[18] = 1790.0;
      buffer[19] = 80.0;
      buffer._16_8_ = 0x3d638e393e152050;
      buffer._40_8_ = 0x3d638e393e32f393;
      buffer[12] = 446.0;
      buffer[13] = 80.0;
      buffer[25] = 768.0;
      buffer._64_8_ = 0x3d638e393e3ce4a9;
      buffer._88_8_ = 0x3d638e393f800000;
      buffer[0] = 130.0;
      buffer[1] = 80.0;
      buffer[2] = 1.0;
      buffer[6] = 178.0;
      buffer[7] = 80.0;
      buffer[8] = 1.0;
      buffer[14] = 1.0;
      buffer[20] = 1.0;
      buffer[24] = 130.0;
      buffer[26] = 1.0;
      buffer._112_8_ = 0x3de38e393e152050;
      buffer[30] = 178.0;
      buffer[32] = 1.0;
      buffer._136_8_ = 0x3de38e393e32f393;
      buffer[36] = 446.0;
      buffer[38] = 1.0;
      buffer._160_8_ = 0x3de38e393e3ce4a9;
      buffer[42] = 1790.0;
      buffer[44] = 1.0;
      buffer._184_8_ = 0x3de38e393f800000;
      buffer[48] = 130.0;
      buffer[50] = 1.0;
      buffer._208_8_ = 0x3f8000003e152050;
      buffer[54] = 178.0;
      buffer[56] = 1.0;
      buffer._232_8_ = 0x3f8000003e32f393;
      buffer[60] = 446.0;
      buffer[62] = 1.0;
      buffer._256_8_ = 0x3f8000003e3ce4a9;
      buffer[68] = 1.0;
      buffer[66] = 1790.0;
      buffer._280_8_ = 0x3f8000003f800000;
      buffer[3] = -107374176.0;
      buffer[9] = -107374176.0;
      buffer[15] = -107374176.0;
      buffer[21] = -107374176.0;
      buffer[27] = -107374176.0;
      buffer[31] = 768.0;
      buffer[33] = -107374176.0;
      buffer[37] = 768.0;
      buffer[39] = -107374176.0;
      buffer[43] = 768.0;
      buffer[45] = -107374176.0;
      buffer[49] = 1024.0;
      buffer[51] = -107374176.0;
      buffer[55] = 1024.0;
      buffer[57] = -107374176.0;
      buffer[61] = 1024.0;
      buffer[63] = -107374176.0;
      buffer[67] = 1024.0;
      buffer[69] = -107374176.0;
    "#;

    println!("Messagebox body:");
    dump_sprite_vertex_buffer(
        text,
        0x120,
        Some(&[0, 4, 1, 5, 2, 6, 3, 7, 11, 6, 10, 5, 9, 4, 8]),
        tex_size,
        translation,
        "messagebox_body",
    );
}
