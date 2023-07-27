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

    match image
        .layers()
        .iter()
        .min_by_key(|l| l.digit_count(0).unwrap_or(0))
        .map(|l| (l.digit_count(1).unwrap_or(0), l.digit_count(2).unwrap_or(0)))
    {
        Some((count_1, count_2)) => println!(
            "There are {} 1s and {} 2s in layer with fewest 0s, and the product is {}",
            count_1,
            count_2,
            count_1 * count_2
        ),
        None => eprintln!("Error in given image({}), there is no layer.", input_path),
    }
}
