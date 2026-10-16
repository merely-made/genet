// Checked buffer layout and reservation accounting for surface-readback diagnostics.
// Compiled with browser-surface-readback-diagnostics; this module issues no GPU work.
use std::sync::{Arc, atomic::{AtomicU64, Ordering}};

const FRAME_LIMIT: u64 = 32 * 1024 * 1024;
const TOTAL_LIMIT: u64 = 128 * 1024 * 1024;
const COPY_ROW_ALIGNMENT: u64 = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT as u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SurfaceCaptureLimit {
    ZeroExtent,
    TextureExtent,
    Arithmetic,
    FrameBytes,
    BufferBytes,
    AggregateBytes,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SurfaceCaptureLayout {
    pub width: u32,
    pub height: u32,
    pub row_bytes: u32,
    pub padded_row_bytes: u32,
    pub padded_bytes: u64,
    pub rgba_bytes: usize,
    pub reservation_bytes: u64,
}

impl SurfaceCaptureLayout {
    pub fn checked(width:u32, height:u32, max_dimension:u32, max_buffer:u64)
        -> Result<Self,SurfaceCaptureLimit>
    {
        if width == 0 || height == 0 { return Err(SurfaceCaptureLimit::ZeroExtent); }
        if width > max_dimension || height > max_dimension {
            return Err(SurfaceCaptureLimit::TextureExtent);
        }
        let row = u64::from(width).checked_mul(4).ok_or(SurfaceCaptureLimit::Arithmetic)?;
        let padded_row = row.checked_add(COPY_ROW_ALIGNMENT - 1)
            .ok_or(SurfaceCaptureLimit::Arithmetic)? / COPY_ROW_ALIGNMENT * COPY_ROW_ALIGNMENT;
        let raw = row.checked_mul(u64::from(height)).ok_or(SurfaceCaptureLimit::Arithmetic)?;
        let padded = padded_row.checked_mul(u64::from(height)).ok_or(SurfaceCaptureLimit::Arithmetic)?;
        if raw > FRAME_LIMIT || padded > FRAME_LIMIT { return Err(SurfaceCaptureLimit::FrameBytes); }
        if padded > max_buffer { return Err(SurfaceCaptureLimit::BufferBytes); }
        let reservation = raw.checked_add(padded).ok_or(SurfaceCaptureLimit::Arithmetic)?;
        Ok(Self {
            width, height,
            row_bytes:u32::try_from(row).map_err(|_| SurfaceCaptureLimit::Arithmetic)?,
            padded_row_bytes:u32::try_from(padded_row).map_err(|_| SurfaceCaptureLimit::Arithmetic)?,
            padded_bytes:padded,
            rgba_bytes:usize::try_from(raw).map_err(|_| SurfaceCaptureLimit::Arithmetic)?,
            reservation_bytes:reservation,
        })
    }
}

// One shared budget belongs to the diagnostic capture session, across sibling
// hosts and Core replacement. Existing retained fixture/reference bytes must
// reserve tickets from this SAME budget before adding acquired-surface copies.
// Per-Core budgets alone would double the cap after replacement.
#[derive(Default)]
pub struct SurfaceCaptureBudget { reserved:AtomicU64 }
impl SurfaceCaptureBudget {
    pub fn reserve(self:&Arc<Self>, bytes:u64) -> Result<SurfaceCaptureReservation,SurfaceCaptureLimit> {
        if bytes == 0 || bytes > TOTAL_LIMIT { return Err(SurfaceCaptureLimit::AggregateBytes); }
        self.reserved.fetch_update(Ordering::AcqRel,Ordering::Acquire,|old| {
            old.checked_add(bytes).filter(|new| *new <= TOTAL_LIMIT)
        }).map_err(|_| SurfaceCaptureLimit::AggregateBytes)?;
        Ok(SurfaceCaptureReservation { budget:self.clone(), bytes })
    }
    pub fn reserved_bytes(&self) -> u64 { self.reserved.load(Ordering::Acquire) }
}

// Move-only; retained normalized pixels keep their CPU-byte reservation.
// This exposes no arbitrary release count. Buffer owners release padded bytes
// only after mapped views are dropped, unmap is requested and buffer dropped.
pub struct SurfaceCaptureReservation { budget:Arc<SurfaceCaptureBudget>, bytes:u64 }
impl SurfaceCaptureReservation {
    pub(super) fn release_buffer_bytes(&mut self, layout:SurfaceCaptureLayout) -> Result<(),SurfaceCaptureLimit> {
        if self.bytes != layout.reservation_bytes { return Err(SurfaceCaptureLimit::Arithmetic); }
        self.bytes -= layout.padded_bytes;
        self.budget.reserved.fetch_sub(layout.padded_bytes,Ordering::AcqRel);
        Ok(())
    }
}
impl Drop for SurfaceCaptureReservation {
    fn drop(&mut self) { self.budget.reserved.fetch_sub(self.bytes,Ordering::AcqRel); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exact_surface_rows_cross_alignment_without_truncation() {
        let a = SurfaceCaptureLayout::checked(63,2,16384,FRAME_LIMIT).unwrap();
        let b = SurfaceCaptureLayout::checked(64,2,16384,FRAME_LIMIT).unwrap();
        let c = SurfaceCaptureLayout::checked(65,2,16384,FRAME_LIMIT).unwrap();
        assert_eq!((a.row_bytes,a.padded_row_bytes,a.rgba_bytes),(252,256,504));
        assert_eq!((b.row_bytes,b.padded_row_bytes,b.rgba_bytes),(256,256,512));
        assert_eq!((c.row_bytes,c.padded_row_bytes,c.rgba_bytes),(260,512,520));
    }

    #[test]
    fn padding_and_device_limits_refuse_before_allocation() {
        assert_eq!(SurfaceCaptureLayout::checked(1,131073,u32::MAX,u64::MAX),Err(SurfaceCaptureLimit::FrameBytes));
        assert_eq!(SurfaceCaptureLayout::checked(1,1,16384,255),Err(SurfaceCaptureLimit::BufferBytes));
        assert_eq!(SurfaceCaptureLayout::checked(0,1,16384,u64::MAX),Err(SurfaceCaptureLimit::ZeroExtent));
        assert_eq!(SurfaceCaptureLayout::checked(16385,1,16384,u64::MAX),Err(SurfaceCaptureLimit::TextureExtent));
        assert_eq!(SurfaceCaptureLayout::checked(u32::MAX,u32::MAX,u32::MAX,u64::MAX),Err(SurfaceCaptureLimit::Arithmetic));
        let exact = SurfaceCaptureLayout::checked(16384,512,16384,FRAME_LIMIT).unwrap();
        assert_eq!((exact.padded_bytes,exact.rgba_bytes),(FRAME_LIMIT,FRAME_LIMIT as usize));
        assert_eq!(SurfaceCaptureLayout::checked(16384,513,16384,u64::MAX),Err(SurfaceCaptureLimit::FrameBytes));
    }

    #[test]
    fn retained_pixels_and_sibling_or_replacement_buffers_share_the_cap() {
        let budget = Arc::new(SurfaceCaptureBudget::default());
        let layout = SurfaceCaptureLayout::checked(16384,512,16384,FRAME_LIMIT).unwrap();
        let mut first = budget.reserve(layout.reservation_bytes).unwrap();
        let sibling = budget.reserve(layout.reservation_bytes).unwrap();
        assert!(matches!(budget.reserve(1),Err(SurfaceCaptureLimit::AggregateBytes)));
        first.release_buffer_bytes(layout).unwrap();
        assert_eq!(budget.reserved_bytes(),3 * FRAME_LIMIT);
        let replacement = budget.reserve(FRAME_LIMIT).unwrap();
        assert_eq!(budget.reserved_bytes(),TOTAL_LIMIT);
        drop(first); // Accepted retained pixels release once when their owner drops.
        drop(sibling); // Cancellation releases the complete pending reservation.
        drop(replacement);
        assert_eq!(budget.reserved_bytes(),0);
        let next = budget.reserve(TOTAL_LIMIT).unwrap();
        drop(next);
        assert_eq!(budget.reserved_bytes(),0);
    }
}
