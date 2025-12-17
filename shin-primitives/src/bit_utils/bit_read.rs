use super::{BitBuffer, BitReadError};

pub type Result<T> = std::result::Result<T, BitReadError>;

pub fn read_u8(reader: &mut BitBuffer) -> Result<u8> {
    reader.read_int(8)
}
pub fn read_i8(reader: &mut BitBuffer) -> Result<i8> {
    reader.read_int(8)
}

pub fn read_u16(reader: &mut BitBuffer) -> Result<u16> {
    reader.read_int(16)
}
pub fn read_i16(reader: &mut BitBuffer) -> Result<i16> {
    reader.read_int(16)
}

pub fn read_u32(reader: &mut BitBuffer) -> Result<u32> {
    reader.read_int(32)
}
pub fn read_i32(reader: &mut BitBuffer) -> Result<i32> {
    reader.read_int(32)
}

pub fn read_vec<T, L: TryInto<usize>>(
    reader: &mut BitBuffer,
    parse_len: impl Fn(&mut BitBuffer) -> Result<L>,
    parse_item: impl Fn(&mut BitBuffer) -> Result<T>,
) -> Result<Vec<T>> {
    let len = parse_len(reader)?
        .try_into()
        .map_err(|_| ())
        .expect("failed to convert the read length type into usize");
    let mut vec = Vec::with_capacity(len);
    for _ in 0..len {
        vec.push(parse_item(reader)?);
    }
    Ok(vec)
}

pub fn read_array<T, const N: usize>(
    reader: &mut BitBuffer,
    parse: impl Fn(&mut BitBuffer) -> Result<T>,
) -> Result<[T; N]> {
    let mut res = [(); N].map(|_| None);

    for res in res.iter_mut() {
        *res = Some(parse(reader)?);
    }

    Ok(res.map(|v| v.unwrap()))
}

pub fn read_opt<T>(
    reader: &mut BitBuffer,
    parse: impl Fn(&mut BitBuffer) -> Result<T>,
) -> Result<Option<T>> {
    let is_some = reader.read_bool()?;
    if is_some {
        Ok(Some(parse(reader)?))
    } else {
        Ok(None)
    }
}
