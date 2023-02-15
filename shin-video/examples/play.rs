use shin_video::mp4::Mp4;
use std::fs::File;

fn main() {
    let file = File::open("ship1.mp4").unwrap();
    // let file = File::open("op1.mp4").unwrap();

    let mp4 = Mp4::new(file).unwrap();

    todo!()
}
