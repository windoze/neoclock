mod scroll;
mod wigwag;

use std::ops::{Deref, DerefMut};

use image::{GenericImageView, ImageBuffer, Pixel};
pub use scroll::{ScrollIterator, Scrollable};
pub use wigwag::{WigwagIterator, Wigwagable};

pub fn blit<Src, P, Container>(
    src: &Src,
    dest: &mut ImageBuffer<P, Container>,
    dest_x: i32,
    dest_y: i32,
)
where
    Src: GenericImageView<Pixel=P>,
    P: Pixel + 'static,
    P::Subpixel: 'static,
    Container: Deref<Target = [P::Subpixel]> + DerefMut,
{
    // Copy in range pixels
    for x in 0..src.width() {
        for y in 0..src.height() {
            let px = x as i32 + dest_x;
            let py = y as i32 + dest_y;
            if px >= 0 && px < (dest.width() as i32) && py >= 0 && py < (dest.height() as i32) {
                (*dest.get_pixel_mut(px as u32, py as u32)) = src.get_pixel(x, y);
            }
        }
    }
}
