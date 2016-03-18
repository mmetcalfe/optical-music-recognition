
// use std::ops;

// use std::io::prelude::*;
// use std::fs::File;
// use std::fs::OpenOptions;

extern crate ffmpeg_sys;
extern crate libc;
use ffmpeg_camera;
use ffmpeg_camera::FfmpegError;
use std::ptr;
use std::mem;
use std::ffi::CString;

pub struct Pixel {
    pub y : u8,
    pub cb : u8,
    pub cr : u8,
}

pub struct Image {
    pub width : usize, // width in pixels
    pub height : usize, // height in pixels
    pub data : Vec<u8>, // uyvy422 data buffer
}

impl Image {

    pub fn index(&self, row : usize, col : usize) -> Pixel {
        let row_offset = row * self.width * 2;
        let y_i = row_offset + col*2 + 1;
        // let uv_offset = ((col*2) / 4) * 4;
        let uv_offset = (col / 2) * 4;
        let u_i = row_offset + uv_offset + 0;
        let v_i = row_offset + uv_offset + 2;

        unsafe {
            Pixel {
                y: *self.data.get_unchecked(y_i),
                cb: *self.data.get_unchecked(u_i),
                cr: *self.data.get_unchecked(v_i),
            }
        }
    }

    unsafe fn frame_uyvy422_to_yuvj420p(frame_uyvy422 : *const ffmpeg_sys::AVFrame, frame_yuvj420p : *mut ffmpeg_sys::AVFrame) {

        let src_format = ffmpeg_sys::AV_PIX_FMT_UYVY422;
        let dst_format = ffmpeg_sys::AV_PIX_FMT_YUVJ420P;

        println!("frame_uyvy: {:?}", frame_uyvy422);
        println!("frame_yuvj420p: {:?}", frame_yuvj420p);

        let src_w = (*frame_uyvy422).width;
        let src_h = (*frame_uyvy422).height;
        // let src_format = (*frame_uyvy422).format as ffmpeg_sys::AVPixelFormat;
        let dst_w = src_w;
        let dst_h = src_h;
        // let dst_format = ffmpeg_sys::AV_PIX_FMT_RGB24;
        let flags = ffmpeg_sys::SWS_BILINEAR;
        let src_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
        let dst_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
        let param : *const libc::c_double = ptr::null();

        let src_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(src_format);
        let dst_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(dst_format);
        println!("src_fmt ({}): {:?}, {:?}", src_format as i32, src_format, src_fmt_name);
        println!("dst_fmt ({}): {:?}, {:?}", dst_format as i32, dst_format, ffmpeg_camera::cstring_to_str_safe(dst_fmt_name));

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
            &((*frame_uyvy422).data[0] as *const u8),
            &(*frame_uyvy422).linesize[0],
            src_slice_y,
            src_slice_h,
            &mut (*frame_yuvj420p).data[0],
            &mut (*frame_yuvj420p).linesize[0],
        );
    }

    unsafe fn get_avframe(&self) -> Result<*mut ffmpeg_sys::AVFrame, FfmpegError> {
        let pixel_format = ffmpeg_sys::AV_PIX_FMT_UYVY422;

        // Initialise the input frame:
        // See: http://stackoverflow.com/a/20498359/3622526
        let mut yuyv_frame = ffmpeg_sys::av_frame_alloc();
        if yuyv_frame.is_null() {
            println!("ERROR: av_frame_alloc failure.");
            return Err(FfmpegError::from_message("ERROR: av_frame_alloc failure"));
        }

        (*yuyv_frame).width = self.width as i32;
        (*yuyv_frame).height = self.height as i32;
        (*yuyv_frame).format = pixel_format as i32;

        (*yuyv_frame).pts = 1;
        // (*yuyv_frame).quality = (*jpeg_context).global_quality;

        // let align = width * 3 * mem::size_of::<u8>() as i32;
        println!("pixel_format: {:?}", pixel_format);
        let align = 1; // self.width*2;
        let fill_error = ffmpeg_sys::av_image_fill_arrays( // av_image_fill_arrays
            &mut (*yuyv_frame).data[0],
            &mut (*yuyv_frame).linesize[0],
            ptr::null(),
            pixel_format,
            self.width as i32,
            self.height as i32,
            align
        );
        println!("(*yuyv_frame).data: {:?}", (*yuyv_frame).data);

        (*yuyv_frame).data[0] = self.data.as_ptr() as *mut u8;

        println!("(*yuyv_frame).data: {:?}", (*yuyv_frame).data);
        println!("(*yuyv_frame).linesize: {:?}", (*yuyv_frame).linesize);

        if fill_error < 0 {
            ffmpeg_camera::log_av_error("fill_error", fill_error);
            return Err(FfmpegError::from_av_error(fill_error));
        }

        Ok(yuyv_frame)
    }

    unsafe fn make_empty_yuv420p_frame(&self) -> Result<*mut ffmpeg_sys::AVFrame, FfmpegError> {
        let pixel_format = ffmpeg_sys::AV_PIX_FMT_YUV420P;

        // Initialise the input frame:
        // See: http://stackoverflow.com/a/20498359/3622526
        let mut yuv420p_frame = ffmpeg_sys::av_frame_alloc();
        if yuv420p_frame.is_null() {
            println!("ERROR: av_frame_alloc failure.");
            return Err(FfmpegError::from_message("ERROR: av_frame_alloc failure"));
        }

        (*yuv420p_frame).width = self.width as i32;
        (*yuv420p_frame).height = self.height as i32;
        (*yuv420p_frame).format = pixel_format as i32;

        // (*yuv420p_frame).pts = 1;
        // (*yuv420p_frame).quality = (*jpeg_context).global_quality;

        // let align = width * 3 * mem::size_of::<u8>() as i32;
        println!("pixel_format: {:?}", pixel_format);
        let align = 1; // self.width*2;
        let fill_error = ffmpeg_sys::av_image_alloc( // av_image_fill_arrays
            &mut (*yuv420p_frame).data[0],
            &mut (*yuv420p_frame).linesize[0],
            self.width as i32,
            self.height as i32,
            pixel_format,
            align
        );

        println!("(*yuv420p_frame).data: {:?}", (*yuv420p_frame).data);
        println!("(*yuv420p_frame).linesize: {:?}", (*yuv420p_frame).linesize);

        if fill_error < 0 {
            ffmpeg_camera::log_av_error("fill_error", fill_error);
            return Err(FfmpegError::from_av_error(fill_error));
        }

        Ok(yuv420p_frame)
    }

    // Based on: https://lists.libav.org/pipermail/libav-user/2010-August/005159.html
    pub unsafe fn save_jpeg(&self, save_fname : &str) -> Result<(), FfmpegError> {
        let input_pixel_format = ffmpeg_sys::AV_PIX_FMT_UYVY422;
        let output_pixel_format = ffmpeg_sys::AV_PIX_FMT_YUVJ420P;

        // Get the mjpeg encoder:
        let codec_id = ffmpeg_sys::AV_CODEC_ID_MJPEG;
        let jpeg_encoder = ffmpeg_sys::avcodec_find_encoder(codec_id);
        if jpeg_encoder.is_null() {
            return Err(FfmpegError::from_message("avcodec_find_encoder: encoder not found."));
        }
        println!("jpeg_encoder.long_name: {:?}", ffmpeg_camera::cstring_to_str_safe((*jpeg_encoder).long_name));

        // Allocate an encoder context:
        let mut jpeg_context = ffmpeg_sys::avcodec_alloc_context3(jpeg_encoder);
        if jpeg_context.is_null() {
            return Err(FfmpegError::from_message("avcodec_alloc_context: Could not allocate context."));
        }

        // (*jpeg_context).bit_rate = pCodecCtx->bit_rate;
        (*jpeg_context).width = self.width as i32;
        (*jpeg_context).height = self.height as i32;
        (*jpeg_context).pix_fmt = output_pixel_format;
        (*jpeg_context).codec_id = codec_id;
        (*jpeg_context).codec_type = ffmpeg_sys::AVMEDIA_TYPE_VIDEO;
        (*jpeg_context).time_base.num = 1;
        (*jpeg_context).time_base.den = 1;

        (*jpeg_context).bit_rate = 400000;
        (*jpeg_context).gop_size = 10;
        // (*jpeg_context).max_b_frames = 1;

        // let mut codec_options = ptr::null_mut();
        let mut codec_options = ffmpeg_camera::create_av_dict(vec!(
            ("refcounted_frames", "0", 0),
        ));
        // println!("codec_options: {}", ffmpeg_camera::av_dict_string(codec_options));
        let encoder_open_error = ffmpeg_sys::avcodec_open2(jpeg_context, jpeg_encoder, &mut codec_options);
        // println!("rejected codec_options: {}", ffmpeg_camera::av_dict_string(codec_options));
        if encoder_open_error < 0 {
            println!("avcodec_open2: failed to open codec.");
            ffmpeg_camera::log_av_error("encoder_open_error", encoder_open_error);
            return Err(FfmpegError::from_av_error(encoder_open_error));
        }

        (*jpeg_context).lmin = (*jpeg_context).qmin * ffmpeg_sys::FF_QP2LAMBDA;
        (*jpeg_context).mb_lmin = (*jpeg_context).lmin;
        (*jpeg_context).lmax = (*jpeg_context).qmax * ffmpeg_sys::FF_QP2LAMBDA;
        (*jpeg_context).mb_lmax = (*jpeg_context).lmax;
        (*jpeg_context).global_quality = (*jpeg_context).qmin * ffmpeg_sys::FF_QP2LAMBDA;
        // (*jpeg_context).flags = ffmpeg_sys::CODEC_FLAG_QSCALE;

        let mut yuyv422_frame = try!(self.get_avframe());
        let mut yuv420p_frame = try!(self.make_empty_yuv420p_frame());
        Self::frame_uyvy422_to_yuvj420p(yuyv422_frame, yuv420p_frame);

        println!("Initialise the output packet");

        // Initialise the output packet
        let mut jpeg_packet_vec: Vec<u8> = Vec::with_capacity(mem::size_of::<ffmpeg_sys::AVPacket>());
        let mut jpeg_packet : *mut ffmpeg_sys::AVPacket = jpeg_packet_vec.as_mut_ptr() as *mut ffmpeg_sys::AVPacket;
        ffmpeg_sys::av_init_packet(jpeg_packet);
        (*jpeg_packet).data = ptr::null_mut();
        (*jpeg_packet).size = 0;

        println!("Encode the frame");

        // Encode the frame:
        let mut got_packet = 0;
        let encode_error = ffmpeg_sys::avcodec_encode_video2(
            jpeg_context,
            jpeg_packet,
            yuv420p_frame,
            &mut got_packet
        );

        if encode_error < 0 {
            ffmpeg_camera::log_av_error("saveJpeg: avcodec_encode_video2", encode_error);
            return Err(FfmpegError::from_av_error(encode_error));
        }

        if (*jpeg_packet).data.is_null() {
            return Err(FfmpegError::from_message("ERROR: encoded packet is empty."));
        }

        println!("Save frame");
        {
            let data_ptr = (*jpeg_packet).data;
            let data_size = (*jpeg_packet).size as usize;
            let file_name = CString::new(save_fname.clone()).unwrap();
            let open_options = CString::new("w").unwrap();
            let f = libc::fopen(file_name.as_ptr(), open_options.as_ptr());
            libc::fwrite(data_ptr as *const libc::c_void, 1, data_size, f);
            libc::fclose(f);
        }

        println!("Free resources");

        ffmpeg_sys::av_frame_free(&mut yuyv422_frame);


        ffmpeg_sys::av_freep((&(*yuv420p_frame).data[0] as *const _) as *mut libc::c_void);
        ffmpeg_sys::av_frame_free(&mut yuv420p_frame);

        ffmpeg_sys::av_free_packet(jpeg_packet);
        ffmpeg_sys::avcodec_close(jpeg_context);
        ffmpeg_sys::avcodec_free_context(&mut jpeg_context);
        Ok(())
    }

    // pub fn index(&self, row : usize, col : usize) -> Pixel {
    //     // // Use nearest u and v:
    //     // // u y v y u y v y ...
    //     // // 1 1 1
    //     // //     2 2 2
    //     // let pos = (row * self.width + col) * 2;
    //     // let y_i = pos + 1;
    //     // // let u_i = pos + if even_col { 0 } else { 2 };
    //     // // let v_i = pos + if even_col { 2 } else { 0 };
    //     // let uv_offset = (col % 2) * 2;
    //     // let u_i = pos + uv_offset;
    //     // let v_i = pos + (2 - uv_offset);
    //
    //     // // Grouped yuyv:
    //     // // u y v y u y v y ...
    //     // // 1 1 1
    //     // // 2   2 2
    //     // let row_offset = row * self.width * 2;
    //     // let y_i = row_offset + col*2 + 1;
    //     // // let uv_offset = ((col*2) / 4) * 4;
    //     // let uv_offset = (col / 2) * 4;
    //     // let u_i = row_offset + uv_offset + 0;
    //     // let v_i = row_offset + uv_offset + 2;
    //
    //     // Grouped yuyv:
    //     // u y v y u y v y ...
    //     // 1 1 1
    //     // 2   2 2
    //     let row_offset = row * self.width * 2;
    //     let y_i = row_offset + col*2 + 1;
    //     // let uv_offset = ((col*2) / 4) * 4;
    //     let uv_offset = (col / 2) * 4;
    //     let u_i = row_offset + uv_offset + 0;
    //     let v_i = row_offset + uv_offset + 2;
    //
    //     unsafe {
    //         Pixel {
    //             y: *self.data.get_unchecked(y_i),
    //             u: *self.data.get_unchecked(u_i),
    //             v: *self.data.get_unchecked(v_i),
    //         }
    //     }
    // }

}
