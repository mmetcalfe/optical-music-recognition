# Optical music recognition (OMR) project

Note: does not (yet) actually perform OMR.


## Setup instructions:

Install dependencies:

    $ brew install rust
    $ brew install ffmpeg --with-ffplay # Install FFmpeg
    $ brew install homebrew/science/arrayfire

Clone the `ffmpeg-sys` repository: https://crates.io/crates/ffmpeg-sys.
The current version from crates.io detects your ffmpeg config incorrectly, which can lead to weird behaviour and memory corruption.


## FFmpeg test commands:

Command to find camera device name:

    $ ffmpeg -f avfoundation -list_devices true -i "" -v 1000


Command to take a grayscale photo:

http://stackoverflow.com/a/19524005/3622526

    ffmpeg -f avfoundation -an -r 30.000030 -s 1920x1080 -pix_fmt uyvy422 -video_device_index 0 -i "" -vframes 1 -pix_fmt gray out.bmp -v 100

# TODO List:

1. Use detected staff lines in both frames to constrain the homography estimation.
Options:
  - Require the lines to be parallel
  - Require one or more lines to be collinear

2. Detect bar lines, and use the fact that they are perpendicular to stafflines to rectify the camera images.

3. Merge the aligned, rectified frames into a global 'map' of the manuscript. Where images overlap, attempt to keep the highest quality portion of the image
(i.e. the least distorted by the homography, the largest scale, or the least blurry).
Could investigate super-resolution techniques.

4. Maybe perform staffline removal on the resulting image.

5. Detect musical objects (investigate different patch/shape descriptors + deep learning methods).
