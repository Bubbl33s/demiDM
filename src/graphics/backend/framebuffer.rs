use std::fs::{File, OpenOptions};
use std::os::unix::io::AsRawFd;

use image::RgbaImage;
use memmap2::MmapMut;

use super::{BackgroundBackend, RenderOpts};
use crate::errors::{AuraError, AuraResult};
use crate::graphics::scaler;

const FBIOGET_VSCREENINFO: u64 = 0x4600;
const FBIOGET_FSCREENINFO: u64 = 0x4602;

#[repr(C)]
#[derive(Default)]
struct VarScreenInfo {
    xres: u32,
    yres: u32,
    xres_virtual: u32,
    yres_virtual: u32,
    xoffset: u32,
    yoffset: u32,
    bits_per_pixel: u32,
    grayscale: u32,
    red: BitfieldEntry,
    green: BitfieldEntry,
    blue: BitfieldEntry,
    transp: BitfieldEntry,
    nonstd: u32,
    activate: u32,
    height: u32,
    width: u32,
    accel_flags: u32,
    pixclock: u32,
    left_margin: u32,
    right_margin: u32,
    upper_margin: u32,
    lower_margin: u32,
    hsync_len: u32,
    vsync_len: u32,
    sync: u32,
    vmode: u32,
    rotate: u32,
    colorspace: u32,
    _reserved: [u32; 4],
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct BitfieldEntry {
    offset: u32,
    length: u32,
    msb_right: u32,
}

#[repr(C)]
#[derive(Default)]
struct FixScreenInfo {
    id: [u8; 16],
    smem_start: u64,
    smem_len: u32,
    fb_type: u32,
    type_aux: u32,
    visual: u32,
    xpanstep: u16,
    ypanstep: u16,
    ywrapstep: u16,
    line_length: u32,
    mmio_start: u64,
    mmio_len: u32,
    accel: u32,
    capabilities: u16,
    _reserved: [u16; 2],
}

#[allow(dead_code)]
pub struct FramebufferBackend {
    _device: File,
    mmap: MmapMut,
    var_info: VarScreenInfo,
    #[allow(dead_code)]
    fix_info: FixScreenInfo,
}

pub fn check_available() -> bool {
    let path = std::path::Path::new("/dev/fb0");
    path.exists()
        && std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .open(path)
            .is_ok()
}

impl FramebufferBackend {
    pub fn new() -> AuraResult<Self> {
        let device = OpenOptions::new()
            .read(true)
            .write(true)
            .open("/dev/fb0")
            .map_err(|e| AuraError::Framebuffer(format!("Failed to open /dev/fb0: {}", e)))?;

        let mut var_info = VarScreenInfo::default();
        let ret = unsafe {
            libc::ioctl(
                device.as_raw_fd(),
                FBIOGET_VSCREENINFO as libc::c_ulong,
                &mut var_info,
            )
        };
        if ret < 0 {
            return Err(AuraError::Framebuffer(
                "Failed to get var screen info".into(),
            ));
        }

        let mut fix_info = FixScreenInfo::default();
        let ret = unsafe {
            libc::ioctl(
                device.as_raw_fd(),
                FBIOGET_FSCREENINFO as libc::c_ulong,
                &mut fix_info,
            )
        };
        if ret < 0 {
            return Err(AuraError::Framebuffer(
                "Failed to get fix screen info".into(),
            ));
        }

        let mmap = unsafe {
            memmap2::MmapOptions::new()
                .len(fix_info.smem_len as usize)
                .map_mut(&device)
                .map_err(|e| AuraError::Framebuffer(format!("Failed to mmap framebuffer: {}", e)))?
        };

        Ok(Self {
            _device: device,
            mmap,
            var_info,
            fix_info,
        })
    }

    fn pixel_offset(&self, x: u32, y: u32) -> usize {
        let bytes_per_pixel = self.var_info.bits_per_pixel.div_ceil(8);
        ((y + self.var_info.yoffset) as usize) * (self.fix_info.line_length as usize)
            + ((x + self.var_info.xoffset) as usize) * (bytes_per_pixel as usize)
    }
}

impl BackgroundBackend for FramebufferBackend {
    fn name(&self) -> &'static str {
        "framebuffer"
    }

    fn is_available(&self) -> bool {
        check_available()
    }

    fn render(&mut self, image: &RgbaImage, opts: &RenderOpts) -> AuraResult<()> {
        let screen_w = self.var_info.xres;
        let screen_h = self.var_info.yres;
        let _ = opts;

        let scaled = scaler::scale_image(
            image,
            screen_w,
            screen_h,
            crate::graphics::backend::ScaleMode::Stretch,
        );

        for y in 0..screen_h {
            for x in 0..screen_w {
                let pixel = scaled.get_pixel(x, y);
                let offset = self.pixel_offset(x, y);
                if offset + 3 < self.mmap.len() {
                    self.mmap[offset] = pixel[2];
                    self.mmap[offset + 1] = pixel[1];
                    self.mmap[offset + 2] = pixel[0];
                    self.mmap[offset + 3] = pixel[3];
                }
            }
        }
        Ok(())
    }

    fn clear(&mut self) -> AuraResult<()> {
        self.mmap.fill(0);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_var_info() -> VarScreenInfo {
        VarScreenInfo {
            xres: 100,
            yres: 50,
            bits_per_pixel: 32,
            xoffset: 0,
            yoffset: 0,
            ..Default::default()
        }
    }

    fn make_test_fix_info() -> FixScreenInfo {
        FixScreenInfo {
            smem_len: 100 * 50 * 4,
            line_length: 100 * 4,
            ..Default::default()
        }
    }

    fn make_test_fb() -> FramebufferBackend {
        let var_info = make_test_var_info();
        let fix_info = make_test_fix_info();
        let device = File::open("/dev/null").unwrap();
        let mmap = memmap2::MmapOptions::new()
            .len(100 * 50 * 4)
            .map_anon()
            .unwrap();
        FramebufferBackend {
            _device: device,
            mmap,
            var_info,
            fix_info,
        }
    }

    #[test]
    fn test_pixel_offset_calculation() {
        let fb = make_test_fb();
        assert_eq!(fb.pixel_offset(0, 0), 0);
        assert_eq!(fb.pixel_offset(1, 0), 4);
        assert_eq!(fb.pixel_offset(0, 1), 400);
        assert_eq!(fb.pixel_offset(10, 5), 5 * 400 + 10 * 4);
    }

    #[test]
    fn test_pixel_offset_with_offsets() {
        let mut fb = make_test_fb();
        fb.var_info.xoffset = 5;
        fb.var_info.yoffset = 3;
        assert_eq!(fb.pixel_offset(0, 0), 3 * 400 + 5 * 4);
    }

    #[test]
    fn test_bgra_byte_ordering() {
        let mut fb = make_test_fb();
        let offset = fb.pixel_offset(0, 0);
        fb.mmap[offset] = 0xBB;
        fb.mmap[offset + 1] = 0x11;
        fb.mmap[offset + 2] = 0x22;
        fb.mmap[offset + 3] = 0xAA;
        assert_eq!(fb.mmap[offset], 0xBB);
        assert_eq!(fb.mmap[offset + 1], 0x11);
        assert_eq!(fb.mmap[offset + 2], 0x22);
        assert_eq!(fb.mmap[offset + 3], 0xAA);
    }
}
