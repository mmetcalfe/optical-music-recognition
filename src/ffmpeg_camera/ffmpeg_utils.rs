extern crate ffmpeg_sys;
extern crate libc;

use std::mem;
use std::ptr;
use std::ffi::CStr;
use std::ffi::CString;

use std::str::*;

use std::error;
use std::fmt;

pub fn make_uninitialised_vec<T>(length : usize) -> Vec<T> {
    // See: https://doc.rust-lang.org/std/vec/struct.Vec.html#method.from_raw_parts
    let mut tmp_data = Vec::<T>::with_capacity(length);
    let data_ptr = tmp_data.as_mut_ptr();
    unsafe {
        mem::forget(tmp_data); // Don't run tmp_data's destructor.
        // Create a full vector of uninitialised values:
        Vec::from_raw_parts(data_ptr, length, length)
    }
}

// Define an error type for FFmpeg:
#[derive(Debug)]
pub struct FfmpegError {
    errnum : libc::c_int,
    message : String,
}
impl fmt::Display for FfmpegError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{{FfmpegError ({}): {}}}", self.errnum, self.message)
    }
}
impl error::Error for FfmpegError {
    fn description(&self) -> &str {
        &self.message
    }
}
impl FfmpegError {
    pub fn from_av_error(errnum : libc::c_int) -> FfmpegError {
        FfmpegError {
            errnum: errnum,
            message: av_error_string(errnum),
        }
    }

    pub fn from_message(message : &str) -> FfmpegError {
        FfmpegError {
            errnum: 0,
            message: String::from_str(message).unwrap(),
        }
    }
}

// From: https://doc.rust-lang.org/std/ffi/struct.CStr.html
pub fn cstring_to_str_safe(c_string : *const libc::c_char) -> String {
    unsafe {
        CStr::from_ptr(c_string).to_string_lossy().into_owned()
    }
}

fn av_error_string(errnum : libc::c_int) -> String {
    let errbuf_size = 256;
    let errbuff = CString::new(vec!(' ' as u8; errbuf_size)).unwrap();
    let modified_buff : CString;

    unsafe {
        let errbuff_ptr = errbuff.into_raw();
        ffmpeg_sys::av_strerror(errnum, errbuff_ptr, errbuf_size);
        modified_buff = CString::from_raw(errbuff_ptr);
    }

    modified_buff.to_string_lossy().into_owned()
}

pub fn log_av_error(operation : &str, errnum : libc::c_int) {
    println!("{}, AVERROR ({}): {}.", operation, errnum, av_error_string(errnum));
}

pub fn av_dict_string(dict : *const ffmpeg_sys::AVDictionary) -> String {
    let result_string : String;

    unsafe {
        let mut dict_buff : *mut libc::c_char = ptr::null_mut();

        let err = ffmpeg_sys::av_dict_get_string(dict, &mut dict_buff, ':' as i8, ',' as i8);
        if err < 0 {
            log_av_error("av_dict_string", err)
        }

        // Note: Having rust take ownership of the buffer is leading to memory corruption.
        // modified_buff = CStr::from_raw(dict_buff);
        // result_string = modified_buff.to_string_lossy().into_owned()

        // Copy the buffer into a String:
        result_string = cstring_to_str_safe(dict_buff);

        // Free the buffer:
        ffmpeg_sys::av_free(dict_buff as *mut libc::c_void);
    }

    result_string
}

pub unsafe fn create_av_dict(entries : Vec<(&str, &str, libc::c_int)>) -> *mut ffmpeg_sys::AVDictionary {
    let mut dict : *mut ffmpeg_sys::AVDictionary = ptr::null_mut();

    for entry in entries {
        let opt_key = CString::new(entry.0).unwrap();
        let opt_val = CString::new(entry.1).unwrap();
        let opt_flags = entry.2;
        ffmpeg_sys::av_dict_set(&mut dict, opt_key.as_ptr(), opt_val.as_ptr(), opt_flags);
    }

    dict
}

pub fn av_pix_fmt_from_i32(pix_fmt: i32) -> ffmpeg_sys::AVPixelFormat {
    match pix_fmt {
        0 => ffmpeg_sys::AV_PIX_FMT_YUVJ420P,
        2 => ffmpeg_sys::AV_PIX_FMT_RGB24,
        5 => ffmpeg_sys::AV_PIX_FMT_YUV444P,
        17 => ffmpeg_sys::AV_PIX_FMT_UYVY422,
        _ => {
            println!("Unknown av_pix_fmt value: {}", pix_fmt);
            panic!();
        },
    }
}

pub unsafe fn convert_frame(
    src_frame : *const ffmpeg_sys::AVFrame,
    pixel_format : ffmpeg_sys::AVPixelFormat)
    -> Result<*mut ffmpeg_sys::AVFrame, FfmpegError> {
    let src_w = (*src_frame).width as usize;
    let src_h = (*src_frame).height as usize;
    let mut dst_frame = try!(make_empty_avframe(src_w, src_h, pixel_format));
    copy_and_convert_frame(src_frame, dst_frame);

    Ok(dst_frame)
}

pub unsafe fn copy_and_convert_frame(src_frame : *const ffmpeg_sys::AVFrame, dst_frame : *mut ffmpeg_sys::AVFrame) {
    let src_format = av_pix_fmt_from_i32((*src_frame).format);
    let dst_format = av_pix_fmt_from_i32((*dst_frame).format);

    let src_w = (*src_frame).width;
    let src_h = (*src_frame).height;
    let dst_w = src_w;
    let dst_h = src_h;

    let flags = ffmpeg_sys::SWS_BILINEAR;
    let src_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
    let dst_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
    let param : *const libc::c_double = ptr::null();

    // let src_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(src_format);
    // let dst_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(dst_format);
    // println!("src_fmt ({}): {:?}, {:?}", src_format as i32, src_format, src_fmt_name);
    // println!("dst_fmt ({}): {:?}, {:?}", dst_format as i32, dst_format, cstring_to_str_safe(dst_fmt_name));

    // let img_convert_ctx = ffmpeg_sys::sws_getCachedContext(ptr::null_mut(),
    let img_convert_ctx = ffmpeg_sys::sws_getContext(
        src_w, src_h, src_format,
        dst_w, dst_h, dst_format,
        flags, src_filter, dst_filter, param
    );

    // Convert the image from its native format to RGB
    let src_slice_y = 0;
    let src_slice_h = src_h;
    ffmpeg_sys::sws_scale(
        img_convert_ctx,
        &((*src_frame).data[0] as *const u8),
        &(*src_frame).linesize[0],
        src_slice_y,
        src_slice_h,
        &mut (*dst_frame).data[0],
        &mut (*dst_frame).linesize[0],
    );
}

pub unsafe fn make_frame_buffer_vec(width: usize, height: usize, pixel_format: ffmpeg_sys::AVPixelFormat) -> Vec<u8> {
    let align = 1; // "the assumed linesize alignment"
    let num_bytes = ffmpeg_sys::av_image_get_buffer_size(
        pixel_format,
        width as i32,
        height as i32,
        align
    );

    // Create the buffer:
    make_uninitialised_vec(num_bytes as usize)
}

pub unsafe fn copy_frame_data_to_vec(frame: *mut ffmpeg_sys::AVFrame) -> Result<Vec<u8>, FfmpegError> {
    let width = (*frame).width as usize;
    let height = (*frame).height as usize;

    let pixel_format = av_pix_fmt_from_i32((*frame).format);
    let mut data = make_frame_buffer_vec(width, height, pixel_format);

    let num_bytes = data.len();
    let align = 1; // "the assumed linesize alignment"

    // Copy raw frame to the buffer:
    let dst_size = num_bytes;
    let src_data_ptr = (&(*frame).data as *const _) as *const *const u8;
    let src_linesize_ptr = &(*frame).linesize as *const i32;
    let bytes_written = ffmpeg_sys::av_image_copy_to_buffer(
        data.as_mut_ptr(),
        dst_size as i32,
        src_data_ptr,
        src_linesize_ptr,
        pixel_format,
        width as i32,
        height as i32,
        align
    );

    if bytes_written > num_bytes as i32 {
        println!("bytes_written > num_bytes : {} > {}", bytes_written, num_bytes);
        panic!("get_image, av_image_get_buffer_size: Data buffer overrun.");
    } else if bytes_written < 0 {
        log_av_error("get_image, av_image_copy_to_buffer", bytes_written);
        return Err(FfmpegError::from_av_error(bytes_written))
    }

    Ok(data)
}

pub unsafe fn make_empty_avframe(width : usize, height : usize, pixel_format : ffmpeg_sys::AVPixelFormat) -> Result<*mut ffmpeg_sys::AVFrame, FfmpegError> {
    // let pixel_format = ffmpeg_sys::AV_PIX_FMT_YUV420P;

    // Initialise the input frame:
    // See: http://stackoverflow.com/a/20498359/3622526
    let mut yuv420p_frame = ffmpeg_sys::av_frame_alloc();
    if yuv420p_frame.is_null() {
        println!("ERROR: av_frame_alloc failure.");
        return Err(FfmpegError::from_message("ERROR: av_frame_alloc failure"));
    }

    (*yuv420p_frame).width = width as i32;
    (*yuv420p_frame).height = height as i32;
    (*yuv420p_frame).format = pixel_format as i32;

    // (*yuv420p_frame).pts = 1;
    // (*yuv420p_frame).quality = (*jpeg_context).global_quality;

    // let align = width * 3 * mem::size_of::<u8>() as i32;
    // println!("pixel_format: {:?}", pixel_format);
    let align = 1; // width*2;
    let fill_error = ffmpeg_sys::av_image_alloc( // av_image_fill_arrays
        &mut (*yuv420p_frame).data[0],
        &mut (*yuv420p_frame).linesize[0],
        width as i32,
        height as i32,
        pixel_format,
        align
    );

    // println!("(*yuv420p_frame).data: {:?}", (*yuv420p_frame).data);
    // println!("(*yuv420p_frame).linesize: {:?}", (*yuv420p_frame).linesize);

    if fill_error < 0 {
        log_av_error("fill_error", fill_error);
        return Err(FfmpegError::from_av_error(fill_error));
    }

    Ok(yuv420p_frame)
}

pub unsafe fn make_avframe(width : usize, height : usize, data : &Vec<u8>) -> Result<*mut ffmpeg_sys::AVFrame, FfmpegError> {
    let pixel_format = ffmpeg_sys::AV_PIX_FMT_UYVY422;
    // let pixel_format = ffmpeg_sys::AV_PIX_FMT_YUV444P;

    // Initialise the input frame:
    // See: http://stackoverflow.com/a/20498359/3622526
    let mut yuyv_frame = ffmpeg_sys::av_frame_alloc();
    if yuyv_frame.is_null() {
        println!("ERROR: av_frame_alloc failure.");
        return Err(FfmpegError::from_message("ERROR: av_frame_alloc failure"));
    }

    (*yuyv_frame).width = width as i32;
    (*yuyv_frame).height = height as i32;
    (*yuyv_frame).format = pixel_format as i32;

    (*yuyv_frame).pts = 1;
    // (*yuyv_frame).quality = (*jpeg_context).global_quality;

    // let align = width * 3 * mem::size_of::<u8>() as i32;
    // println!("pixel_format: {:?}", pixel_format);
    let align = 1; // self.width*2;
    let fill_error = ffmpeg_sys::av_image_fill_arrays( // av_image_fill_arrays
        &mut (*yuyv_frame).data[0],
        &mut (*yuyv_frame).linesize[0],
        ptr::null(),
        pixel_format,
        width as i32,
        height as i32,
        align
    );
    // println!("(*yuyv_frame).data: {:?}", (*yuyv_frame).data);

    (*yuyv_frame).data[0] = data.as_ptr() as *mut u8;

    // println!("(*yuyv_frame).data: {:?}", (*yuyv_frame).data);
    // println!("(*yuyv_frame).linesize: {:?}", (*yuyv_frame).linesize);

    if fill_error < 0 {
        log_av_error("fill_error", fill_error);
        return Err(FfmpegError::from_av_error(fill_error));
    }

    Ok(yuyv_frame)
}

pub unsafe fn save_frame_to_jpeg(src_frame : *mut ffmpeg_sys::AVFrame, save_fname : &str) -> Result<(), FfmpegError> {
    // let input_pixel_format = ffmpeg_sys::AV_PIX_FMT_UYVY422;
    let output_pixel_format = ffmpeg_sys::AV_PIX_FMT_YUVJ420P;

    let width = (*src_frame).width as usize;
    let height = (*src_frame).height as usize;

    // Get the mjpeg encoder:
    let codec_id = ffmpeg_sys::AV_CODEC_ID_MJPEG;
    let jpeg_encoder = ffmpeg_sys::avcodec_find_encoder(codec_id);
    if jpeg_encoder.is_null() {
        return Err(FfmpegError::from_message("avcodec_find_encoder: encoder not found."));
    }
    // println!("jpeg_encoder.long_name: {:?}", cstring_to_str_safe((*jpeg_encoder).long_name));

    // Allocate an encoder context:
    let mut jpeg_context = ffmpeg_sys::avcodec_alloc_context3(jpeg_encoder);
    if jpeg_context.is_null() {
        return Err(FfmpegError::from_message("avcodec_alloc_context: Could not allocate context."));
    }

    // (*jpeg_context).bit_rate = pCodecCtx->bit_rate;
    (*jpeg_context).width = width as i32;
    (*jpeg_context).height = height as i32;
    (*jpeg_context).pix_fmt = output_pixel_format;
    (*jpeg_context).codec_id = codec_id;
    (*jpeg_context).codec_type = ffmpeg_sys::AVMEDIA_TYPE_VIDEO;
    (*jpeg_context).time_base.num = 1;
    (*jpeg_context).time_base.den = 1;

    (*jpeg_context).bit_rate = 400000;
    (*jpeg_context).gop_size = 10;
    // (*jpeg_context).max_b_frames = 1;

    // let mut codec_options = ptr::null_mut();
    let mut codec_options = create_av_dict(vec!(
        ("refcounted_frames", "0", 0),
    ));
    // println!("codec_options: {}", av_dict_string(codec_options));
    let encoder_open_error = ffmpeg_sys::avcodec_open2(jpeg_context, jpeg_encoder, &mut codec_options);
    // println!("rejected codec_options: {}", av_dict_string(codec_options));
    if encoder_open_error < 0 {
        println!("avcodec_open2: failed to open codec.");
        log_av_error("encoder_open_error", encoder_open_error);
        return Err(FfmpegError::from_av_error(encoder_open_error));
    }

    // Set quality:
    (*jpeg_context).qmin = 1; // high -- low <==> 1 -- 31
    (*jpeg_context).qmax = 1; // high -- low <==> 1 -- 31

    (*jpeg_context).lmin = (*jpeg_context).qmin * ffmpeg_sys::FF_QP2LAMBDA;
    (*jpeg_context).mb_lmin = (*jpeg_context).lmin;
    (*jpeg_context).lmax = (*jpeg_context).qmax * ffmpeg_sys::FF_QP2LAMBDA;
    (*jpeg_context).mb_lmax = (*jpeg_context).lmax;
    (*jpeg_context).global_quality = (*jpeg_context).qmin * ffmpeg_sys::FF_QP2LAMBDA;
    // (*jpeg_context).flags = ffmpeg_sys::CODEC_FLAG_QSCALE;

    // let mut src_frame = try!(self.make_avframe());
    // let mut yuv420p_frame = try!(make_empty_avframe(width, height, ffmpeg_sys::AV_PIX_FMT_YUV420P));
    // copy_and_convert_frame(src_frame, yuv420p_frame);
    let mut yuv420p_frame = try!(convert_frame(src_frame, ffmpeg_sys::AV_PIX_FMT_YUV420P));

    // println!("Initialise the output packet");

    // Initialise the output packet
    let mut jpeg_packet_vec: Vec<u8> = Vec::with_capacity(mem::size_of::<ffmpeg_sys::AVPacket>());
    let mut jpeg_packet : *mut ffmpeg_sys::AVPacket = jpeg_packet_vec.as_mut_ptr() as *mut ffmpeg_sys::AVPacket;
    ffmpeg_sys::av_init_packet(jpeg_packet);
    (*jpeg_packet).data = ptr::null_mut();
    (*jpeg_packet).size = 0;

    // println!("Encode the frame");

    // Encode the frame:
    let mut got_packet = 0;
    let encode_error = ffmpeg_sys::avcodec_encode_video2(
        jpeg_context,
        jpeg_packet,
        yuv420p_frame,
        &mut got_packet
    );

    if encode_error < 0 {
        log_av_error("saveJpeg: avcodec_encode_video2", encode_error);
        return Err(FfmpegError::from_av_error(encode_error));
    }

    if (*jpeg_packet).data.is_null() {
        return Err(FfmpegError::from_message("ERROR: encoded packet is empty."));
    }

    // println!("Save frame");
    {
        let data_ptr = (*jpeg_packet).data;
        let data_size = (*jpeg_packet).size as usize;
        let file_name = CString::new(save_fname.clone()).unwrap();
        let open_options = CString::new("w").unwrap();
        let f = libc::fopen(file_name.as_ptr(), open_options.as_ptr());
        libc::fwrite(data_ptr as *const libc::c_void, 1, data_size, f);
        libc::fclose(f);
    }

    // println!("Free resources");
    ffmpeg_sys::av_freep((&(*yuv420p_frame).data[0] as *const _) as *mut libc::c_void);
    ffmpeg_sys::av_frame_free(&mut yuv420p_frame);

    ffmpeg_sys::av_free_packet(jpeg_packet);
    ffmpeg_sys::avcodec_close(jpeg_context);
    ffmpeg_sys::avcodec_free_context(&mut jpeg_context);

    Ok(())
}
