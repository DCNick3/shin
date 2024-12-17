use std::fmt::Display;

use glam::{Mat3, Mat4, Vec3, Vec4, Vec4Swizzles as _};
use iced::{font, widget::column};
use iced_core::color;
use iced_widget::{text, Column, Space, Text};

#[derive(Debug, Copy, Clone)]
pub enum Matrix {
    Mat4(Mat4),
    Mat3(Mat3),
}

fn matrix_max_len<const C: usize>(matrix: impl AsRef<[f32; C]>) -> usize {
    matrix
        .as_ref()
        .iter()
        .cloned()
        .map(|v| format!("{:.8}", v).len())
        .max()
        .unwrap()
}

fn format_matrix_with_max_len(max_len: usize, iter: impl Iterator<Item = impl Display>) -> String {
    let mut res = "[ ".to_string();
    res.push_str(
        &iter
            .map(|v| format!("{:>width$.8}", v, width = max_len))
            .collect::<Vec<_>>()
            .join(" "),
    );
    res.push_str(" ]");
    res
}
fn matrix3_view_with_max_len<Message: 'static>(
    max_len: usize,
    matrix: Mat3,
) -> Column<'static, Message> {
    // transpose matrix to show the mathematical notation
    // (glam stores matrices in column-major order, like all CG libraries)
    let matrix = matrix.transpose();

    let format_row = |row: &Vec3| format_matrix_with_max_len(max_len, row.as_ref().iter().cloned());

    // TODO: add colors
    // - make header colored?
    // - make zeroes gray?
    let header = text(format_matrix_with_max_len(
        max_len,
        ["x", "y", "z", "w"].into_iter(),
    ))
    .font(iced::Font {
        weight: font::Weight::Bold,
        ..iced::Font::MONOSPACE
    });
    let body = text(format!(
        "{}\n{}\n{}",
        format_row(&matrix.x_axis),
        format_row(&matrix.y_axis),
        format_row(&matrix.z_axis),
    ))
    .font(iced::Font {
        weight: font::Weight::Bold,
        ..iced::Font::MONOSPACE
    });

    column!(header, Space::with_height(2), body)
}
fn matrix4_view_with_max_len<Message: 'static>(
    max_len: usize,
    matrix: Mat4,
) -> Column<'static, Message> {
    // transpose matrix to show the mathematical notation
    // (glam stores matrices in column-major order, like all CG libraries)
    let matrix = matrix.transpose();

    let format_row = |row: &Vec4| format_matrix_with_max_len(max_len, row.as_ref().iter().cloned());

    // TODO: add colors
    // - make header colored?
    // - make zeroes gray?
    let header = text(format_matrix_with_max_len(
        max_len,
        ["x", "y", "z", "w"].into_iter(),
    ))
    .font(iced::Font {
        weight: font::Weight::Bold,
        ..iced::Font::MONOSPACE
    });
    let body = text(format!(
        "{}\n{}\n{}\n{}",
        format_row(&matrix.x_axis),
        format_row(&matrix.y_axis),
        format_row(&matrix.z_axis),
        format_row(&matrix.w_axis),
    ))
    .font(iced::Font {
        weight: font::Weight::Bold,
        ..iced::Font::MONOSPACE
    });

    column!(header, Space::with_height(2), body)
}

fn matrix3_view<Message: 'static>(matrix: Mat3) -> Column<'static, Message> {
    let max_len = matrix_max_len(matrix);

    matrix3_view_with_max_len(max_len, matrix)
}

fn matrix4_view<Message: 'static>(matrix: Mat4) -> Column<'static, Message> {
    let max_len = matrix_max_len(matrix);

    matrix4_view_with_max_len(max_len, matrix)
}

fn matrix_view<Message: 'static>(matrix: Matrix) -> Column<'static, Message> {
    match matrix {
        Matrix::Mat3(matrix) => matrix3_view(matrix),
        Matrix::Mat4(matrix) => matrix4_view(matrix),
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum DecompositionOrder {
    #[default]
    RST,
    TRS,
}

impl Display for DecompositionOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DecompositionOrder::RST => write!(f, "R * S * T * x"),
            DecompositionOrder::TRS => write!(f, "T * R * S * x"),
        }
    }
}

pub struct MatrixDecomposition {
    order: DecompositionOrder,
    rotation: Mat3,
    scale: Vec3,
    translation: Vec3,
}

impl MatrixDecomposition {
    pub fn try_decompose(matrix: Mat4, order: DecompositionOrder) -> Option<Self> {
        assert!(matches!(
            order,
            DecompositionOrder::RST | DecompositionOrder::TRS
        ));

        if matrix
            .transpose()
            .w_axis
            .distance(Vec4::new(0.0, 0.0, 0.0, 1.0))
            > 0.0001
        {
            return None;
        }

        let post_translation = matrix.w_axis.xyz();
        let other = Mat4::from_cols(
            matrix.x_axis,
            matrix.y_axis,
            matrix.z_axis,
            Vec4::new(0.0, 0.0, 0.0, 1.0),
        );

        let pre_translation = other.inverse() * matrix;
        let pre_translation = pre_translation.w_axis.truncate();

        let qr = faer::mat![
            [matrix.x_axis.x, matrix.y_axis.x, matrix.z_axis.x],
            [matrix.x_axis.y, matrix.y_axis.y, matrix.z_axis.y],
            [matrix.x_axis.z, matrix.y_axis.z, matrix.z_axis.z]
        ]
        .qr();

        let rotation_matrix = qr.compute_q();
        let scale_matrix = qr.compute_r();

        // convert faer matrix to glam matrix
        // TODO: decompose the rotation into quaternion or euler angles
        let rotation = Mat3::from_cols_array(&[
            *rotation_matrix.get(0, 0),
            *rotation_matrix.get(1, 0),
            *rotation_matrix.get(2, 0),
            *rotation_matrix.get(0, 1),
            *rotation_matrix.get(1, 1),
            *rotation_matrix.get(2, 1),
            *rotation_matrix.get(0, 2),
            *rotation_matrix.get(1, 2),
            *rotation_matrix.get(2, 2),
        ]);

        // check if scale matrix is diagonal
        for i in 0..3 {
            for j in 0..3 {
                if i != j && scale_matrix.get(i, j).abs() > 0.0001 {
                    return None;
                }
            }
        }
        let scale = scale_matrix.diagonal().column_vector();
        let scale = Vec3::new(*scale.get(0), *scale.get(1), *scale.get(2));

        // convert post-translation to pre-translation
        let translation = match order {
            DecompositionOrder::TRS => post_translation,
            DecompositionOrder::RST => pre_translation,
        };

        // test if the results match
        let recomposed = match order {
            DecompositionOrder::TRS => {
                Mat4::from_translation(translation)
                    * Mat4::from_mat3(rotation)
                    * Mat4::from_scale(scale)
            }
            DecompositionOrder::RST => {
                Mat4::from_mat3(rotation)
                    * Mat4::from_scale(scale)
                    * Mat4::from_translation(translation)
            }
        };

        if (matrix - recomposed)
            .abs()
            .as_ref()
            .iter()
            .cloned()
            .max_by_key(|&f| float_ord::FloatOrd(f))
            .unwrap()
            > 0.0001
        {
            eprintln!("Decomposition failed: recomposed matrix does not match the original matrix");
        }

        Some(Self {
            order,
            translation,
            rotation,
            scale,
        })
    }

    pub fn view<Message: 'static>(&self) -> Column<'static, Message> {
        let translation = Mat4::from_translation(self.translation);
        let rotation = Mat4::from_mat3(self.rotation);
        let scale = Mat4::from_scale(self.scale);

        let max_len = matrix_max_len(translation)
            .max(matrix_max_len(rotation))
            .max(matrix_max_len(scale));

        let mut res = column!();

        match self.order {
            DecompositionOrder::TRS => {
                res = res.extend([
                    text(format!("Translation: {:.3}", self.translation)).into(),
                    matrix4_view_with_max_len(max_len, translation).into(),
                    Space::with_height(10).into(),
                    text("Rotation:").into(),
                    matrix4_view_with_max_len(max_len, rotation).into(),
                    Space::with_height(10).into(),
                    text(format!(
                        "Scale: {:.3} (1 / {:.3})",
                        self.scale,
                        1.0 / self.scale
                    ))
                    .into(),
                    matrix4_view_with_max_len(max_len, scale).into(),
                ]);
            }
            DecompositionOrder::RST => {
                res = res.extend([
                    text("Rotation:").into(),
                    matrix4_view_with_max_len(max_len, rotation).into(),
                    Space::with_height(10).into(),
                    text(format!(
                        "Scale: {:.3} (1 / {:.3})",
                        self.scale,
                        1.0 / self.scale
                    ))
                    .into(),
                    matrix4_view_with_max_len(max_len, scale).into(),
                    Space::with_height(10).into(),
                    text(format!("Translation: {:.3}", self.translation)).into(),
                    matrix4_view_with_max_len(max_len, translation).into(),
                ]);
            }
        }

        res
    }
}

pub enum MatrixParseResult {
    Valid,
    PartiallyValid,
}

impl MatrixParseResult {
    pub fn view(&self) -> Text<'static> {
        match self {
            MatrixParseResult::Valid => text("Valid matrix").color(color!(0x00c000)),
            MatrixParseResult::PartiallyValid => text("Invalid matrix!").color(color!(0xc00000)),
        }
    }
}

pub struct MatrixParseState {
    pub matrix: Matrix,
    pub result: MatrixParseResult,
    pub decomposition: Option<MatrixDecomposition>,
}

impl MatrixParseState {
    pub fn parse(floats: &[f32], decomposition_order: DecompositionOrder) -> Self {
        // 3x3 matrix is the rare case, so detect it by length  only exactly
        let (matrix, result) = if floats.len() == 9 {
            let mut floats_array = [0.0; 9];
            let size = std::cmp::min(floats.len(), floats_array.len());
            floats_array[..size].copy_from_slice(&floats[..size]);

            let matrix = Mat3::from_cols_array(&floats_array);
            let result = if floats.len() == floats_array.len() {
                MatrixParseResult::Valid
            } else {
                MatrixParseResult::PartiallyValid
            };

            (Matrix::Mat3(matrix), result)
        } else {
            let mut floats_array = [0.0; 16];
            let size = std::cmp::min(floats.len(), floats_array.len());
            floats_array[..size].copy_from_slice(&floats[..size]);

            let matrix = Mat4::from_cols_array(&floats_array);
            let result = if floats.len() == floats_array.len() {
                MatrixParseResult::Valid
            } else {
                MatrixParseResult::PartiallyValid
            };

            (Matrix::Mat4(matrix), result)
        };

        let decomposition = if let Matrix::Mat4(mat) = matrix {
            MatrixDecomposition::try_decompose(mat, decomposition_order)
        } else {
            None
        };

        Self {
            matrix,
            result,
            decomposition,
        }
    }

    pub fn matrix_view<Message: 'static>(&self) -> Column<'static, Message> {
        matrix_view(self.matrix)
    }

    pub fn decomposition_view<Message: 'static>(&self) -> Column<'static, Message> {
        match &self.decomposition {
            Some(decomposition) => decomposition.view(),
            None => column!(text("Decomposition failed!").color(color!(0xc00000))),
        }
    }
}
