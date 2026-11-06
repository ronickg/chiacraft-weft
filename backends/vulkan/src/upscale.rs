//! Temporal upscalers behind `Cmd::Upscale`: NVIDIA's DLSS through Streamline (`streamline.rs`,
//! the `streamline` feature, Windows), or a stand-in that only scales the picture up
//! (`WEFT_UPSCALE_STANDIN=1`), for checking everything around it on any GPU.

use ash::vk;
use weft_render::{UpscaleMode, UpscaleParams, UpscalerInfo};

/// A texture as an upscaler is told about it.
#[cfg_attr(not(feature = "streamline"), allow(dead_code))]
pub(crate) struct Image {
    pub image: vk::Image,
    pub view: vk::ImageView,
    pub memory: vk::DeviceMemory,
    /// Its layout now (the inputs shader-readable, the output `GENERAL`).
    pub layout: vk::ImageLayout,
    pub format: vk::Format,
    pub width: u32,
    pub height: u32,
    pub usage: vk::ImageUsageFlags,
}

#[cfg_attr(not(feature = "streamline"), allow(dead_code))]
pub(crate) struct Inputs {
    pub color: Image,
    pub depth: Image,
    pub motion: Image,
    pub output: Image,
    pub normals: Option<Image>,
    pub albedo: Option<Image>,
    pub specular: Option<Image>,
}

pub(crate) trait Upscaler: Send {
    fn info(&self) -> UpscalerInfo;
    /// The size it wants the world drawn at for `mode`, with Ray Reconstruction or not (None:
    /// `UpscaleMode::scale`'s).
    fn render_size(&mut self, mode: UpscaleMode, reconstruct: bool, output: (u32, u32)) -> Option<(u32, u32)>;
    /// Records the upscale into `cmd`. False when it could not: the backend then scales the
    /// colour up plainly, so the frame still has a picture.
    ///
    /// # Safety
    /// The images and `cmd` are live, and `cmd` is recording outside a render pass.
    unsafe fn record(&mut self, cmd: vk::CommandBuffer, inputs: &Inputs, params: &UpscaleParams) -> bool;
}

/// Scales the picture up and nothing else (the backend's plain blit), in every mode, Ray
/// Reconstruction too (its guides are drawn); no models, no DLSS 5.
pub(crate) struct Standin;

impl Upscaler for Standin {
    fn info(&self) -> UpscalerInfo {
        let modes = UpscaleMode::ALL
            .iter()
            .filter(|m| **m != UpscaleMode::Off)
            .fold(0, |bits, m| bits | 1 << *m as u32);
        UpscalerInfo { name: "stand-in (plain scaling, core:upscale_standin)".into(), modes, reconstruct: true, ..Default::default() }
    }

    fn render_size(&mut self, _mode: UpscaleMode, _reconstruct: bool, _output: (u32, u32)) -> Option<(u32, u32)> {
        None
    }

    unsafe fn record(&mut self, _cmd: vk::CommandBuffer, _inputs: &Inputs, _params: &UpscaleParams) -> bool {
        false
    }
}

/// Before the Vulkan instance: Streamline (which then also stands in for Vulkan's loader), if
/// built with it and its files are there.
pub(crate) enum Early {
    None,
    #[cfg(feature = "streamline")]
    Streamline(crate::streamline::Streamline),
}

pub(crate) fn early() -> Early {
    #[cfg(feature = "streamline")]
    if let Some(sl) = crate::streamline::Streamline::load() {
        return Early::Streamline(sl);
    }
    Early::None
}

/// Once the device is made: the upscaler the game gets, if any.
pub(crate) fn ready(early: Early, pdev: vk::PhysicalDevice, standin: bool) -> Option<Box<dyn Upscaler>> {
    let _ = pdev;
    let up: Option<Box<dyn Upscaler>> = match early {
        #[cfg(feature = "streamline")]
        Early::Streamline(sl) => sl.ready(pdev),
        Early::None => None,
    };
    if up.is_none() && (standin || std::env::var_os("WEFT_UPSCALE_STANDIN").is_some()) {
        log::info!("upscaler: the stand-in (plain scaling)");
        return Some(Box::new(Standin));
    }
    up
}
