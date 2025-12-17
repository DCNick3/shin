use super::BitBuffer;

pub fn write_u8(value: &u8, writer: &mut BitBuffer) {
    writer.write_int(8, *value);
}
pub fn write_i8(value: &i8, writer: &mut BitBuffer) {
    writer.write_int(8, *value);
}

pub fn write_u16(value: &u16, writer: &mut BitBuffer) {
    writer.write_int(16, *value);
}
pub fn write_i16(value: &i16, writer: &mut BitBuffer) {
    writer.write_int(16, *value);
}

pub fn write_u32(value: &u32, writer: &mut BitBuffer) {
    writer.write_int(32, *value);
}
pub fn write_i32(value: &i32, writer: &mut BitBuffer) {
    writer.write_int(32, *value);
}

pub fn write_vec<T, L: TryFrom<usize>>(
    value: &[T],
    writer: &mut BitBuffer,
    write_len: impl Fn(&L, &mut BitBuffer),
    write_item: impl Fn(&T, &mut BitBuffer),
) {
    let len = value
        .len()
        .try_into()
        .map_err(|_| ())
        .expect("failed to convert vector size from usize to the target type");

    write_len(&len, writer);
    for item in value {
        write_item(item, writer);
    }
}

pub fn write_array<T, const N: usize>(
    value: &[T; N],
    writer: &mut BitBuffer,
    write: impl Fn(&T, &mut BitBuffer),
) {
    for item in value {
        write(item, writer);
    }
}

pub fn write_opt<T>(value: &Option<T>, writer: &mut BitBuffer, write: impl Fn(&T, &mut BitBuffer)) {
    match value {
        None => {
            writer.write_bool(false);
        }
        Some(value) => {
            writer.write_bool(true);
            write(value, writer);
        }
    }
}
