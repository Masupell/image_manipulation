use std::io;

use image_manipulation::*;

fn main()
{
    let mut input = String::new();
    println!("1: Circle Image\n2: String Art\n");
    loop 
    {
        io::stdin().read_line(&mut input).unwrap();
        let test: Vec<char> = input.trim().chars().collect();
        if test.len() != 1
        {
            println!("Only one Number... Try Again!");
        }
        else 
        {
            if test[0] == '1'
            {
                circle_image::run();
                return;
            }
            else if test[0] == '2'
            {
                string_art::run();
                return;
            }
            else 
            {
                println!("Only 1 or 2");
            }
        }
    }
}