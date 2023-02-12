use ffmpeg_next as ffmpeg;
use ffmpeg_sys_next as sys;

use std::io::{Read, Seek, SeekFrom};
use tracing::warn;

trait ReadSeek: Read + Seek {}

impl<T: Read + Seek> ReadSeek for T {}

struct AvioReadContextInner {
    reader: Box<dyn ReadSeek + Send>,
}

unsafe extern "C" fn read_packet(
    opaque: *mut std::ffi::c_void,
    buf: *mut u8,
    buf_size: i32,
) -> i32 {
    let inner = opaque as *mut AvioReadContextInner;
    let inner = &mut *inner;
    let reader = &mut *inner.reader;

    let buf = std::slice::from_raw_parts_mut(buf, buf_size as usize);

    match reader.read(buf) {
        Ok(0) => sys::AVERROR_EOF,
        Ok(n) => n.try_into().unwrap(),
        Err(e) => {
            warn!("AvioReadContext: read error: {}", e);
            -1
        }
    }
}

fn stream_len(reader: &mut (impl Seek + ?Sized)) -> std::io::Result<u64> {
    let old_pos = reader.stream_position()?;
    let len = reader.seek(SeekFrom::End(0))?;

    // Avoid seeking a third time when we were already at the end of the
    // stream. The branch is usually way cheaper than a seek operation.
    if old_pos != len {
        reader.seek(SeekFrom::Start(old_pos))?;
    }

    Ok(len)
}

unsafe extern "C" fn seek(opaque: *mut std::ffi::c_void, offset: i64, whence: i32) -> i64 {
    let inner = opaque as *mut AvioReadContextInner;
    let inner = &mut *inner;
    let reader = &mut *inner.reader;

    let whence = match whence {
        sys::SEEK_SET => SeekFrom::Start(offset.try_into().unwrap()),
        sys::SEEK_CUR => SeekFrom::Current(offset),
        sys::SEEK_END => SeekFrom::End(offset),
        sys::AVSEEK_SIZE => {
            return stream_len(reader)
                .map(|v| v.try_into().unwrap())
                .unwrap_or(-1)
        }
        _ => panic!("invalid whence: {}", whence),
    };

    match reader.seek(whence) {
        Ok(n) => n.try_into().unwrap(),
        Err(e) => {
            warn!("AvioReadContext: seek error: {}", e);
            -1
        }
    }
}

pub struct AvioReadContext {
    ctx: *mut sys::AVIOContext,
}

impl AvioReadContext {
    pub fn new(inner: impl Read + Seek + Send + 'static, buffer_size: usize) -> Self {
        let inner: Box<dyn ReadSeek + Send> = Box::new(inner);
        let inner = AvioReadContextInner { reader: inner };

        let buffer = unsafe { sys::av_malloc(buffer_size) } as *mut u8;
        if buffer.is_null() {
            panic!("av_malloc for buffer failed");
        }

        // yes, we do need two pointers
        // that's because *mut dyn ReadSeek + Send is actually a fat pointer, we can't just cast it to *mut c_void
        let inner =
            Box::leak(Box::new(inner)) as *mut AvioReadContextInner as *mut std::ffi::c_void;

        let avio_context = unsafe {
            sys::avio_alloc_context(
                buffer,
                buffer_size.try_into().unwrap(),
                0, // Buffer is only readable - set to 1 for read/write
                inner,
                Some(read_packet),
                None,
                Some(seek),
            )
        };

        if avio_context.is_null() {
            // TODO: we have memory leaks here
            panic!("avio_alloc_context failed");
        }

        Self { ctx: avio_context }
    }
}

impl Drop for AvioReadContext {
    fn drop(&mut self) {
        unsafe {
            sys::av_free((*self.ctx).buffer as *mut std::ffi::c_void);
            std::mem::drop(Box::from_raw(
                (*self.ctx).opaque as *mut AvioReadContextInner,
            ));
            sys::av_free(self.ctx as *mut std::ffi::c_void);
        }
    }
}

/// # Safety
///
/// Do not drop the returned AvioReadContext until the returned format context is dropped.
pub unsafe fn input_reader(
    reader: impl Read + Seek + Send + 'static,
) -> Result<(ffmpeg::format::context::Input, AvioReadContext), ffmpeg::Error> {
    let avio_context = AvioReadContext::new(reader, 8192);

    let mut ctx = unsafe { sys::avformat_alloc_context() };
    if ctx.is_null() {
        panic!("avformat_alloc_context failed");
    }

    unsafe {
        (*ctx).flags |= sys::AVFMT_FLAG_CUSTOM_IO;
        (*ctx).pb = avio_context.ctx;
    }

    match unsafe {
        sys::avformat_open_input(
            &mut ctx,
            std::ptr::null(),
            std::ptr::null(),
            std::ptr::null_mut(),
        )
    } {
        0 => match unsafe { sys::avformat_find_stream_info(ctx, std::ptr::null_mut()) } {
            r if r >= 0 => Ok((ffmpeg::format::context::Input::wrap(ctx), avio_context)),
            // TODO: we leak the ctx here
            e => Err(ffmpeg::Error::from(e)),
        },
        // TODO: we leak the ctx here
        e => Err(ffmpeg::Error::from(e)),
    }
}
