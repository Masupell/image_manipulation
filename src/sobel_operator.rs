use image::{DynamicImage, GenericImage, GenericImageView};

use crate::image_shader::image_shader;

// Assuming GrayScale
pub fn sobel(img: &mut DynamicImage)
{
    let (width, height) = img.dimensions();
    let img_buffer = img.to_luma8().as_raw().clone();
    
    let gx = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let gy = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    let mut gradient: Vec<(f32, f32)> = vec![(0.0, 0.0); (width * height) as usize]; // Gradient, direction

    for y in 0..height
    {
        for x in 0..width
        {
            if x > 0 && y > 0 && x < width - 1 && y < height - 1
            {
                let img_matrix = 
                [
                    [img_buffer[((y-1)*width+x-1) as usize], img_buffer[((y-1)*width+x) as usize], img_buffer[((y-1)*width+x+1) as usize]],
                    [img_buffer[(y*width+x-1) as usize], img_buffer[(y*width+x) as usize], img_buffer[(y*width+x+1) as usize]],
                    [img_buffer[((y+1)*width+x-1) as usize], img_buffer[((y+1)*width+x) as usize], img_buffer[((y+1)*width+x+1) as usize]]
                ];
                
                let result_x = convolution(gx, img_matrix) as f32;
                let result_y = convolution(gy, img_matrix) as f32;
                let result = ((result_x.powf(2.0) + result_y.powf(2.0)) as f32).sqrt();
                let dir = result_y.atan2(result_x);

                
                gradient[(y * width + x) as usize] = (result, dir);
            }
            else 
            {
                gradient[(y * width + x) as usize] = (0.0, 0.0);
            }
        }
    }

    for y in 0..height as usize
    {
        for x in 0..width as usize
        {
            if x > 0 && y > 0 && (x as u32) < width - 1 && (y as u32) < height - 1
            {
                let result = gradient[y * width as usize + x].0;
                let dir = gradient[y * width as usize + x].1;

                let degrees = dir.to_degrees();
                let simple_degrees = if degrees >= -22.5 && degrees < 22.5 || degrees < -157.5 && degrees >= 157.5
                {
                    0
                } 
                else if degrees >= 22.5 && degrees < 67.5 || degrees < -112.5 && degrees >= -157.5
                {
                    45
                } 
                else if degrees >= 67.5 && degrees < 112.5 || degrees < -67.5 && degrees >= -112.5
                {
                    90
                } 
                else if degrees >= 112.5 && degrees < 157.5 || degrees < -22.5 && degrees >= -67.5
                {
                    135
                } 
                else 
                {
                    0
                };
                

                match simple_degrees
                {
                    0 =>
                    {
                        if result > gradient[y * width as usize + x -1].0 && result > gradient[y * width as usize + x + 1].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    45 =>
                    {
                        if result > gradient[(y-1) * width as usize + x - 1].0 && result > gradient[(y+1) * width as usize + x + 1].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    90 =>
                    {
                        if result > gradient[(y-1) * width as usize + x].0 && result > gradient[(y+1) * width as usize + x].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    135 =>
                    {
                        if result > gradient[(y-1) * width as usize + x + 1].0 && result > gradient[(y+1) * width as usize + x - 1].0
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([255, 255, 255, 255]));
                        }
                        else 
                        {
                            img.put_pixel(x as u32, y as u32, image::Rgba([0, 0, 0, 255]));   
                        }
                    },
                    _ => {}
                }
            }
        }
    }
}

// fn map_to_direction(degrees)

fn convolution(kernel: [[i32; 3]; 3], matrix: [[u8; 3]; 3]) -> i32
{
    let mut sum = 0;
    for y in 0..matrix.len()
    {
        for x in 0..matrix[y].len()
        {
            sum += kernel[y][x] * matrix[y][x] as i32;
        }
    }
    sum
}

#[cfg(test)]
mod tests
{
    use super::*;

    #[test]
    pub fn run()
    {
        // let input_img = image::open("tests/swan.jpg").unwrap();

        // let blurred = input_img.blur(5.0);
        // let mut gray = blurred.grayscale();
        // sobel(&mut gray);

        // gray.save("tests/result.png").unwrap();

        // pollster::block_on(sobel_on_gpu());
        sobel_gpu();
    }
}

pub fn sobel_cpu()
{
    let input_img = image::open("tests/swan.jpg").unwrap();

    let blurred = input_img.blur(5.0);
    let mut gray = blurred.grayscale();
    sobel(&mut gray);
    gray.save("/tests/result.png").unwrap();
}
pub fn sobel_gpu()
{
    let input_img = image::open("tests/peppe_sad.png").unwrap();
    let blurred = input_img.blur(5.0);
    // let gray = blurred.grayscale();

    pollster::block_on(image_shader(blurred, "src/shader/sobel_operator.wgsl"));
}