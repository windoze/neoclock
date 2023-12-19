use std::cmp::Ordering;

use image::{GenericImage, GenericImageView, ImageBuffer, Pixel};

use crate::{PartImage, PartPixel};

use super::blit;

pub struct ScrollIterator<P: Pixel> {
    image: ImageBuffer<P, Vec<P::Subpixel>>,
    target_width: u32,
    target_height: u32,
    delta_x: i32,
    delta_y: i32,

    orig_x: i32,
    orig_y: i32,
}

impl<P> ScrollIterator<P>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
{
    fn new<V: GenericImageView<Pixel = P>>(
        image: V,
        target_width: u32,
        target_height: u32,
        delta_x: i32,
        delta_y: i32,
    ) -> Self {
        let mut r = Self {
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
            orig_x: 0,
            orig_y: 0,
        };
        let (orig_x, orig_y) = r.get_origin();
        r.orig_x = orig_x;
        r.orig_y = orig_y;
        r
    }

    fn get_origin(&self) -> (i32, i32) {
        let orig_x: i32 = match self.delta_x.cmp(&0) {
            Ordering::Greater => 1 - (self.image.width() as i32),
            Ordering::Less => (self.target_width - 1) as i32,
            Ordering::Equal => 0,
        };

        let orig_y: i32 = match self.delta_y.cmp(&0) {
            Ordering::Greater => 1 - (self.image.height() as i32),
            Ordering::Less => (self.target_height - 1) as i32,
            Ordering::Equal => 0,
        };

        (orig_x, orig_y)
    }
}

impl<P> Iterator for ScrollIterator<P>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
{
    type Item = ImageBuffer<P, Vec<P::Subpixel>>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut img =
            ImageBuffer::<P, Vec<P::Subpixel>>::new(self.target_width, self.target_height);
        blit(&self.image, &mut img, self.orig_x, self.orig_y);

        let (nox, noy) = self.get_origin();
        // Loop back on out of rage
        self.orig_x += self.delta_x;
        if (self.orig_x <= -(self.image.width() as i32))
            || (self.orig_x >= (self.target_width as i32))
        {
            self.orig_x = nox;
        }
        self.orig_y += self.delta_y;
        if (self.orig_y <= -(self.image.height() as i32))
            || (self.orig_y >= (self.target_height as i32))
        {
            self.orig_y = noy;
        }

        Some(img)
    }
}

pub trait Scrollable<P>
where
    P: Pixel + 'static,
    P::Subpixel: 'static,
{
    fn scroll(
        &self,
        target_width: u32,
        target_height: u32,
        delta_x: i32,
        delta_y: i32,
    ) -> ScrollIterator<P>;
}

impl Scrollable<PartPixel> for PartImage {
    fn scroll(
        &self,
        target_width: u32,
        target_height: u32,
        delta_x: i32,
        delta_y: i32,
    ) -> ScrollIterator<PartPixel> {
        ScrollIterator::new(
            self.view(0, 0, self.width(), self.height()),
            target_width,
            target_height,
            delta_x,
            delta_y,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PartImage, PartPixel};

    const RED: PartPixel = image::Rgba::<u8>([255, 0, 0, 255]);
    const TRANSPARENT: PartPixel = image::Rgba::<u8>([0, 0, 0, 0]);

    fn solid_image(width: u32, height: u32, color: &PartPixel) -> PartImage {
        let mut img = PartImage::new(width, height);
        for p in img.pixels_mut() {
            p.0 = color.0;
        }
        img
    }

    #[test]
    fn test_scroll_left() {
        let img = solid_image(3, 5, &RED);
        let mut s = ScrollIterator::new(img, 5, 5, -1, 0); // Scroll left
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &RED);
        assert_eq!(image.get_pixel(4, 4), &RED);
        assert_eq!(image.get_pixel(3, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 4), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s2.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &RED);
        assert_eq!(image.get_pixel(4, 4), &RED);
        assert_eq!(image.get_pixel(3, 0), &RED);
        assert_eq!(image.get_pixel(3, 4), &RED);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s3.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &RED);
        assert_eq!(image.get_pixel(4, 4), &RED);
        assert_eq!(image.get_pixel(3, 0), &RED);
        assert_eq!(image.get_pixel(3, 4), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(2, 4), &RED);
        assert_eq!(image.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 4), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s4.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(4, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 0), &RED);
        assert_eq!(image.get_pixel(3, 4), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(2, 4), &RED);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(1, 4), &RED);
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 4), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s5.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(4, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(2, 4), &RED);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(1, 4), &RED);
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(0, 4), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s6.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(4, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(1, 4), &RED);
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(0, 4), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s7.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(4, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(0, 4), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s8.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &RED);
        assert_eq!(image.get_pixel(4, 4), &RED);
        assert_eq!(image.get_pixel(3, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(3, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 4), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s9.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(4, 0), &RED);
        assert_eq!(image.get_pixel(4, 4), &RED);
        assert_eq!(image.get_pixel(3, 0), &RED);
        assert_eq!(image.get_pixel(3, 4), &RED);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 4), &TRANSPARENT);
    }

    #[test]
    fn test_scroll_left_1() {
        let img = solid_image(5, 1, &RED);
        let mut s = ScrollIterator::new(img, 3, 1, -1, 0); // Scroll left
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-1.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-2.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-3.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-4.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-5.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-6.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(1, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-7.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s1-8.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(1, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &RED);
    }

    #[test]
    fn test_scroll_down() {
        let img = solid_image(3, 3, &RED);
        let mut s = ScrollIterator::new(img, 3, 5, 0, 1); // Scroll down
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd1.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 1), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd2.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(0, 1), &RED);
        assert_eq!(image.get_pixel(2, 1), &RED);
        assert_eq!(image.get_pixel(0, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 2), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd3.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(0, 1), &RED);
        assert_eq!(image.get_pixel(2, 1), &RED);
        assert_eq!(image.get_pixel(0, 2), &RED);
        assert_eq!(image.get_pixel(2, 2), &RED);
        assert_eq!(image.get_pixel(0, 3), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 3), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd4.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 1), &RED);
        assert_eq!(image.get_pixel(2, 1), &RED);
        assert_eq!(image.get_pixel(0, 2), &RED);
        assert_eq!(image.get_pixel(2, 2), &RED);
        assert_eq!(image.get_pixel(0, 3), &RED);
        assert_eq!(image.get_pixel(2, 3), &RED);
        assert_eq!(image.get_pixel(0, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd5.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 2), &RED);
        assert_eq!(image.get_pixel(2, 2), &RED);
        assert_eq!(image.get_pixel(0, 3), &RED);
        assert_eq!(image.get_pixel(2, 3), &RED);
        assert_eq!(image.get_pixel(0, 4), &RED);
        assert_eq!(image.get_pixel(2, 4), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd6.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 3), &RED);
        assert_eq!(image.get_pixel(2, 3), &RED);
        assert_eq!(image.get_pixel(0, 4), &RED);
        assert_eq!(image.get_pixel(2, 4), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd7.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 0), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 3), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 3), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 4), &RED);
        assert_eq!(image.get_pixel(2, 4), &RED);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/sd8.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(0, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 1), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 3), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 3), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
        let image = s.next().unwrap();
        image
            .save_with_format("/tmp/s9.png", image::ImageFormat::Png)
            .unwrap();
        assert_eq!(image.get_pixel(0, 0), &RED);
        assert_eq!(image.get_pixel(2, 0), &RED);
        assert_eq!(image.get_pixel(0, 1), &RED);
        assert_eq!(image.get_pixel(2, 1), &RED);
        assert_eq!(image.get_pixel(0, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 2), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 3), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 3), &TRANSPARENT);
        assert_eq!(image.get_pixel(0, 4), &TRANSPARENT);
        assert_eq!(image.get_pixel(2, 4), &TRANSPARENT);
    }
}
