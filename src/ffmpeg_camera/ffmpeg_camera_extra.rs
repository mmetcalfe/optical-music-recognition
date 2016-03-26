

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


// unsafe fn pgm_save(buffer : *const u8, img_w : usize, img_h : usize, stride : usize, fname: &str) {
//     // let mut file = OpenOptions::new()
//     //     .create(true)
//     //     .write(true)
//     //     .truncate(true)
//     //     .open(fname)
//     //     .unwrap();
//     // write!(file, "P5\n{} {}\n{}\n", img_w, img_h, 255);
//     // for i  in 0..img_h {
//     // // // for i in 0..img_h {
//     //     writeln!(file, "{}", *buffer.offset((i * stride) as isize));
//     // // // }
//     // }
//
//     // let stride = img_w;
//
//     // println!("pgm_save: {:?}", buffer);
//
//     // println!("pgm_save: fname {:?}", fname);
//     let file_name = CString::new(fname.clone()).unwrap();
//     // println!("pgm_save: file_name {:?}", file_name);
//     let open_options = CString::new("w").unwrap();
//     // println!("pgm_save: open_options {:?}", open_options);
//     // let header_format = format!("P5\n{} {}\n{}\n", img_w, img_h, 255);
//     let header_format = format!("P6\n{} {}\n{}\n", img_w, img_h, 255);
//     let header_len = header_format.len();
//     // println!("pgm_save: header_format {:?}", header_format);
//     let c_header_format = CString::new(header_format).unwrap();
//
//     // println!("pgm_save: fopen {:?}", file_name);
//     let f = libc::fopen(file_name.as_ptr(), open_options.as_ptr());
//
//     // println!("f: {:?}", f);
//
//     libc::fwrite(c_header_format.as_ptr() as *const libc::c_void, 1, header_len, f);
//     // println!("header: {:?}", c_header_format);
//
//     for i in 0..img_h {
//         // // println!("    row: {}", i);
//         libc::fwrite(buffer.offset((i * stride) as isize) as *const libc::c_void, 1, img_w*3, f);
//         // libc::fwrite(buffer.offset((i * stride) as isize) as *const libc::c_void, 1, img_w, f);
//     }
//     libc::fclose(f);
//     // println!("closed");
// }


// pub struct FrameBuffers {
//     frame_raw : *mut ffmpeg_sys::AVFrame,
//     // frame_rgb : *mut ffmpeg_sys::AVFrame,
//     // frame_buffer : *mut u8,
// }

// pub struct FfmpegCamera {
//     stream_index : usize,
//     format_context : *mut ffmpeg_sys::AVFormatContext,
//     decoder_context : *mut ffmpeg_sys::AVCodecContext,
//     frame_raw : *mut ffmpeg_sys::AVFrame,
//     // video_dst_data_vec : Vec<*mut u8>,
//     // video_dst_linesize_vec : Vec<libc::c_int>,
// }

// impl Drop for FfmpegCamera {
//     fn drop(&mut self) {
//         unsafe {
//
//             ffmpeg_sys::avcodec_close(self.decoder_context);
//             println!("FfmpegCamera::drop, Codec closed.");
//
//             ffmpeg_sys::avformat_close_input(&mut self.format_context);
//             println!("FfmpegCamera::drop, Input closed.");
//
//             ffmpeg_sys::av_frame_free(&mut self.frame_raw);
//             // ffmpeg_sys::av_frame_free(&mut self.buffers.frame_raw);
//             // ffmpeg_sys::av_frame_free(&mut self.buffers.frame_rgb);
//             // ffmpeg_sys::av_free(self.buffers.frame_buffer as *mut libc::c_void);
//             println!("FfmpegCamera::drop, Buffers freed.");
//
//             // let video_dst_data : *mut *mut u8 = self.video_dst_data_vec.as_mut_ptr();
//             // ffmpeg_sys::av_free(*video_dst_data.offset(0) as *mut libc::c_void);
//             // println!("FfmpegCamera::drop, video_dst_data freed.");
//         }
//     }
// }

// unsafe fn frame_uyvy422_to_rgb(frame_uyvy : *const ffmpeg_sys::AVFrame, frame_rgb : *mut ffmpeg_sys::AVFrame) {
//
//     println!("frame_uyvy: {:?}", frame_uyvy);
//     println!("frame_rgb: {:?}", frame_rgb);
//
//     let src_w = (*frame_uyvy).width;
//     let src_h = (*frame_uyvy).height;
//     // let src_format = (*frame_uyvy).format as ffmpeg_sys::AVPixelFormat;
//     let src_format = ffmpeg_sys::AV_PIX_FMT_UYVY422;
//     let dst_w = src_w;
//     let dst_h = src_h;
//     let dst_format = ffmpeg_sys::AV_PIX_FMT_RGB24;
//     let flags = ffmpeg_sys::SWS_BILINEAR;
//     let src_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
//     let dst_filter : *mut ffmpeg_sys::SwsFilter = ptr::null_mut();
//     let param : *const libc::c_double = ptr::null();
//
//     // let src_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(src_format);
//     // let dst_fmt_name = ffmpeg_sys::av_get_pix_fmt_name(dst_format);
//     // println!("src_fmt ({}): {:?}, {:?}", src_format as i32, src_format, src_fmt_name);
//     // println!("dst_fmt ({}): {:?}, {:?}", dst_format as i32, dst_format, cstring_to_str_safe(dst_fmt_name));
//
//     // let img_convert_ctx = ffmpeg_sys::sws_getCachedContext(ptr::null_mut(),
//     let img_convert_ctx = ffmpeg_sys::sws_getContext(
//         src_w, src_h, src_format,
//         dst_w, dst_h, dst_format,
//         flags, src_filter, dst_filter, param
//     );
//
//     // Convert the image from its native format to RGB
//     let src_slice_y = 0;
//     let src_slice_h = src_h;
//     ffmpeg_sys::sws_scale(
//         img_convert_ctx,
//         &((*frame_uyvy).data[0] as *const u8),
//         &(*frame_uyvy).linesize[0],
//         src_slice_y,
//         src_slice_h,
//         &mut (*frame_rgb).data[0],
//         &mut (*frame_rgb).linesize[0],
//     );
// }


// fn allocate_dst_buffers() {
//
//     // let video_stream = *(*format_context).streams.offset(stream_index as isize);
//     // let video_dec_ctx = (*video_stream).codec;
//
//     // // Allocate image contatiner for each decoded frame:
//     // // let mut video_dst_data : [*mut u8; 4] = [ptr::null_mut(); 4];
//     // // let pointers : *mut *mut u8 = (&video_dst_data as *const _) as *mut *mut u8;
//     // // let mut video_dst_linesize : [libc::c_int; 4] = [0; 4];
//     // // let linesizes : *mut libc::c_int = (&video_dst_linesize as *const libc::c_int) as *mut libc::c_int;
//     //
//     // // let mut video_dst_data_vec : Vec<*mut u8> = vec![ptr::null_mut(); 4];
//     // let video_dst_data : *mut *mut u8 = camera.video_dst_data_vec.as_mut_ptr();
//     //
//     // // let mut video_dst_linesize_vec : Vec<libc::c_int> = vec![0; 4];
//     // let mut video_dst_linesize : *mut libc::c_int = camera.video_dst_linesize_vec.as_mut_ptr();
//     //
//     // let pointers : *mut *mut u8 = video_dst_data;
//     // let linesizes :  *mut libc::c_int = video_dst_linesize;
//     // println!("video_dst_data: {:?}", camera.video_dst_data_vec);
//     // println!("video_dst_linesize: {:?}", camera.video_dst_linesize_vec);
//     // let width = (*video_dec_ctx).width;
//     // let height = (*video_dec_ctx).height;
//     // let pix_fmt = (*video_dec_ctx).pix_fmt;
//     //
//     // println!("width: {:?}", width);
//     // println!("height: {:?}", height);
//     // println!("pix_fmt: {:?}", pix_fmt);
//     //
//     // let align = 1;
//     // let video_dst_bufsize = ffmpeg_sys::av_image_alloc(
//     //     pointers,
//     //     linesizes,
//     //     width,
//     //     height,
//     //     pix_fmt,
//     //     align
//     // );
//     // println!("video_dst_data: {:?}", camera.video_dst_data_vec);
//     // println!("video_dst_linesize: {:?}", camera.video_dst_linesize_vec);
//     //
//     // if video_dst_bufsize < 0 {
//     //     log_av_error("av_image_alloc", video_dst_bufsize);
//     //     println!("ERROR: av_image_alloc: Could not allocate raw video buffer.");
//     //     return Err(FfmpegError::from_av_error(video_dst_bufsize));
//     // }
// }


//     unsafe fn allocate_buffers(decoder_context : *mut ffmpeg_sys::AVCodecContext) -> FrameBuffers {
//     // unsafe fn allocate_buffers(video_size : (usize, usize)) -> FrameBuffers {
//
//         // av_frame_alloc(), av_frame_unref() and av_frame_free()
//         let frame_raw = ffmpeg_sys::av_frame_alloc();
//         // let mut frame_rgb = ffmpeg_sys::av_frame_alloc();
//
//         if frame_raw.is_null() {
//         // if frame_raw.is_null() || frame_rgb.is_null() {
//             println!("ERROR: av_frame_alloc failure.");
//             panic!()
//         }
//
// // ////////////////////
// //         let pixel_format = ffmpeg_sys::AV_PIX_FMT_RGB24;
// //         let width = (*decoder_context).width;
// //         let height = (*decoder_context).height;
// //         // let align = width * 3 * mem::size_of::<u8>() as i32;
// //         println!("pixel_format: {:?}", pixel_format);
// //         println!("width: {}", width);
// //         println!("height: {}", height);
// //         // println!("align: {}", align);
// //         let linesize = 1;
// //         let num_bytes = ffmpeg_sys::av_image_get_buffer_size( // avpicture_get_size(
// //             pixel_format,
// //             width,
// //             height,
// //             linesize // linesize
// //         );
// //         println!("depth: {}", (num_bytes as f64 / width as f64) / height as f64);
// //         let frame_buffer = ffmpeg_sys::av_malloc(num_bytes as usize * mem::size_of::<u8>());
// //         // let picture = frame_rgb as *mut ffmpeg_sys::AVPicture;
// //         // let fill_error = ffmpeg_sys::av_image_fill_arrays( // av_image_fill_arrays
// //         //     &mut (*picture).data[0],
// //         //     &mut (*picture).linesize[0],
// //         //     frame_buffer as *const u8,
// //         //     pixel_format,
// //         //     width,
// //         //     height,
// //         //     linesize
// //         // );
// //         let fill_error = ffmpeg_sys::avpicture_fill( // av_image_fill_arrays
// //             frame_rgb as *mut ffmpeg_sys::AVPicture,
// //             frame_buffer as *const u8,
// //             pixel_format,
// //             width,
// //             height
// //         );
// //
// //         if fill_error < 0 {
// //             log_av_error("fill_error", fill_error);
// //             panic!()
// //         }
// // ////////////////////
//
//         FrameBuffers {
//             // frame_buffer: frame_buffer  as *mut u8,
//             // frame_buffer: ptr::null_mut(),
//             frame_raw: frame_raw,
//             // frame_rgb: frame_rgb, // ptr::null_mut(),
//             // frame_rgb: frame_rgb,
//         }
//     }

// fn copy_to_dst_buffer_and_save() {
//     //
//     // let video_dst_data : *mut *mut u8 = self.video_dst_data_vec.as_mut_ptr();
//     // let mut video_dst_linesize : *mut libc::c_int = self.video_dst_linesize_vec.as_mut_ptr();
//     //
//     // // /* copy decoded frame to destination buffer:
//     // //  * this is required since rawvideo expects non aligned data */
//     // // ffmpeg_sys::av_image_copy(
//     // //     video_dst_data,
//     // //     video_dst_linesize,
//     // //     (&(*self.buffers.frame_raw).data as *const _) as *const *const u8,
//     // //     &(*self.buffers.frame_raw).linesize as *const i32,
//     // //     pix_fmt,
//     // //     width,
//     // //     height
//     // // );
//     // // // Save the image:
//     // // pgm_save(
//     // //     self.video_dst_data_vec[0] as *const u8,
//     // //     width as usize,
//     // //     height as usize,
//     // //     self.video_dst_linesize_vec[0] as usize,
//     // //     "test.pgm"
//     // // );
//     //
//     // let picture_save = self.buffers.frame_rgb;
//     // frame_uyvy422_to_rgb(self.buffers.frame_raw, picture_save);
//     //
//     // // Save the image:
//     // // println!("(*picture_save).data[0] as *const u8: {:?}", (*picture_save).data[0] as *const u8);
//     // // println!("width as usize: {:?}", width as usize);
//     // // println!("height as usize: {:?}", height as usize);
//     // // println!("(*picture_save).linesize[0] as usize: {:?}", (*picture_save).linesize[0] as usize);
//     // pgm_save(
//     //     ((*picture_save).data[0] as *const u8),
//     //     width as usize,
//     //     height as usize,
//     //     (*picture_save).linesize[0] as usize,
//     //     "test.pgm"
//     // );
// }
