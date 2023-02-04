//! Implements "crypto" used by savefiles
//! (it's just a bunch of XORs, actually)

use itertools::Itertools;

fn decode_once(data: &mut [u8], key: u32) {
    #[inline]
    fn transform(data: &mut [u8; 4], key: &mut u32) {
        let word = u32::from_be_bytes(*data);
        // transform the data
        *data = (word ^ *key).to_be_bytes();

        // transform the key. Use the encrypted CRC!
        *key ^= super::crc32::crc32(&word.to_le_bytes(), 0)
    }

    let mut current_key = key;

    // I REALLY want array_chunks to be stable...
    for chunk_data in data.chunks_mut(4) {
        let chunk_data_len = chunk_data.len();
        let mut chunk = [0; 4];

        chunk[..chunk_data_len].copy_from_slice(chunk_data);
        transform(&mut chunk, &mut current_key);
        chunk_data.copy_from_slice(&chunk[..chunk_data_len]);
    }
}

#[cfg(test)]
mod test {
    use insta::assert_debug_snapshot;
    use rand::{Rng, RngCore, SeedableRng};

    fn decode_once(data: &str, key: u32) -> String {
        let mut data = hex::decode(data).unwrap();
        super::decode_once(&mut data, key);
        hex::encode(data)
    }

    #[test]
    fn test_decode_once_simpl() {
        assert_debug_snapshot!(decode_once("0123456789abcdef", 0x1337), @r###""01235650b1e1c6a2""###);
    }

    #[test]
    fn test_decode_once_random() {
        // generate random test cases
        let mut rng = rand::rngs::StdRng::seed_from_u64(0x42);

        for _ in 0..100 {
            let len = rng.gen_range(1..20);
            let data = (0..len).map(|_| rng.gen::<u8>()).collect::<Vec<_>>();
            let data = hex::encode(data);

            let key = rng.next_u32();
            assert_debug_snapshot!(format!("decode_once_random_{}", data), decode_once(&data, key));
        }
    }
}
