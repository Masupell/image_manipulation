use image::{DynamicImage, GenericImage, GenericImageView};

// Assuming GrayScale
pub fn sobel(img: &mut DynamicImage)
{
    let gx = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let gy = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    let mut gradient: Vec<Vec<(f32, f32)>> = Vec::new(); // Gradient, direction

    for y in 0..img.height()
    {
        let mut row = Vec::new();
        for x in 0..img.width()
        {
            if x > 0 && y > 0 && x < img.width() - 1 && y < img.height() - 1
            {
                let img_matrix = 
                [
                    [img.get_pixel(x-1, y-1).0[0], img.get_pixel(x, y-1).0[0], img.get_pixel(x+1, y-1).0[0]],
                    [img.get_pixel(x-1, y).0[0], img.get_pixel(x, y).0[0], img.get_pixel(x+1, y).0[0]],
                    [img.get_pixel(x-1, y+1).0[0], img.get_pixel(x, y+1).0[0], img.get_pixel(x+1, y+1).0[0]]
                ];
                
                
                let result_x = convolution(gx, img_matrix) as f32;
                let result_y = convolution(gy, img_matrix) as f32;
                let result = ((result_x.powf(2.0) + result_y.powf(2.0)) as f32).sqrt();
                let dir = result_y.atan2(result_x);

                row.push((result, dir));
            }
            else 
            {
                row.push((0.0, 0.0));
            }
        }
        gradient.push(row);
    }

    for y in 0..img.height() as usize
    {
        for x in 0..img.width() as usize
        {
            if x > 0 && y > 0 && (x as u32) < img.width() - 1 && (y as u32) < img.height() - 1
            {
                let result = gradient[y as usize][x as usize].0;
                let dir = gradient[y as usize][x as usize].1;

                // println!("{}", direction);

                let degrees = dir.to_degrees();
                // println!("{}", degrees);
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
                        if result > gradient[y][x-1].0 && result > gradient[y][x+1].0
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
                        if result > gradient[y-1][x-1].0 && result > gradient[y+1][x+1].0
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
                        if result > gradient[y-1][x].0 && result > gradient[y+1][x].0
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
                        if result > gradient[y-1][x+1].0 && result > gradient[y+1][x-1].0
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

fn convolution(gx_y: [[i32; 3]; 3], test: [[u8; 3]; 3]) -> i32
{
    let mut sum = 0;
    for y in 0..test.len()
    {
        for x in 0..test[y].len()
        {
            sum += gx_y[y][x] * test[y][x] as i32;
        }
    }
    sum
}