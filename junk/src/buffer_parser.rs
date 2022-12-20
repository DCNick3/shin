use binrw::BinRead;
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
        self.values[offset] = Some(bytes[0]);
        self.values[offset + 1] = Some(bytes[1]);
        self.values[offset + 2] = Some(bytes[2]);
        self.values[offset + 3] = Some(bytes[3]);
    }

    pub fn set_undefined8(&mut self, offset: usize, value: u64) {
        let bytes = value.to_le_bytes();
        self.values[offset] = Some(bytes[0]);
        self.values[offset + 1] = Some(bytes[1]);
        self.values[offset + 2] = Some(bytes[2]);
        self.values[offset + 3] = Some(bytes[3]);
        self.values[offset + 4] = Some(bytes[4]);
        self.values[offset + 5] = Some(bytes[5]);
        self.values[offset + 6] = Some(bytes[6]);
        self.values[offset + 7] = Some(bytes[7]);
    }

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

#[derive(BinRead, Debug)]
struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}

#[derive(BinRead, Debug)]
struct Vector2 {
    pub x: f32,
    pub y: f32,
}

#[derive(BinRead, Debug)]
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

fn dump_sprite_vertex_buffer(text: &str, size: usize) {
    let buffer = parse_buffer(size, text);

    // println!("{}", buffer.dump());

    let buffer = buffer.into_vec();

    let vertices = SpriteVertices::read(&mut std::io::Cursor::new(&buffer)).unwrap();

    let tex_size = Vector2 {
        x: 1648.0,
        y: 288.0,
    };

    // print legend
    println!(
        "[{:5} {:5} {:3}] [{:5} {:5}]",
        "x", "y", "z", "tex_x", "tex_y"
    );
    for vertex in vertices.vertices {
        println!(
            "[{:5} {:5} {:3}] [{:5} {:5}]",
            vertex.pos.x,
            vertex.pos.y,
            vertex.pos.z,
            (vertex.tex_pos.x * tex_size.x).round(),
            (vertex.tex_pos.y * tex_size.y).round()
        );
    }
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

    println!("Messagebox header:");
    dump_sprite_vertex_buffer(text, 0xc0);
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
    dump_sprite_vertex_buffer(text, 0x120);
}
