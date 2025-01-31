use std::fmt::Display;

#[derive(Debug, Clone, Copy)]
pub enum VertexFieldType {
    #[expect(unused)]
    Float,
    Vector2,
    Vector3,
    Vector4,
    UintColor,
}

impl VertexFieldType {
    pub fn float_count(&self) -> u32 {
        match self {
            VertexFieldType::Float => 1,
            VertexFieldType::Vector2 => 2,
            VertexFieldType::Vector3 => 3,
            VertexFieldType::Vector4 => 4,
            VertexFieldType::UintColor => 1,
        }
    }
}

pub struct FieldValueIntoIter {
    array: [VertexPrimitiveValue; 4],
    pos: usize,
}

impl Iterator for FieldValueIntoIter {
    type Item = VertexPrimitiveValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos == self.array.len() {
            None
        } else {
            let result = self.array[self.pos];
            self.pos += 1;

            Some(result)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.array.len() - self.pos;
        (len, Some(len))
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VertexPrimitiveValue {
    Float(f32),
    UintColor(u32),
}

impl Display for VertexPrimitiveValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VertexPrimitiveValue::Float(v) => v.fmt(f),
            VertexPrimitiveValue::UintColor(v) => write!(f, "{:08x}", v),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VertexFieldValue {
    Float(f32),
    Vector2([f32; 2]),
    Vector3([f32; 3]),
    Vector4([f32; 4]),
    UintColor(u32),
}

impl IntoIterator for VertexFieldValue {
    type Item = VertexPrimitiveValue;
    type IntoIter = FieldValueIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let (array, pos) = match self {
            VertexFieldValue::Float(s) => (
                [
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(s),
                ],
                3,
            ),
            VertexFieldValue::Vector2(a) => (
                [
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(a[0]),
                    VertexPrimitiveValue::Float(a[1]),
                ],
                2,
            ),
            VertexFieldValue::Vector3(a) => (
                [
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(a[0]),
                    VertexPrimitiveValue::Float(a[1]),
                    VertexPrimitiveValue::Float(a[2]),
                ],
                1,
            ),
            VertexFieldValue::Vector4(a) => (
                [
                    VertexPrimitiveValue::Float(a[0]),
                    VertexPrimitiveValue::Float(a[1]),
                    VertexPrimitiveValue::Float(a[2]),
                    VertexPrimitiveValue::Float(a[3]),
                ],
                0,
            ),
            VertexFieldValue::UintColor(v) => (
                [
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::Float(0.0),
                    VertexPrimitiveValue::UintColor(v),
                ],
                3,
            ),
        };
        FieldValueIntoIter { array, pos }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct VertexFieldDescriptor {
    pub name: &'static str,
    pub ty: VertexFieldType,
}

#[derive(Debug, Clone, Copy)]
pub struct VertexDescriptor {
    pub name: &'static str,
    pub fields: &'static [VertexFieldDescriptor],
    #[expect(unused)]
    pub position_visualizations: &'static [&'static str],
}

impl VertexDescriptor {
    pub fn float_count_per_vertex(&self) -> u32 {
        self.fields.iter().map(|fd| fd.ty.float_count()).sum()
    }

    pub fn column_names(&self) -> Vec<String> {
        let mut result = Vec::new();

        for field in self.fields {
            let subfields: &[&str] = match field.ty {
                VertexFieldType::Float | VertexFieldType::UintColor => {
                    result.push(field.name.to_string());
                    continue;
                }
                VertexFieldType::Vector2 => &["x", "y"],
                VertexFieldType::Vector3 => &["x", "y", "z"],
                VertexFieldType::Vector4 => &["x", "y", "z", "w"],
            };

            for subfield in subfields {
                result.push(format!("{}.{}", field.name, subfield));
            }
        }

        result
    }
}

impl Display for VertexDescriptor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.name.fmt(f)
    }
}

pub const VERTEX_DESCRTIPTORS: &[VertexDescriptor] = &[
    VertexDescriptor {
        name: "Pos3",
        fields: &[VertexFieldDescriptor {
            name: "pos",
            ty: VertexFieldType::Vector3,
        }],
        position_visualizations: &["pos"],
    },
    VertexDescriptor {
        name: "Pos4",
        fields: &[VertexFieldDescriptor {
            name: "pos",
            ty: VertexFieldType::Vector4,
        }],
        position_visualizations: &["pos"],
    },
    VertexDescriptor {
        name: "Pos2Tex2",
        fields: &[
            VertexFieldDescriptor {
                name: "pos",
                ty: VertexFieldType::Vector2,
            },
            VertexFieldDescriptor {
                name: "tex",
                ty: VertexFieldType::Vector2,
            },
        ],
        position_visualizations: &["pos", "tex"],
    },
    VertexDescriptor {
        name: "Sprite (Pos3ColUTex2)",
        fields: &[
            VertexFieldDescriptor {
                name: "pos",
                ty: VertexFieldType::Vector3,
            },
            VertexFieldDescriptor {
                name: "col",
                ty: VertexFieldType::UintColor,
            },
            VertexFieldDescriptor {
                name: "tex",
                ty: VertexFieldType::Vector2,
            },
        ],
        position_visualizations: &["pos", "tex"],
    },
    VertexDescriptor {
        name: "Text (Pos2Tex2Col)",
        fields: &[
            VertexFieldDescriptor {
                name: "pos",
                ty: VertexFieldType::Vector2,
            },
            VertexFieldDescriptor {
                name: "tex",
                ty: VertexFieldType::Vector2,
            },
            VertexFieldDescriptor {
                name: "col",
                ty: VertexFieldType::Float,
            },
        ],
        position_visualizations: &["pos", "tex"],
    },
];
