// Output-only proposal; not applied or compiled.
use std::sync::{Arc, Weak};
use wasm_bindgen::prelude::*;
use crate::{DeviceLossMonitor, PendingRgbaReadback, ReadbackFailure, RenderCore, RgbaFrame};
use super::surface_capture_layout::{SurfaceCaptureBudget, SurfaceCaptureLayout, SurfaceCaptureLimit, SurfaceCaptureReservation};

#[derive(Clone, Debug)]
pub enum SurfaceReadbackFailure {
    Unsupported(&'static str),
    Properties(&'static str),
    Limit(SurfaceCaptureLimit),
    DeviceLost(crate::DeviceLossNotice),
    CoreRetired,
    Deadline,
    WrongFrame,
    FrameRejected(crate::RasterizeFailure),
    Mapping(ReadbackFailure),
    Cancelled(SurfaceCaptureCancellation),
    AlreadyCollected,
}
impl std::fmt::Display for SurfaceReadbackFailure {
    fn fmt(&self,f:&mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f,"surface readback refused: {self:?}") }
}
impl std::error::Error for SurfaceReadbackFailure {}
#[derive(Clone, Copy, Debug)]
pub enum SurfaceCaptureCancellation { Source, Viewport, Surface, Hidden, Disposed, Explicit }

#[derive(Clone, Debug)]
pub struct SurfaceCaptureFrameIdentity(Arc<()>);
impl PartialEq for SurfaceCaptureFrameIdentity {
    fn eq(&self, other:&Self) -> bool { Arc::ptr_eq(&self.0,&other.0) }
}
impl Eq for SurfaceCaptureFrameIdentity {}
impl SurfaceCaptureFrameIdentity {
    pub(crate) fn new() -> Self { Self(Arc::new(())) }
    /// Process-local diagnostic label. Retaining the identity prevents ABA.
    pub fn diagnostic_label(&self) -> String { format!("{:p}",Arc::as_ptr(&self.0)) }
}

/// Copied bounded observations, not an independently forgeable acceptance token.
#[derive(Clone, Debug)]
pub struct BrowserFrameScopeObservation {
    pub filter:Option<String>, pub status:Option<String>, pub error:Option<bool>, pub message:String,
}
#[derive(Clone, Debug)]
pub struct BrowserFrameQualityObservation {
    pub frame:SurfaceCaptureFrameIdentity, pub generation:u64, pub phase:u32,
    pub rows:Option<Vec<BrowserFrameScopeObservation>>, pub actual_row_count:u32,
    pub wrapper_rejected:bool, pub zero_classic_fifo_observed:bool,
}

#[derive(Clone, Debug)]
pub struct SurfaceCaptureConfiguration {
    pub(crate) advertised_usage:u32,
    pub(crate) requested_usage:u32,
    pub(crate) format:wgpu::TextureFormat,
    pub(crate) view_format:wgpu::TextureFormat,
    pub(crate) color_space:wgpu::SurfaceColorSpace,
    pub(crate) alpha_mode:wgpu::CompositeAlphaMode,
    pub(crate) width:u32,
    pub(crate) height:u32,
    // Constructed only by the owned WindowSurface diagnostic configuration.
    _private:(),
}
impl SurfaceCaptureConfiguration {
    pub fn advertised_usage(&self) -> u32 { self.advertised_usage }
    pub fn requested_usage(&self) -> u32 { self.requested_usage }
    pub fn format(&self) -> wgpu::TextureFormat { self.format }
    pub fn view_format(&self) -> wgpu::TextureFormat { self.view_format }
    pub fn color_space(&self) -> wgpu::SurfaceColorSpace { self.color_space }
    pub fn alpha_mode(&self) -> wgpu::CompositeAlphaMode { self.alpha_mode }
    pub fn extent(&self) -> (u32,u32) { (self.width,self.height) }
    pub(crate) fn from_window(
        advertised_usage:u32, config:&wgpu::SurfaceConfiguration,
    ) -> Result<Self,SurfaceReadbackFailure> {
        if !matches!(config.format,wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Bgra8Unorm)
            || !config.view_formats.is_empty() {
            return Err(SurfaceReadbackFailure::Unsupported("non-sRGB RGBA8/BGRA8 base and view only"));
        }
        if !matches!(config.color_space,wgpu::SurfaceColorSpace::Auto | wgpu::SurfaceColorSpace::Srgb) {
            return Err(SurfaceReadbackFailure::Unsupported("display-referred sRGB encoding only"));
        }
        if !matches!(config.alpha_mode,wgpu::CompositeAlphaMode::Opaque | wgpu::CompositeAlphaMode::PreMultiplied) {
            return Err(SurfaceReadbackFailure::Unsupported("explicit opaque or premultiplied compositor alpha only"));
        }
        if config.usage != (wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC) {
            return Err(SurfaceReadbackFailure::Unsupported("diagnostic usage must retain render attachment and copy source"));
        }
        Ok(Self { advertised_usage, requested_usage:config.usage.bits(), format:config.format,
            view_format:config.format, color_space:config.color_space, alpha_mode:config.alpha_mode,
            width:config.width,height:config.height,_private:() })
    }
}

/// A synchronous configuration call observation, not GPU scope acceptance.
#[derive(Clone, Debug)]
pub struct SurfaceCapturePreparation {
    configuration:SurfaceCaptureConfiguration,
    did_configure:bool,
}
impl SurfaceCapturePreparation {
    pub(crate) fn new(configuration:SurfaceCaptureConfiguration,did_configure:bool) -> Self {
        Self { configuration,did_configure }
    }
    pub fn configuration(&self) -> &SurfaceCaptureConfiguration { &self.configuration }
    pub fn did_configure(&self) -> bool { self.did_configure }
}

#[derive(Clone, Debug)]
pub struct AcquiredSurfaceMetadata {
    pub configuration:SurfaceCaptureConfiguration,
    pub observed_usage:u32,
    pub observed_format:String,
    pub observed_width:u32,
    pub observed_height:u32,
    pub generation:u64,
    pub quality_phase:u32,
    pub frame:SurfaceCaptureFrameIdentity,
    pub layout:SurfaceCaptureLayout,
    // Raw coverage alpha is retained even when opaque canvas ignores it.
    pub normalization:&'static str,
}

// No raw texture, Core, Host or renderer capability escapes via this helper.
#[wasm_bindgen(inline_js = r#"
export function genet_observe_acquired_surface(texture) {
  const width=texture.width, height=texture.height, usage=texture.usage;
  const samples=texture.sampleCount, layers=texture.depthOrArrayLayers;
  if (![width,height,usage,samples,layers].every(Number.isSafeInteger))
    throw new Error('nonintegral acquired texture properties');
  return {width,height,usage,samples,layers,format:String(texture.format),dimension:String(texture.dimension)};
}
"#)]
extern "C" {
    #[wasm_bindgen(catch)]
    fn genet_observe_acquired_surface(texture:&JsValue) -> Result<JsValue,JsValue>;
}

#[derive(Clone)]
pub(crate) struct SurfaceCaptureContext {
    pub core:Weak<RenderCore>,
    pub loss:Arc<DeviceLossMonitor>,
    pub generation:u64,
    pub quality_phase:u32,
    pub frame:SurfaceCaptureFrameIdentity,
    pub deadline:web_time::Instant,
}
impl SurfaceCaptureContext {
    fn check(&self) -> Result<Arc<RenderCore>,SurfaceReadbackFailure> {
        if let Some(loss)=self.loss.pending_notice() { return Err(SurfaceReadbackFailure::DeviceLost(loss)); }
        let core=self.core.upgrade().ok_or(SurfaceReadbackFailure::CoreRetired)?;
        if let Some(loss)=core.device_failure() { return Err(SurfaceReadbackFailure::DeviceLost(loss)); }
        if core.generation()!=self.generation { return Err(SurfaceReadbackFailure::WrongFrame); }
        if web_time::Instant::now()>=self.deadline { return Err(SurfaceReadbackFailure::Deadline); }
        Ok(core)
    }
}

/// Private until the exact phase2 token is accepted. No polling/public pixel API.
pub struct PendingSurfaceReadback {
    inner:Option<PendingRgbaReadback>,
    context:SurfaceCaptureContext,
    metadata:AcquiredSurfaceMetadata,
    reservation:Option<SurfaceCaptureReservation>,
}
impl PendingSurfaceReadback {
    pub(crate) fn start(
        context:SurfaceCaptureContext, frame:&wgpu::SurfaceTexture,
        configuration:&SurfaceCaptureConfiguration,budget:Arc<SurfaceCaptureBudget>,
    ) -> Result<Self,SurfaceReadbackFailure> {
        let core=context.check()?;
        if context.quality_phase!=2 { return Err(SurfaceReadbackFailure::Unsupported("acquired surface copy requires final phase2 owner")); }
        let _creation=core.loss.begin_operation().map_err(SurfaceReadbackFailure::DeviceLost)?;
        let texture=&frame.texture;
        if core.device().as_webgpu().is_none() { return Err(SurfaceReadbackFailure::Unsupported("actual WebGPU only")); }
        let raw=texture.as_webgpu().ok_or(SurfaceReadbackFailure::Unsupported("acquired texture is not actual WebGPU"))?;
        let properties=genet_observe_acquired_surface(raw.as_ref())
            .map_err(|_| SurfaceReadbackFailure::Properties("raw acquired texture observation threw"))?;
        let number=|name:&str| -> Result<u32,SurfaceReadbackFailure> {
            let value=js_sys::Reflect::get(&properties,&JsValue::from_str(name))
                .ok().and_then(|v|v.as_f64()).ok_or(SurfaceReadbackFailure::Properties("missing acquired numeric property"))?;
            if value<0. || value>f64::from(u32::MAX) || value.fract()!=0. {
                return Err(SurfaceReadbackFailure::Properties("acquired numeric property outside u32"));
            }
            Ok(value as u32)
        };
        let string=|name:&str| -> Result<String,SurfaceReadbackFailure> {
            js_sys::Reflect::get(&properties,&JsValue::from_str(name)).ok().and_then(|v|v.as_string())
                .ok_or(SurfaceReadbackFailure::Properties("missing acquired string property"))
        };
        let width=number("width")?;let height=number("height")?;let usage=number("usage")?;
        let format=string("format")?;
        let expected_format=match configuration.format {
            wgpu::TextureFormat::Rgba8Unorm=>"rgba8unorm",
            wgpu::TextureFormat::Bgra8Unorm=>"bgra8unorm",
            _=>return Err(SurfaceReadbackFailure::Unsupported("unsupported capture base format")),
        };
        if width!=configuration.width || height!=configuration.height || width!=texture.width() || height!=texture.height()
            || number("samples")?!=1 || number("layers")?!=1 || string("dimension")?!="2d"
            || format!=expected_format || texture.format()!=configuration.format
            || texture.sample_count()!=1 || texture.dimension()!=wgpu::TextureDimension::D2 {
            return Err(SurfaceReadbackFailure::Properties("raw acquired texture does not match current configuration"));
        }
        let required=(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC).bits();
        if configuration.requested_usage!=required || usage!=configuration.requested_usage
            || !texture.usage().contains(wgpu::TextureUsages::COPY_SRC) {
            return Err(SurfaceReadbackFailure::Unsupported("raw acquired texture does not permit COPY_SRC"));
        }
        let limits=core.device().limits();
        let layout=SurfaceCaptureLayout::checked(width,height,limits.max_texture_dimension_2d,limits.max_buffer_size)
            .map_err(SurfaceReadbackFailure::Limit)?;
        let reservation=budget.reserve(layout.reservation_bytes).map_err(SurfaceReadbackFailure::Limit)?;
        context.check()?;
        let buffer=core.device().create_buffer(&wgpu::BufferDescriptor {
            label:Some("Genet actual acquired surface readback"),size:layout.padded_bytes,
            usage:wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,mapped_at_creation:false,
        });
        let mut encoder=core.device().create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label:Some("Genet before-present acquired surface copy"),
        });
        encoder.copy_texture_to_buffer(wgpu::TexelCopyTextureInfo { texture,mip_level:0,
            origin:wgpu::Origin3d::ZERO,aspect:wgpu::TextureAspect::All },
            wgpu::TexelCopyBufferInfo { buffer:&buffer,layout:wgpu::TexelCopyBufferLayout {
                offset:0,bytes_per_row:Some(layout.padded_row_bytes),rows_per_image:Some(height),
            } },wgpu::Extent3d {width,height,depth_or_array_layers:1});
        let submission=core.queue().submit([encoder.finish()]);
        let (send,mapped)=std::sync::mpsc::channel();
        buffer.slice(..).map_async(wgpu::MapMode::Read,move |result| { let _=send.send(result); });
        // The outer owner already holds monitor admission until resources are
        // registered. PendingRgbaReadback cancellation runs before host wake.
        let monitor:&Arc<DeviceLossMonitor>=&core.loss;
        let inner=PendingRgbaReadback::new(core.device().clone(),buffer,mapped,submission,
            width,height,layout.padded_row_bytes,monitor);
        let metadata=AcquiredSurfaceMetadata { configuration:configuration.clone(),observed_usage:usage,
            observed_format:format,observed_width:width,observed_height:height,generation:context.generation,
            quality_phase:context.quality_phase,frame:context.frame.clone(),layout,normalization:"padding removed; BGRA swizzled if needed; raw encoded attachment RGBA preserved without alpha conversion" };
        let pending=Self { inner:Some(inner),context,metadata,reservation:Some(reservation) };
        // Once registered, ownership must reach the host's private slot even
        // if the original frame postcheck later refuses. No late by-value drop.
        Ok(pending)
    }
    pub(crate) fn check_accept(
        &self,core:&Arc<RenderCore>,identity:&SurfaceCaptureFrameIdentity,generation:u64,quality_phase:u32,
    ) -> Result<(),SurfaceReadbackFailure> {
        let original=self.context.check()?;
        if !Arc::ptr_eq(core,&original) || &self.context.frame!=identity || self.context.generation!=generation || self.context.quality_phase!=quality_phase || quality_phase!=2 {
            return Err(SurfaceReadbackFailure::WrongFrame);
        }
        Ok(())
    }
}
impl Drop for PendingSurfaceReadback {
    fn drop(&mut self) {
        // Internal readback abort takes handles outside its state mutex.
        if let Some(mut inner)=self.inner.take() { inner.cancel(); }
        drop(self.reservation.take());
    }
}

pub struct AcceptedSurfaceReadback { pending:Option<PendingSurfaceReadback>,completed:bool }
impl AcceptedSurfaceReadback {
    pub(crate) fn new(pending:PendingSurfaceReadback) -> Self { Self { pending:Some(pending),completed:false } }
    pub fn metadata(&self) -> Option<&AcquiredSurfaceMetadata> { self.pending.as_ref().map(|p|&p.metadata) }
    pub fn check_current(&self) -> Result<(),SurfaceReadbackFailure> {
        self.pending.as_ref().ok_or(SurfaceReadbackFailure::AlreadyCollected)?.context.check().map(|_| ())
    }
    pub fn cancel(mut self,reason:SurfaceCaptureCancellation) -> SurfaceReadbackFailure {
        drop(self.pending.take());self.completed=true;
        SurfaceReadbackFailure::Cancelled(reason)
    }
    pub fn poll(&mut self) -> Option<Result<AcquiredSurfaceFrame,SurfaceReadbackFailure>> {
        if self.completed { return Some(Err(SurfaceReadbackFailure::AlreadyCollected)); }
        let pending=self.pending.as_mut()?;
        if let Err(error)=pending.context.check() {
            self.completed=true;drop(self.pending.take());return Some(Err(error));
        }
        let result=pending.inner.as_mut().expect("owned pending readback").poll()?;
        // Actual monitor loss/deadline wins over a simultaneous map result.
        if let Err(error)=pending.context.check() {
            self.completed=true;drop(self.pending.take());return Some(Err(error));
        }
        self.completed=true;
        let mut pending=self.pending.take().expect("owned readback");
        let RgbaFrame {width,height,mut rgba}=match result {
            Ok(frame)=>frame,
            Err(error)=>return Some(Err(SurfaceReadbackFailure::Mapping(error))),
        };
        if rgba.len()!=pending.metadata.layout.rgba_bytes || width!=pending.metadata.observed_width
            || height!=pending.metadata.observed_height {
            return Some(Err(SurfaceReadbackFailure::Properties("mapped extent or byte count mismatch")));
        }
        if pending.metadata.configuration.format==wgpu::TextureFormat::Bgra8Unorm {
            for (i,pixel) in rgba.chunks_exact_mut(4).enumerate() {
                if i % 4096 == 0 {
                    if let Err(error)=pending.context.check() { return Some(Err(error)); }
                }
                pixel.swap(0,2);
            }
        }
        if let Err(error)=pending.context.check() { return Some(Err(error)); }
        drop(pending.inner.take()); // Mapped view was dropped/unmapped by collection.
        let mut reservation=pending.reservation.take().expect("owned capture reservation");
        if let Err(error)=reservation.release_buffer_bytes(pending.metadata.layout) {
            return Some(Err(SurfaceReadbackFailure::Limit(error)));
        }
        Some(Ok(AcquiredSurfaceFrame {metadata:pending.metadata.clone(),context:pending.context.clone(),rgba,_reservation:reservation}))
    }
}
/// Move-only CPU output. Its raw-byte credit survives retained artifact storage.
pub struct AcquiredSurfaceFrame {
    metadata:AcquiredSurfaceMetadata,
    context:SurfaceCaptureContext,
    rgba:Vec<u8>,
    _reservation:SurfaceCaptureReservation,
}
impl AcquiredSurfaceFrame {
    /// Call at success delivery; historical exported pixels remain immutable.
    pub fn check_current(&self) -> Result<(),SurfaceReadbackFailure> { self.context.check().map(|_| ()) }
    pub fn metadata(&self) -> &AcquiredSurfaceMetadata { &self.metadata }
    pub fn rgba(&self) -> &[u8] { &self.rgba }
}
