extern crate ffmpeg_sys;
extern crate libc;
use std::ptr;
use std::mem;
use std::ffi::CString;

use ffmpeg_camera::image_ycbcr; // ::Image;

use ffmpeg_camera::ffmpeg_utils;
use ffmpeg_camera::ffmpeg_utils::FfmpegError;

pub struct FfmpegCamera {
    stream_index : usize,
    format_context : *mut ffmpeg_sys::AVFormatContext,
    decoder_context : *mut ffmpeg_sys::AVCodecContext,
    frame_raw : *mut ffmpeg_sys::AVFrame,
}

impl Drop for FfmpegCamera {
    fn drop(&mut self) {
        unsafe {
            ffmpeg_sys::avcodec_close(self.decoder_context);
            // println!("FfmpegCamera::drop, Codec closed.");

            ffmpeg_sys::avformat_close_input(&mut self.format_context);
            // println!("FfmpegCamera::drop, Input closed.");

            ffmpeg_sys::av_frame_free(&mut self.frame_raw);
            // println!("FfmpegCamera::drop, Buffers freed.");
        }
    }
}

impl FfmpegCamera {
    unsafe fn get_default_format_context(device_filename_str : &str, framerate_str : &str, frame_dims : (usize, usize)) -> Result<*mut ffmpeg_sys::AVFormatContext, FfmpegError> {

        // Get the avfoundation format:
        let format_name = CString::new("avfoundation").unwrap();
        let input_format = ffmpeg_sys::av_find_input_format(format_name.as_ptr());
        println!("input_format: {:?}", input_format);
        println!("input_format: {:?}", ffmpeg_utils::cstring_to_str_safe((*input_format).long_name));

        // Note the trailing ':' (avfoundation input filename format is "[[VIDEO]:[AUDIO]]").
        // Note: avdevice_list_input_sources is not implemented for avfoundation.
        // Instead, run the following command to find an appropriate device name.
        // $ ffmpeg -f avfoundation -list_devices true -i "default" -v 1000
        // let device_filename_str = format!("{}:", video_filename);

        let video_size_str = format!("{}x{}", frame_dims.0, frame_dims.1);
        let device_filename = CString::new(device_filename_str).unwrap();
        let mut open_options = ffmpeg_utils::create_av_dict(vec!(
            // ("list_devices", "1", 0),
            // ("video_device_index", "0", 0),
            // ("audio_device_index", "0", 0),
            ("pixel_format", "uyvy422", 0),
            // ("framerate", "14.999993", 0),
            // ("video_size", "1280x720", 0),
            // ("video_size", "1920x1080", 0),
            // ("framerate", "30.000030", 0),
            ("framerate", framerate_str, 0),
            ("video_size", &video_size_str, 0),
            // ("capture_cursor", "1", 0),
            // ("capture_mouse_clicks", "1", 0),
        ));
        println!("device_filename: {:?}", device_filename);
        println!("open_options: {}", ffmpeg_utils::av_dict_string(open_options));

        // let mut format_context = ffmpeg_sys::avformat_alloc_context();
        let mut format_context : *mut ffmpeg_sys::AVFormatContext = ptr::null_mut();

        println!("format_context: {:?}", format_context);

        let open_error = ffmpeg_sys::avformat_open_input(
            &mut format_context,
            device_filename.as_ptr(),
            input_format,
            // &mut open_options
            &mut open_options
        );
        println!("format_context: {:?}", format_context);

        let num_rejected_options = ffmpeg_sys::av_dict_count(&*open_options);
        if num_rejected_options > 0 {
            println!("num rejected open_options: {}", num_rejected_options);
            println!("rejected open_options: {}", ffmpeg_utils::av_dict_string(open_options));
        }

        ffmpeg_sys::av_dict_free(&mut open_options);

        if open_error < 0 {
            ffmpeg_utils::log_av_error("open_error", open_error);
            return Err(FfmpegError::from_av_error(open_error));
        }

        let info_options = ptr::null_mut();
        let stream_info_error = ffmpeg_sys::avformat_find_stream_info(format_context, info_options);

        if stream_info_error < 0 {
            ffmpeg_utils::log_av_error("stream_info_error", stream_info_error);
            return Err(FfmpegError::from_av_error(stream_info_error));
        }

        Ok(format_context)
    }

    unsafe fn get_codec_context(format_context : *mut ffmpeg_sys::AVFormatContext, stream_index : isize) -> Result<*mut ffmpeg_sys::AVCodecContext, FfmpegError> {
        let stream = (*(*format_context).streams).offset(stream_index as isize);
        let decoder_context = (*stream).codec;

        let decoder = ffmpeg_sys::avcodec_find_decoder((*decoder_context).codec_id);
        if decoder.is_null() {
            // let type_str = ffmpeg_utils::cstring_to_str_safe(ffmpeg_sys::av_get_media_type_string(kind));
            // let message = format!("avcodec_find_decoder: decoder not found for {}.", type_str);
            let message = "avcodec_find_decoder: decoder not found.";
            println!("{}", message);
            return Err(FfmpegError::from_message(message));
        }

        println!("decoder.long_name: {:?}", ffmpeg_utils::cstring_to_str_safe((*decoder).long_name));

        // Initialize the decoder:
        // let mut codec_options = ptr::null_mut();
        let mut codec_options = ffmpeg_utils::create_av_dict(vec!(
            ("refcounted_frames", "0", 0),
        ));
        // println!("codec_options: {}", ffmpeg_utils::av_dict_string(codec_options));
        let decoder_open_error = ffmpeg_sys::avcodec_open2(decoder_context, decoder, &mut codec_options);
        // println!("rejected codec_options: {}", ffmpeg_utils::av_dict_string(codec_options));
        if decoder_open_error < 0 {
            // let type_str = ffmpeg_utils::cstring_to_str_safe(ffmpeg_sys::av_get_media_type_string(kind));
            // println!("avcodec_find_decoder: failed to open {} codec.", type_str);
            println!("avcodec_find_decoder: failed to open codec.");
            ffmpeg_utils::log_av_error("decoder_open_error", decoder_open_error);
            return Err(FfmpegError::from_av_error(decoder_open_error));
        }

        Ok(decoder_context)
    }

    unsafe fn decode_packet(&mut self, packet : *mut ffmpeg_sys::AVPacket, got_frame : *mut i32) -> Result<(), FfmpegError> {
        // Is this a packet from the video stream?
        if (*packet).stream_index == self.stream_index as i32 {
            // println!("av_read_frame: packet.buff {:?}, cached: {}", (*packet).buf, cached);

            // Decode video frame
            let decode_error = ffmpeg_sys::avcodec_decode_video2(
                self.decoder_context,
                // self.buffers.frame_raw,
                self.frame_raw,
                got_frame,
                packet
            );

            if decode_error < 0 {
                ffmpeg_utils::log_av_error("decode_error", decode_error);
                *got_frame = 0; // got_frame is undefined if avcodec_decode_video2 returns an error.
                return Err(FfmpegError::from_av_error(decode_error));
            }

            if *got_frame != 0 {
                // println!("  Read success");

                let width = (*self.decoder_context).width;
                let height = (*self.decoder_context).height;
                let pix_fmt = (*self.decoder_context).pix_fmt;

                if (*self.frame_raw).width != width ||
                   (*self.frame_raw).height != height ||
                   (*self.frame_raw).format != pix_fmt as i32 {
                    let err_message = "ERROR: width, height, or format is incorrect";
                    let err_info = format!("(*self.buffers.frame_raw).format: {:?} != {:?}", (*self.frame_raw).format, pix_fmt);
                    let message = format!("decode_packet: {}: ({}).", err_message, err_info);
                    println!("{}", message);
                    // ffmpeg_sys::av_packet_unref(packet);
                    return Err(FfmpegError::from_message(&message));
                }
            }
        } else {
            println!("BAD FRAME!");
        }

        Ok(())
    }

    unsafe fn read_next_frame(&mut self) -> Result<(), FfmpegError> {
        let mut packet_vec: Vec<u8> = Vec::with_capacity(mem::size_of::<ffmpeg_sys::AVPacket>());
        let mut packet : *mut ffmpeg_sys::AVPacket = packet_vec.as_mut_ptr() as *mut ffmpeg_sys::AVPacket;

        // initialize packet, set data to NULL, let the demuxer fill it
        ffmpeg_sys::av_init_packet(packet);
        (*packet).data = ptr::null_mut();
        (*packet).size = 0;

        let mut result = Ok(());

        // Read frames until a valid frame is read:
        let mut got_frame = 0;
        while got_frame == 0 {
            let read_error = ffmpeg_sys::av_read_frame(self.format_context, packet);
            if read_error >= 0 {
                let decode_result = self.decode_packet(packet, &mut got_frame);
                if decode_result.is_err() {
                    // ffmpeg_utils::log_av_error("read_next_frame, decode_packet", decoded_size);
                    result = decode_result; //Err(FfmpegError::from_av_error(decoded_size))
                    break;
                }
            } else {
                ffmpeg_utils::log_av_error("read_next_frame, av_read_frame", read_error);
                result = Err(FfmpegError::from_av_error(read_error));
                break;
            }
        }

        // Free the packet's data buffers:
        ffmpeg_sys::av_free_packet(packet);
        result
    }

    pub fn get_image(&mut self) -> Result<image_ycbcr::Image, FfmpegError> {
        let mut data : Vec<u8>;
        let width;
        let height;

        unsafe {
            // Read a frame into the camera buffers:
            try!(self.read_next_frame());

            // Find the buffer size:
            let pixel_format = ffmpeg_sys::AV_PIX_FMT_UYVY422;
            width = (*self.decoder_context).width;
            height = (*self.decoder_context).height;
            let align = 1; // "the assumed linesize alignment"
            let num_bytes = ffmpeg_sys::av_image_get_buffer_size(
                pixel_format,
                width,
                height,
                align
            );

            // Create the buffer:
            data = ffmpeg_utils::make_uninitialised_vec(num_bytes as usize);

            // Copy raw frame to the buffer:
            let dst_size = num_bytes;
            let src_data = (&(*self.frame_raw).data as *const _) as *const *const u8;
            let src_linesize = &(*self.frame_raw).linesize as *const i32;
            let bytes_written = ffmpeg_sys::av_image_copy_to_buffer(
                data.as_mut_ptr(),
                dst_size,
                src_data,
                src_linesize,
                pixel_format,
                width,
                height,
                align
            );

            if bytes_written > num_bytes {
                println!("bytes_written > num_bytes : {} > {}", bytes_written, num_bytes);
                panic!("get_image, av_image_get_buffer_size: Data buffer overrun.");
            } else if bytes_written < 0 {
                ffmpeg_utils::log_av_error("get_image, av_image_copy_to_buffer", bytes_written);
                return Err(FfmpegError::from_av_error(bytes_written))
            }
        }

        Ok(image_ycbcr::Image {
            width: width as usize,
            height: height as usize,
            data: data,
        })
    }

    pub fn get_camera(video_filename : &str, framerate_str : &str, video_size : (usize, usize)) -> Result<FfmpegCamera, FfmpegError> {

        let mut camera = FfmpegCamera {
            stream_index: 0,
            format_context: ptr::null_mut(),
            decoder_context: ptr::null_mut(),
            frame_raw: ptr::null_mut(),
        };

        unsafe {
            // Register available file formats and codecs with ffmpeg.
            ffmpeg_sys::avdevice_register_all();
            ffmpeg_sys::av_register_all();

            let device_filename_str = format!("{}:", video_filename);
            let format_context = try!(Self::get_default_format_context(&device_filename_str, framerate_str, video_size));

            // Find the best stream:
            let kind = ffmpeg_sys::AVMEDIA_TYPE_VIDEO;
            let wanted_stream_nb = -1;
            let related_stream = -1;
            let ecoder_ret : *mut *mut ffmpeg_sys::AVCodec = ptr::null_mut::<*mut ffmpeg_sys::AVCodec>();
            let flags = 0;
            let stream_index = ffmpeg_sys::av_find_best_stream(
                format_context,
                kind,
                wanted_stream_nb,
                related_stream,
                ecoder_ret,
                flags
            );
            if stream_index < 0 {
                let type_str = ffmpeg_utils::cstring_to_str_safe(ffmpeg_sys::av_get_media_type_string(kind));
                println!("av_find_best_stream: Could not find {} stream in input file.", type_str);
                return Err(FfmpegError::from_av_error(stream_index));
            }
            println!("av_find_best_stream: stream_index: {:?}", stream_index);

            let decoder_context = try!(Self::get_codec_context(format_context, stream_index as isize));

            println!("av_dump_format");
            let index = 0;
            let is_output = 0;
            let device_filename = CString::new(device_filename_str).unwrap();
            ffmpeg_sys::av_dump_format(format_context, index, device_filename.as_ptr(), is_output);

            let frame_raw = ffmpeg_sys::av_frame_alloc();
            if frame_raw.is_null() {
                println!("ERROR: av_frame_alloc failure.");
                return Err(FfmpegError::from_message("ERROR: av_frame_alloc failure"));
            }

            camera.format_context = format_context;
            camera.decoder_context = decoder_context;
            camera.stream_index = stream_index as usize;
            camera.frame_raw = frame_raw;
        }

        Ok(camera)
    }


    pub fn get_default() -> Result<FfmpegCamera, FfmpegError> {
        let video_size = (640, 480);
        // let video_size = (1280, 720);
        // let video_filename = "HD Pro Webcam C920";
        let video_filename = "default";
        // let video_filename = "FaceTime HD Camera (Built-in)";
        // let video_filename = "Capture screen 0";
        // let framerate = "30.000030";
        let framerate = "29.97";

        Self::get_camera(&video_filename, framerate, video_size)
    }

}
