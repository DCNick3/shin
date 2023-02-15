// we have this pipeline:
// 1. Read an mp4 sample from mp4 demuxer
// 2. (asyncronously) Decode the sample using openh264
// 3. Present the frame with wgpu
