use image::{DynamicImage, GenericImage, GenericImageView};

pub fn run(img_path: &str)
{
    let input_img = image::open(img_path).unwrap();

    let blurred = input_img.blur(5.0);
    let mut gray = blurred.grayscale();
    sobel(&mut gray);

    gray.save("tests/blurred.png").unwrap();
}


// Assuming GrayScale
fn sobel(img: &mut DynamicImage)
{
    let gx = [[-1, 0, 1], [-2, 0, 2], [-1, 0, 1]];
    let gy = [[-1, -2, -1], [0, 0, 0], [1, 2, 1]];

    let mut gradient: Vec<Vec<(f32, f32)>> = Vec::new(); // Gradient, direction
    let mut max = 0.0;

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

                // if result > 620.0
                // {
                //     img.put_pixel(x, y, image::Rgba([255, 255, 255, 255]));
                // }
                // else 
                // {
                //     img.put_pixel(x, y, image::Rgba([0, 0, 0, 255]));
                // }
            }
            else 
            {
                row.push((0.0, 0.0));
            }
        }
        gradient.push(row);
    }
    println!("{}", max);

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
            // let temp = gx_y[3 - 1 - y][3 - 1 - x] * test[y][x]; //Not sure if gx_y flipped or not, will see, guess not
            sum += gx_y[y][x] * test[y][x] as i32;
        }
    }
    sum
}

// I would have had to put it on the gpu later anyways

// fn gaussian_blur(img: &DynamicImage, sigma: f32) -> DynamicImage
// {
//     let kernel_size: u32 = 3;
//     let mut init = Vec::new();

//     for y in (-(kernel_size as i32) / 2)..=(kernel_size as i32 / 2)
//     {
//         let mut row = Vec::new();
//         for x in (-(kernel_size as i32) / 2)..=(kernel_size as i32 / 2)
//         {
//             row.push((x, y));
//         }
//         init.push(row);
//     }

//     let kernel = calculate_kernel(&mut init, sigma);
//     println!("{:?}", kernel);

//     let mut blurred_img = DynamicImage::new_rgba8(img.width(), img.height());

//     for y in 0..img.height()
//     {
//         for x in 0..img.width()
//         {
//             if x >= kernel_size/2 && y >= kernel_size/2 && x < img.width() - kernel_size/2 && y < img.height() - kernel_size/2
//             {
//                 let top_left = img.get_pixel(x-1, y-1);
//                 let top_center = img.get_pixel(x, y-1);
//                 let top_right = img.get_pixel(x+1, y-1);
//                 let center_left = img.get_pixel(x-1, y);
//                 let center = img.get_pixel(x, y);
//                 let center_right = img.get_pixel(x+1, y);
//                 let bottom_left = img.get_pixel(x-1, y+1);
//                 let bottom_center = img.get_pixel(x, y+1);
//                 let bottom_right = img.get_pixel(x+1, y+1);
//                 let pixels = [top_left, top_center, top_right, center_left, center, center_right, bottom_left, bottom_center, bottom_right];

//                 let mut r = 0.0;
//                 let mut g = 0.0;
//                 let mut b = 0.0;

//                 // let value = pixels[0].0[0] as f32 * kernel[0][0];
//                 // let value2 = pixels[1].0[0] as f32 * kernel[0][1];
//                 // let value3 = pixels[2].0[0] as f32 * kernel[0][2];
//                 // let value4 = pixels[3].0[0] as f32 * kernel[1][0];
//                 // let value5 = pixels[4].0[0] as f32 * kernel[1][1];
//                 // let value6 = pixels[5].0[0] as f32 * kernel[1][2];
//                 // let value7 = pixels[6].0[0] as f32 * kernel[2][0];
//                 // let value8 = pixels[7].0[0] as f32 * kernel[2][1];
//                 // let value9 = pixels[8].0[0] as f32 * kernel[2][2];

//                 for i in 0..3
//                 {
//                     for j in 0..3
//                     {
//                         r += pixels[i*3+j].0[0] as f32 * kernel[i][j];
//                         g += pixels[i*3+j].0[1] as f32 * kernel[i][j];
//                         b += pixels[i*3+j].0[2] as f32 * kernel[i][j];
//                     }
//                 }

//                 blurred_img.put_pixel(x, y, image::Rgba([r as u8, g as u8, b as u8, 255]));
//                 // println!("{} {} {}", r as u8, g as u8, b as u8);
//             }
//             else
//             {
//                 blurred_img.put_pixel(x, y, img.get_pixel(x, y));
//             }
//         }
//     }
//     blurred_img
// }

// fn calculate_kernel(init: &Vec<Vec<(i32, i32)>>, sigma: f32) -> Vec<Vec<f32>>
// {
//     let pi:f32 = 3.141592653;
//     let e: f32 = 2.718281828;
//     let mut kernel: Vec<Vec<f32>> = Vec::new();
//     for kernel_y in 0..init.len()
//     {
//         let mut row = Vec::new();
//         for kernel_x in 0..init[kernel_y].len()
//         {
//             let x = init[kernel_y][kernel_x].0 as f32;
//             let y = init[kernel_y][kernel_x].1 as f32;
            
//             let step_one = -(x*x+y*y)/(2.0*sigma*sigma);
//             let step_two = e.powf(step_one);
//             let step_three = 1.0/(2.0*pi*sigma*sigma);
//             row.push(step_two*step_three);
//         }
//         kernel.push(row);
//     }

//     let sum = kernel.iter().flatten().sum::<f32>();
//     println!("{}", sum);

//     for value in kernel.iter_mut().flatten()
//     {
//         *value /= sum;
//     }
//     println!("{}", kernel.iter().flatten().sum::<f32>());

//     kernel
// }



#[cfg(test)]
mod tests 
{
    use super::*;

    #[test]
    fn it_works() 
    {
        run("example/peppe_sad.png");
    }// cargo test -- --nocapture
}