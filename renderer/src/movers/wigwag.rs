use std::cmp::{max, min, Ordering};

use image::{GenericImage, GenericImageView, ImageBuffer, Pixel};

use crate::{PartImage, PartPixel};

use super::blit;

pub struct WigwagIterator<P: Pixel> {
    image: ImageBuffer<P, Vec<P::Subpixel>>,
    target_width: u32,
    target_height: u32,
    delta_x: i32,
    delta_y: i32,

    current_x: i32,
    current_y: i32,
    min_x: i32,
    max_x: i32,
    min_y: i32,
    max_y: i32,
}

impl<P> WigwagIterator<P>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
{
    fn new<V>(image: V, target_width: u32, target_height: u32) -> Self
    where
        V: GenericImageView<Pixel = P>,
    {
        let min_x = min(0, (target_width as i32) - (image.width() as i32));
        let max_x = max(0, (target_width as i32) - (image.width() as i32));
        let min_y = min(0, (target_height as i32) - (image.height() as i32));
        let max_y = max(0, (target_height as i32) - (image.height() as i32));
        let current_x = 0i32;
        let current_y = 0i32;
        let delta_x = match image.width().cmp(&target_width) {
            Ordering::Greater => -1,
            Ordering::Less => 1,
            Ordering::Equal => 0,
        };
        let delta_y = match image.height().cmp(&target_height) {
            Ordering::Greater => -1,
            Ordering::Less => 1,
            Ordering::Equal => 0,
        };
        Self {
            image: {
                let mut img =
                    ImageBuffer::<P, Vec<P::Subpixel>>::new(image.width(), image.height());
                img.copy_from(&image, 0, 0).unwrap();
                img
            },
            target_width,
            target_height,
            delta_x,
            delta_y,
            current_x,
            current_y,
            min_x,
            max_x,
            min_y,
            max_y,
        }
    }
}

impl<P> Iterator for WigwagIterator<P>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
{
    type Item = ImageBuffer<P, Vec<P::Subpixel>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut image =
            ImageBuffer::<P, Vec<P::Subpixel>>::new(self.target_width, self.target_height);
        blit(&self.image, &mut image, self.current_x, self.current_y);

        let new_x = self.current_x + self.delta_x;
        if new_x < self.min_x || new_x > self.max_x {
            // Out of range
            self.delta_x = -self.delta_x
        }
        self.current_x += self.delta_x;
        let new_y = self.current_y + self.delta_y;
        if new_y < self.min_y || new_y > self.max_y {
            // Out of range
            self.delta_y = -self.delta_y
        }
        self.current_y += self.delta_y;

        Some(image)
    }
}

pub trait Wigwagable<P>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
{
    fn wigwag(&self, target_width: u32, target_height: u32) -> WigwagIterator<P>;
}

impl Wigwagable<PartPixel> for PartImage {
    fn wigwag(&self, target_width: u32, target_height: u32) -> WigwagIterator<PartPixel> {
        WigwagIterator::new(self.clone(), target_width, target_height)
    }
}

#[cfg(test)]
mod tests {
    use image::ImageFormat;

    use crate::{PartImage, PartPixel};

    use super::Wigwagable;

    const RED: PartPixel = image::Rgba::<u8>([255, 0, 0, 255]);
    const TRANSPARENT: PartPixel = image::Rgba::<u8>([0, 0, 0, 0]);

    fn solid_image(width: u32, height: u32, color: &PartPixel) -> PartImage {
        let mut img = PartImage::new(width, height);
        for p in img.pixels_mut() {
            (*p).0 = color.0;
        }
        img
    }

    #[test]
    fn test_h() {
        let image = solid_image(3, 2, &RED);

        let mut w = image.wigwag(5, 2);
        let img = w.next().unwrap();
        img.save_with_format("/tmp/w1.png", ImageFormat::Png)
            .unwrap();
        assert_eq!(img.get_pixel(0, 0), &RED);
        assert_eq!(img.get_pixel(0, 1), &RED);
        assert_eq!(img.get_pixel(1, 0), &RED);
        assert_eq!(img.get_pixel(1, 1), &RED);
        assert_eq!(img.get_pixel(2, 0), &RED);
        assert_eq!(img.get_pixel(2, 1), &RED);
        assert_eq!(img.get_pixel(3, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(3, 1), &TRANSPARENT);
        assert_eq!(img.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(4, 1), &TRANSPARENT);
        let img = w.next().unwrap();
        img.save_with_format("/tmp/w2.png", ImageFormat::Png)
            .unwrap();
        assert_eq!(img.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(img.get_pixel(1, 0), &RED);
        assert_eq!(img.get_pixel(1, 1), &RED);
        assert_eq!(img.get_pixel(2, 0), &RED);
        assert_eq!(img.get_pixel(2, 1), &RED);
        assert_eq!(img.get_pixel(3, 0), &RED);
        assert_eq!(img.get_pixel(3, 1), &RED);
        assert_eq!(img.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(4, 1), &TRANSPARENT);
        let img = w.next().unwrap();
        img.save_with_format("/tmp/w3.png", ImageFormat::Png)
            .unwrap();
        assert_eq!(img.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(img.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(1, 1), &TRANSPARENT);
        assert_eq!(img.get_pixel(2, 0), &RED);
        assert_eq!(img.get_pixel(2, 1), &RED);
        assert_eq!(img.get_pixel(3, 0), &RED);
        assert_eq!(img.get_pixel(3, 1), &RED);
        assert_eq!(img.get_pixel(4, 0), &RED);
        assert_eq!(img.get_pixel(4, 1), &RED);
        let img = w.next().unwrap();
        img.save_with_format("/tmp/w4.png", ImageFormat::Png)
            .unwrap();
        assert_eq!(img.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(img.get_pixel(1, 0), &RED);
        assert_eq!(img.get_pixel(1, 1), &RED);
        assert_eq!(img.get_pixel(2, 0), &RED);
        assert_eq!(img.get_pixel(2, 1), &RED);
        assert_eq!(img.get_pixel(3, 0), &RED);
        assert_eq!(img.get_pixel(3, 1), &RED);
        assert_eq!(img.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(4, 1), &TRANSPARENT);
        let img = w.next().unwrap();
        img.save_with_format("/tmp/w5.png", ImageFormat::Png)
            .unwrap();
        assert_eq!(img.get_pixel(0, 0), &RED);
        assert_eq!(img.get_pixel(0, 1), &RED);
        assert_eq!(img.get_pixel(1, 0), &RED);
        assert_eq!(img.get_pixel(1, 1), &RED);
        assert_eq!(img.get_pixel(2, 0), &RED);
        assert_eq!(img.get_pixel(2, 1), &RED);
        assert_eq!(img.get_pixel(3, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(3, 1), &TRANSPARENT);
        assert_eq!(img.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(img.get_pixel(4, 1), &TRANSPARENT);
    }
}
