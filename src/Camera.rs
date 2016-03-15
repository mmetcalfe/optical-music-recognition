/*
https://crates.io/crates/ffmpeg-sys

ffmpeg call to take a grayscale photo:
http://stackoverflow.com/a/19524005/3622526

    ffmpeg -an -r 30.000030 -s 1920x1080 -f avfoundation -pix_fmt uyvy422 -video_device_index 0 -i "" -vframes 1 -pix_fmt gray out.bmp -v 100

    ffmpeg -f avfoundation -an -r 30.000030 -s 1920x1080 -pix_fmt uyvy422 -video_device_index 0 -i "" -vframes 1 -pix_fmt gray out.bmp -v 100
*/

extern crate ffmpeg_sys;
extern crate libc;
use std::ptr;
use std::mem;
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

use std::io::prelude::*;
use std::fs::File;
use std::fs::OpenOptions;

// From: https://doc.rust-lang.org/std/ffi/struct.CStr.html
fn cstring_to_str_safe(c_string : *const c_char) -> String {
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

fn log_av_error(operation : &str, errnum : libc::c_int) {
    println!("{}, AVERROR ({}): {}.", operation, errnum, av_error_string(errnum));
}

fn av_dict_string(dict : *const ffmpeg_sys::AVDictionary) -> String {
    let modified_buff : CString;

    unsafe {
        let mut dict_buff : *mut c_char = ptr::null_mut();

        let err = ffmpeg_sys::av_dict_get_string(dict, &mut dict_buff, ':' as i8, ',' as i8);
        if err < 0 {
            log_av_error("av_dict_string", err)
        }

        modified_buff = CString::from_raw(dict_buff);
    }

    modified_buff.to_string_lossy().into_owned()
}


// fn get_device_list() {
//     // Note: avdevice_list_input_sources is not implemented for avfoundation.
//     // Instead, run the following command to find an appropriate device name.
//     // $ ffmpeg -f avfoundation -list_devices true -i "default" -v 1000
//
//     // Get the avfoundation format:
//     let format_name = CString::new("avfoundation").unwrap();
//     let device_format = ffmpeg_sys::av_find_input_format(format_name.as_ptr());
//     println!("device_format: {:?}", cstring_to_str_safe((*device_format).long_name));
//
//     let device_name = ptr::null_mut();
//     let device_options = ptr::null_mut();
//     let mut device_list = ptr::null_mut(); //: ffmpeg_sys::AVDeviceInfoList**
//     let error = ffmpeg_sys::avdevice_list_input_sources(
//         device_format,
//         device_name,
//         device_options,
//         &mut device_list
//     );
//     println!("AVERROR ({}): {}.", error, av_error_string(error));
// }

unsafe fn create_av_dict(entries : Vec<(&str, &str, libc::c_int)>) -> *mut ffmpeg_sys::AVDictionary {
    let mut dict : *mut ffmpeg_sys::AVDictionary = ptr::null_mut();

    for entry in entries {
        let opt_key = CString::new(entry.0).unwrap();
        let opt_val = CString::new(entry.1).unwrap();
        let opt_flags = entry.2;
        ffmpeg_sys::av_dict_set(&mut dict, opt_key.as_ptr(), opt_val.as_ptr(), 0);
    }

    dict
}

unsafe fn pgm_save(buffer : *const u8, img_w : usize, img_h : usize, stride : usize, fname: &str) {
    // let mut file = OpenOptions::new()
    //     .create(true)
    //     .write(true)
    //     .truncate(true)
    //     .open(fname)
    //     .unwrap();
    // write!(file, "P5\n{} {}\n{}\n", img_w, img_h, 255);
    // for i  in 0..img_h {
    // // // for i in 0..img_h {
    //     writeln!(file, "{}", *buffer.offset((i * stride) as isize));
    // // // }
    // }

    // let stride = img_w;

    // println!("pgm_save: {:?}", buffer);

    // println!("pgm_save: fname {:?}", fname);
    let file_name = CString::new(fname.clone()).unwrap();
    // println!("pgm_save: file_name {:?}", file_name);
    let open_options = CString::new("w").unwrap();
    // println!("pgm_save: open_options {:?}", open_options);
    // let header_format = format!("P5\n{} {}\n{}\n", img_w, img_h, 255);
    let header_format = format!("P6\n{} {}\n{}\n", img_w, img_h, 255);
    let header_len = header_format.len();
    // println!("pgm_save: header_format {:?}", header_format);
    let c_header_format = CString::new(header_format).unwrap();

    // println!("pgm_save: fopen {:?}", file_name);
    let f = libc::fopen(file_name.as_ptr(), open_options.as_ptr());

    // println!("f: {:?}", f);

    libc::fwrite(c_header_format.as_ptr() as *const libc::c_void, 1, header_len, f);
    // println!("header: {:?}", c_header_format);

    for i in 0..img_h {
        // // println!("    row: {}", i);
        libc::fwrite(buffer.offset((i * stride) as isize) as *const libc::c_void, 1, img_w*3, f);
        // libc::fwrite(buffer.offset((i * stride) as isize) as *const libc::c_void, 1, img_w, f);
    }
    libc::fclose(f);
    // println!("closed");
}

pub struct FrameBuffers {
    frame_raw : *mut ffmpeg_sys::AVFrame,
    frame_rgb : *mut ffmpeg_sys::AVFrame,
    frame_buffer : *mut u8,
}

pub struct Camera {
    video_size : (usize, usize),
    video_stream : usize,
    format_context : *mut ffmpeg_sys::AVFormatContext,
    codec_context : *mut ffmpeg_sys::AVCodecContext,
    buffers : FrameBuffers,
}

impl Drop for Camera {
    fn drop(&mut self) {
        unsafe {
            ffmpeg_sys::av_free(self.buffers.frame_buffer as *mut libc::c_void);
            ffmpeg_sys::av_frame_free(&mut self.buffers.frame_raw);
            ffmpeg_sys::av_frame_free(&mut self.buffers.frame_rgb);
            println!("Camera::drop, Buffers freed.");

            ffmpeg_sys::avcodec_close(self.codec_context);
            println!("Camera::drop, Codec closed.");

            ffmpeg_sys::avformat_close_input(&mut self.format_context);
            println!("Camera::drop, Input closed.");
        }
    }
}

impl Camera {
    unsafe fn get_default_format_context(frame_dims : (usize, usize)) -> *mut ffmpeg_sys::AVFormatContext {

        // Get the avfoundation format:
        let format_name = CString::new("avfoundation").unwrap();
        let input_format = ffmpeg_sys::av_find_input_format(format_name.as_ptr());
        println!("input_format: {:?}", cstring_to_str_safe((*input_format).long_name));

        // Note the trailing ':' (avfoundation input filename format is "[[VIDEO]:[AUDIO]]").
        // Note: avdevice_list_input_sources is not implemented for avfoundation.
        // Instead, run the following command to find an appropriate device name.
        // $ ffmpeg -f avfoundation -list_devices true -i "default" -v 1000

        let video_size_str = format!("{}x{}", frame_dims.0, frame_dims.1);

        // Create the dictionary of open options:
        // TODO: Refactor into a method.
        // -r 30.000030 -s 1920x1080 -pix_fmt uyvy422
        // let device_filename = CString::new("HD Pro Webcam C920:").unwrap();
        // let mut open_options = create_av_dict(vec!(
        //     // ("list_devices", "1", 0),
        //     // ("video_device_index", "0", 0),
        //     // ("audio_device_index", "0", 0),
        //     ("pixel_format", "uyvy422", 0),
        //     ("framerate", "30.000030", 0),
        //     ("video_size", "1920x1080", 0),
        //     // ("capture_cursor", "1", 0),
        //     // ("capture_mouse_clicks", "1", 0),
        // ));

        let device_filename = CString::new("HD Pro Webcam C920:").unwrap();
        // let device_filename = CString::new("FaceTime HD Camera (Built-in):").unwrap();
        // let device_filename = CString::new("Capture screen 0:").unwrap();
        let mut open_options = create_av_dict(vec!(
            // ("list_devices", "1", 0),
            // ("video_device_index", "0", 0),
            // ("audio_device_index", "0", 0),
            ("pixel_format", "uyvy422", 0),
            // ("framerate", "14.999993", 0),
            // ("video_size", "1280x720", 0),
            ("framerate", "30.000030", 0),
            ("video_size", &video_size_str, 0),
            ("capture_cursor", "1", 0),
            ("capture_mouse_clicks", "1", 0),
        ));
        println!("device_filename: {:?}", device_filename);
        println!("open_options: {}", av_dict_string(open_options));

        let mut format_context = ffmpeg_sys::avformat_alloc_context();

        let open_error = ffmpeg_sys::avformat_open_input(
            &mut format_context,
            device_filename.as_ptr(),
            input_format,
            &mut open_options
        );

        println!("num rejected open_options: {}", ffmpeg_sys::av_dict_count(&*open_options));
        println!("rejected open_options: {}", av_dict_string(open_options));
        ffmpeg_sys::av_dict_free(&mut open_options);

        if open_error < 0 {
            log_av_error("open_error", open_error);
            return ptr::null_mut();
        }

        let mut info_options = ptr::null_mut();
        let stream_info_error = ffmpeg_sys::avformat_find_stream_info(format_context, &mut info_options);

        if stream_info_error < 0 {
            log_av_error("stream_info_error", stream_info_error);
            return ptr::null_mut();
        }

        println!("av_dump_format");
        let index = 0;
        let is_output = 0;
        ffmpeg_sys::av_dump_format(format_context, index, device_filename.as_ptr(), is_output);

        format_context
    }

    unsafe fn get_codec_context(format_context : *mut ffmpeg_sys::AVFormatContext) -> (*mut ffmpeg_sys::AVCodecContext, isize) {
        let streams = *(*format_context).streams;

        let video_stream = 0;
        // // Find an AV_CODEC_ID_RAWVIDEO stream:
        // let mut video_stream : isize = -1;
        // for i in 0..(*format_context).nb_streams as isize {
        //     let coder_type = (*(*streams.offset(i)).codec).coder_type;
        //     println!("video_stream {}, {}", i, coder_type);
        //     println!("AV_CODEC_ID_RAWVIDEO {}", ffmpeg_sys::AV_CODEC_ID_RAWVIDEO  as i32);
        //     if coder_type == ffmpeg_sys::AV_CODEC_ID_RAWVIDEO as i32 {
        //         video_stream = i;
        //         break;
        //     }
        // }
        // if video_stream == -1 {
        //     println!("codec_open_error: Could not find an AV_CODEC_ID_RAWVIDEO stream.");
        //     return ptr::null_mut()
        // }

        let codec_context = (*streams.offset(video_stream)).codec;

        let codec = ffmpeg_sys::avcodec_find_decoder((*codec_context).codec_id);
        if codec.is_null() {
            println!("avcodec_find_decoder: decoder not found for input codec.");
            return (ptr::null_mut(), 0);
        }

        println!("codec.long_name: {:?}", cstring_to_str_safe((*codec).long_name));

        // let mut codec_options = ptr::null_mut();
        let mut codec_options = create_av_dict(vec!(
            ("refcounted_frames", "0", 0),
        ));
        println!("codec_options: {}", av_dict_string(codec_options));
        let codec_open_error = ffmpeg_sys::avcodec_open2(codec_context, codec, &mut codec_options);
        println!("rejected codec_options: {}", av_dict_string(codec_options));
        if codec_open_error < 0 {
            log_av_error("codec_open_error", codec_open_error);
            return (ptr::null_mut(), 0);
        }

        (codec_context, video_stream)
    }

    // unsafe fn allocate_buffers(codec_context : *mut ffmpeg_sys::AVCodecContext) -> FrameBuffers {
    unsafe fn allocate_buffers(video_size : (usize, usize)) -> FrameBuffers {
        // av_frame_alloc(), av_frame_unref() and av_frame_free()
        let mut frame_raw = ffmpeg_sys::av_frame_alloc();
        let mut frame_rgb = ffmpeg_sys::av_frame_alloc();

        if frame_raw.is_null() || frame_rgb.is_null() {
            println!("ERROR: av_frame_alloc failure.");
            panic!()
        }

        let pixel_format = ffmpeg_sys::AV_PIX_FMT_RGB24;
        let width = video_size.0 as i32; // (*codec_context).width;
        let height = video_size.1 as i32; // (*codec_context).height;
        // let align = width * 3 * mem::size_of::<u8>() as i32;
        println!("pixel_format: {:?}", pixel_format);
        println!("width: {}", width);
        println!("height: {}", height);
        // println!("align: {}", align);
        let linesize = 1;
        let num_bytes = ffmpeg_sys::av_image_get_buffer_size( // avpicture_get_size(
            pixel_format,
            width,
            height,
            linesize // linesize
        );
        println!("depth: {}", (num_bytes as f64 / width as f64) / height as f64);
        let frame_buffer = ffmpeg_sys::av_malloc(num_bytes as usize * mem::size_of::<u8>());
        let picture = frame_rgb as *mut ffmpeg_sys::AVPicture;
        let fill_error = ffmpeg_sys::av_image_fill_arrays( // av_image_fill_arrays
            &mut (*picture).data[0],
            &mut (*picture).linesize[0],
            frame_buffer as *const u8,
            pixel_format,
            width,
            height,
            linesize
        );
        // let fill_error = ffmpeg_sys::avpicture_fill( // av_image_fill_arrays
        //     frame_rgb as *mut ffmpeg_sys::AVPicture,
        //     frame_buffer as *const u8,
        //     pixel_format,
        //     width,
        //     height
        // );

        if fill_error < 0 {
            log_av_error("fill_error", fill_error);
            panic!()
        }

        FrameBuffers {
            frame_buffer: frame_buffer  as *mut u8,
            frame_raw: frame_raw,
            frame_rgb: frame_rgb,
        }
    }

    unsafe fn get_next_frame(&self) {
        let width = self.video_size.0 as i32; //(*codec_context).width;
        let height = self.video_size.1 as i32; //(*codec_context).height;
        let src_w = width;
        let src_h = height;
        let src_format = ffmpeg_sys::AV_PIX_FMT_UYVY422; // (*self.codec_context).pix_fmt;
        let dst_w = width;
        let dst_h = height;
        let dst_format = ffmpeg_sys::AV_PIX_FMT_RGB24;
        let flags = ffmpeg_sys::SWS_BILINEAR;
        let src_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
        let dst_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
        let param : *const libc::c_double = ptr::null();

        let src_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(src_format);
        let dst_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(dst_format);
        println!("src_fmt ({}): {:?}, {:?}", src_format as i32, src_format, src_fmt_name);
        println!("dst_fmt ({}): {:?}, {:?}", dst_format as i32, dst_format, cstring_to_str_safe(dst_fmt_name));

        // let img_convert_ctx = ffmpeg_sys::sws_getCachedContext(ptr::null_mut(),
        let img_convert_ctx = ffmpeg_sys::sws_getContext(
            src_w, src_h, src_format,
            dst_w, dst_h, dst_format,
            flags, src_filter, dst_filter, param
        );

        // Read frames and save first five frames to disk
        let mut frame_index = 0;
        // Note: using mem::uninitialized() on a stack allocated packet led to weird behaviour.
        println!("sizeof(ffmpeg_sys::AVPacket): {:?}", mem::size_of::<ffmpeg_sys::AVPacket>());
        // let mut packet_box : Box<ffmpeg_sys::AVPacket> = Box::new(mem::uninitialized());
        // let mut packet_vec: Vec<u8> = Vec::with_capacity(mem::size_of::<ffmpeg_sys::AVPacket>());
        // let mut packet : *mut ffmpeg_sys::AVPacket = packet_vec.as_mut_ptr() as *mut ffmpeg_sys::AVPacket;
        let mut packet : *mut ffmpeg_sys::AVPacket = libc::calloc(1, mem::size_of::<ffmpeg_sys::AVPacket>()) as *mut ffmpeg_sys::AVPacket;
        // let mut packet : *mut ffmpeg_sys::AVPacket = Box::into_raw(packet_box);
        // let mut packet = ffmpeg_sys::av_packet_alloc();
        ffmpeg_sys::av_init_packet(packet);

        // while 1 >= 0 {
        while ffmpeg_sys::av_read_frame(self.format_context, packet) >= 0 {
            // Is this a packet from the video stream?

            if (*packet).stream_index == self.video_stream as i32 {
                frame_index = frame_index + 1;
                println!("Read frame {}", frame_index);
                println!("av_read_frame: packet.buff {:?}", (*packet).buf);

                // Decode video frame
                let mut frame_finished = 0;
                ffmpeg_sys::avcodec_decode_video2(
                    self.codec_context,
                    self.buffers.frame_raw,
                    &mut frame_finished,
                    packet
                );

                // Did we get a video frame?
                if frame_finished != 0 {
                    println!("  Read success {}", frame_index);

                    if (*self.buffers.frame_raw).width != width ||
                       (*self.buffers.frame_raw).height != height ||
                       (*self.buffers.frame_raw).format != src_format as i32 {
                        println!("ERROR: width, height, or format is incorrect.");
                        println!("(*self.buffers.frame_raw).format: {:?} != {:?}", (*self.buffers.frame_raw).format, src_format);
                        ffmpeg_sys::av_packet_unref(packet);
                        break;
                    }

                    // Convert the image from its native format to RGB
                    let src_slice_y = 0;
                    // let src_slice_h = 0; // (*self.codec_context).height;
                    let src_slice_h = height; //(*self.codec_context).height;
                    println!("  src_slice_h: {}", src_slice_h);
                    // let picture_raw = self.buffers.frame_raw as *const ffmpeg_sys::AVPicture;
                    // let picture_rgb = self.buffers.frame_rgb as *mut ffmpeg_sys::AVPicture;
                    let picture_raw = self.buffers.frame_raw;
                    // let picture_rgb = self.buffers.frame_rgb;
                    // ffmpeg_sys::sws_scale(
                    //     img_convert_ctx,
                    //     &((*picture_raw).data[0] as *const u8),
                    //     &(*picture_raw).linesize[0],
                    //     src_slice_y,
                    //     src_slice_h,
                    //     &mut (*picture_rgb).data[0],
                    //     &mut (*picture_rgb).linesize[0],
                    // );

                    // Save the image:
                    let picture_save = picture_raw;
                    println!("(*picture_save).data[0] as *const u8: {:?}", (*picture_save).data[0] as *const u8);
                    println!("width as usize: {:?}", width as usize);
                    println!("height as usize: {:?}", height as usize);
                    println!("(*picture_save).linesize[0] as usize: {:?}", (*picture_save).linesize[0] as usize);
                    pgm_save(
                        ((*picture_save).data[0] as *const u8),
                        width as usize,
                        height as usize,
                        (*picture_save).linesize[0] as usize,
                        "test.pgm"
                    );
                }
            } else {
                println!("Bad frame");
            }

            // Free the packet that was allocated by av_read_frame
            ffmpeg_sys::av_packet_unref(packet);
            if frame_index > 2 {
                break;
            }
        }

        // let packet_box : Box<ffmpeg_sys::AVPacket> = Box::from_raw(packet);
        // ffmpeg_sys::av_packet_free(packet);
        libc::free(packet as *mut libc::c_void);
    }

    pub fn get_default() -> Camera {
        let video_size = (640, 480);

        let mut camera = Camera {
            video_size: video_size,
            video_stream: 0,
            format_context: ptr::null_mut(),
            codec_context: ptr::null_mut(),
            buffers: FrameBuffers {
                frame_buffer: ptr::null_mut(),
                frame_raw: ptr::null_mut(),
                frame_rgb: ptr::null_mut(),
            },
        };

        unsafe {
            // Register available file formats and codecs with ffmpeg.
            ffmpeg_sys::avdevice_register_all();
            ffmpeg_sys::av_register_all();


            let format_context = Self::get_default_format_context(video_size);
            camera.format_context = format_context;
            if format_context.is_null() {
                return camera; // TODO: Return a Result instead.
            }

            let (codec_context, video_stream) = Self::get_codec_context(format_context);
            camera.codec_context = codec_context;
            camera.video_stream = video_stream as usize;
            if codec_context.is_null() {
                return camera;
            }

            // camera.buffers = Self::allocate_buffers(codec_context);
            camera.buffers = Self::allocate_buffers(video_size);

            camera.get_next_frame();
        }

        camera
    }
}
