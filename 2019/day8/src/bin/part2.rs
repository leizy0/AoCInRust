use day8::image::read_image;

fn main() {
    let input_path = "inputs.txt";
    let image_width = 25;
    let image_height = 6;
    let pixel_radix = 3;
    let image = match read_image(input_path, image_width, image_height, pixel_radix) {
        Ok(i) => i,
        Err(e) => {
            eprintln!(
                "Failed to read image from input file({}), get error({})",
                input_path, e
            );
            return;
        }
    };

    println!("{}", image.merge());
}
