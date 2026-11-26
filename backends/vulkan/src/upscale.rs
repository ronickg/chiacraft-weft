//! Temporal upscalers behind `Cmd::Upscale`: NVIDIA's DLSS through Streamline (`streamline.rs`,
//! the `streamline` feature, Windows), through NGX directly (`ngx.rs`, the `ngx` feature, Linux),
//! or a stand-in that only scales the picture up
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

#[cfg_attr(not(any(feature = "streamline", feature = "ngx")), allow(dead_code))]
pub(crate) struct Inputs {
    pub color: Image,
    pub depth: Image,
    pub motion: Image,
    pub output: Image,
    pub normals: Option<Image>,
    pub albedo: Option<Image>,
    pub specular: Option<Image>,
    /// How far the specular rays went (blocks, in r), from the path tracer.
    pub hit_distance: Option<Image>,
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
/// Or NGX (which only adds extensions to the instance and device the backend makes).
pub(crate) enum Early {
    None,
    #[cfg(feature = "streamline")]
    Streamline(crate::streamline::Streamline),
    #[cfg(feature = "ngx")]
    Ngx(crate::ngx::Early),
}

pub(crate) fn early() -> Early {
    #[cfg(feature = "streamline")]
    if let Some(sl) = crate::streamline::Streamline::load() {
        return Early::Streamline(sl);
    }
    #[cfg(feature = "ngx")]
    if let Some(ngx) = crate::ngx::Early::load() {
        return Early::Ngx(ngx);
    }
    Early::None
}

impl Early {
    /// Instance extensions the upscaler needs, of those the loader has (`available`).
    pub fn instance_extensions(&self, available: &[vk::ExtensionProperties]) -> Vec<*const std::ffi::c_char> {
        let _ = available;
        match self {
            #[cfg(feature = "ngx")]
            Early::Ngx(n) => n.instance_extensions(available),
            _ => Vec::new(),
        }
    }

    /// Device extensions it needs on `pdev`, of those it has and `already` doesn't hold.
    ///
    /// # Safety
    /// `instance` and `pdev` are live.
    pub unsafe fn device_extensions(
        &mut self,
        instance: &ash::Instance,
        pdev: vk::PhysicalDevice,
        already: &[*const std::ffi::c_char],
    ) -> Vec<*const std::ffi::c_char> {
        let _ = (instance, pdev, already);
        match self {
            #[cfg(feature = "ngx")]
            Early::Ngx(n) => unsafe { n.device_extensions(instance, pdev, already) },
            _ => Vec::new(),
        }
    }
}

/// The Vulkan objects an upscaler is started on.
pub(crate) struct Made<'a> {
    pub entry: &'a ash::Entry,
    pub instance: &'a ash::Instance,
    pub pdev: vk::PhysicalDevice,
    pub device: &'a ash::Device,
}

/// Once the device is made: the upscaler the game gets, if any.
pub(crate) fn ready(early: Early, made: &Made, standin: bool) -> Option<Box<dyn Upscaler>> {
    let _ = made;
    let up: Option<Box<dyn Upscaler>> = match early {
        #[cfg(feature = "streamline")]
        Early::Streamline(sl) => sl.ready(made.pdev),
        #[cfg(feature = "ngx")]
        Early::Ngx(n) => n.ready(made.entry, made.instance, made.pdev, made.device),
        Early::None => None,
    };
    if up.is_none() && (standin || std::env::var_os("WEFT_UPSCALE_STANDIN").is_some()) {
        log::info!("upscaler: the stand-in (plain scaling)");
        return Some(Box::new(Standin));
    }
    up
}
