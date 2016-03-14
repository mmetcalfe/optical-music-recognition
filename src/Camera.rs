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
use std::ffi::CStr;
use std::ffi::CString;
use std::os::raw::c_char;

// From: https://doc.rust-lang.org/std/ffi/struct.CStr.html
fn cstring_to_str_safe(c_string : *const c_char) -> String {
    unsafe {
        CStr::from_ptr(c_string).to_string_lossy().into_owned()
    }
}

pub struct Camera;

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

impl Camera {
    unsafe fn initialise_default_device() {

        // println!("{:?}", cstring_to_str_safe(ffmpeg_sys::avdevice_configuration()));
        // Register available file formats and codecs with ffmpeg.
        ffmpeg_sys::avdevice_register_all();
        ffmpeg_sys::av_register_all();

        // Get the avfoundation format:
        let format_name = CString::new("avfoundation").unwrap();
        let input_format = ffmpeg_sys::av_find_input_format(format_name.as_ptr());
        println!("input_format: {:?}", cstring_to_str_safe((*input_format).long_name));


        // Note the trailing ':' (avfoundation input filename format is "[[VIDEO]:[AUDIO]]").
        let device_filename = CString::new("HD Pro Webcam C920 #2:").unwrap();

        // Create the dictionary of open options:
        // TODO: Refactor into a method.
        // -r 30.000030 -s 1920x1080 -pix_fmt uyvy422
        let mut open_options = create_av_dict(vec!(
            ("pixel_format", "uyvy422", 0),
            ("video_size", "1920x1080", 0),
            ("framerate", "30.000030", 0),
        ));
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
            return;
        }

        let mut info_options = ptr::null_mut();
        let stream_info_error = ffmpeg_sys::avformat_find_stream_info(format_context, &mut info_options);

        if stream_info_error < 0 {
            log_av_error("stream_info_error", stream_info_error);
            return;
        }

        println!("av_dump_format");
        let index = 0;
        let is_output = 0;
        ffmpeg_sys::av_dump_format(format_context, index, device_filename.as_ptr(), is_output);
    }

    pub fn get_default() -> Camera {
        unsafe {
            Camera::initialise_default_device();
        }

        Camera
    }
}
