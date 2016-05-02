use glium;

use super::Image;
use std::borrow::Cow;

pub trait ToTexture {
    fn to_texture(&self, display: &glium::Display) -> glium::texture::Texture2d;
}

impl ToTexture for super::image_uyvy::Image {
    fn to_texture(&self, display: &glium::Display) -> glium::texture::Texture2d {
        let cow: Cow<[_]> = Cow::Borrowed(&self.data);
        // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(image.into_raw(), image_dimensions);
        let img_w = self.width as u32;
        let img_h = self.height as u32;
        let raw_image = glium::texture::RawImage2d {
            data: cow,
            width: img_w,
            height: img_h,
            format: glium::texture::ClientFormat::U8U8
        };
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();

        texture
    }
}

impl ToTexture for super::image_ycbcr::Image {
    fn to_texture(&self, display: &glium::Display) -> glium::texture::Texture2d {
        let cow: Cow<[_]> = Cow::Borrowed(&self.local_data);

        // let data = self.to_byte_vector();
        // let cow: Cow<[_]> = Cow::Borrowed(&data);

        // let cow: Cow<[_]> = Cow::Borrowed(&self.data);
        // let image = glium::texture::RawImage2d::from_raw_rgba_reversed(self.into_raw(), image_dimensions);
        let img_w = self.width as u32;
        let img_h = self.height as u32;
        let raw_image = glium::texture::RawImage2d {
            data: cow,
            width: img_w,
            height: img_h,
            format: glium::texture::ClientFormat::U8U8U8U8
        };
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();

        texture
    }
}

impl ToTexture for super::image_nv12::Image {
    fn to_texture(&self, display: &glium::Display) -> glium::texture::Texture2d {
        let cow: Cow<[_]> = Cow::Borrowed(&self.local_data);

        let img_w = self.width as u32;
        let img_h = self.height as u32;
        let raw_image = glium::texture::RawImage2d {
            data: cow,
            width: img_w,
            height: img_h,
            format: glium::texture::ClientFormat::U8
        };
        let texture = glium::texture::Texture2d::new(display, raw_image).unwrap();

        texture
    }
}
