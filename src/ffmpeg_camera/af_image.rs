extern crate arrayfire as af;

use super::Image;

pub trait AfImage : Image {
    fn af_grey(&self) -> af::Array;
}
