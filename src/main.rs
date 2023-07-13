use char_map::*;
use clap::*;
use pixel::*;
use read_bytes::*;
use std::fs;

mod char_map;
mod pixel;
mod read_bytes;

const SUPPORTED_BITS_PER_PIXEL: [usize; 1] = [24];
const DEFAULT_CHAR_MAP: [u8; 5] = [b'#', b'!', b'-', b'.', b' '];

const BYTES_2: usize = 2;
const BYTES_4: usize = 4;

// BITMAP file header
const SIGNATURE_OFFSET: usize = 0;
const FILE_SIZE_OFFSET: usize = SIGNATURE_OFFSET + BYTES_2;
const RESERVED_1_OFFSET: usize = FILE_SIZE_OFFSET + BYTES_4;
const RESERVED_2_OFFSET: usize = RESERVED_1_OFFSET + BYTES_2;
const PIXEL_ARRAY_OFFSET_OFFSET: usize = RESERVED_2_OFFSET + BYTES_2;

// DIB header
const DIB_HEADER_SIZE_OFFSET: usize = PIXEL_ARRAY_OFFSET_OFFSET + BYTES_4;
const IMAGE_WIDTH_OFFSET: usize = DIB_HEADER_SIZE_OFFSET + BYTES_4;
const IMAGE_HEIGHT_OFFSET: usize = IMAGE_WIDTH_OFFSET + BYTES_4;
const PLANES_OFFSET: usize = IMAGE_HEIGHT_OFFSET + BYTES_4;
const BITS_PER_PIXEL_OFFSET: usize = PLANES_OFFSET + BYTES_2;
const COMPRESSION_OFFSET: usize = BITS_PER_PIXEL_OFFSET + BYTES_2;
const IMAGE_SIZE_OFFSET: usize = COMPRESSION_OFFSET + BYTES_4;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// the bitmap file to be read
    bitmap_file: String,

    /// prints some header values
    #[arg(short = 'H', long)]
    print_header: bool,

    /// inverts the brightness
    #[arg(short, long)]
    inverse_brightness: bool,

    /// output file on which to write the art to
    #[arg(short, long)]
    output_file: Option<String>,

    /// custom characters set. odd characters ignored
    #[arg(short, long)]
    custom_charmap: Option<String>,
}

fn main() {
    let args = Args::parse();

    if args.bitmap_file.is_empty() {
        eprintln!("provide bitmap path as first argument");
        return;
    }

    let uses_custom_char_map = args.custom_charmap.is_some();
    let custom_charmap = match uses_custom_char_map {
        true => match parse_custom_char_map(&args.custom_charmap.unwrap()) {
            Ok(cc_map) => cc_map,
            Err(_) => {
                eprintln!("could not parse custom char map. example \"@ # ! - .\"");
                return;
            }
        },
        false => Vec::with_capacity(0),
    };

    // read image data
    let bytes = match fs::read(&args.bitmap_file) {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "file '{}' does not exist or is inaccessible: {}",
                &args.bitmap_file,
                err.to_string()
            );
            return;
        }
    };

    // read header
    let sig = std::str::from_utf8(&bytes[0..2]).expect("invalid bmp file");
    let file_size = read_uint32_le(&bytes, FILE_SIZE_OFFSET).unwrap();
    let pixel_array_offset = read_uint32_le(&bytes, PIXEL_ARRAY_OFFSET_OFFSET).unwrap();
    let image_width = read_uint32_le(&bytes, IMAGE_WIDTH_OFFSET).unwrap();
    let image_height = read_uint32_le(&bytes, IMAGE_HEIGHT_OFFSET).unwrap();
    let bits_per_pixel = read_uint16_le(&bytes, BITS_PER_PIXEL_OFFSET).unwrap();
    let read_image_size = read_uint32_le(&bytes, IMAGE_SIZE_OFFSET).unwrap();

    let bytes_per_pixel = (bits_per_pixel / 8) as u32;
    let padding_bytes = (image_width * bytes_per_pixel) % 4;
    let bytes_per_row = bytes_per_pixel * image_width + padding_bytes;

    let image_size = match read_image_size == 0 {
        true => bytes_per_row * image_height,
        false => read_image_size,
    };

    if args.print_header {
        println!("signature         : {}", sig);
        println!("file size         : {}", file_size);
        println!("pixel array offset: {}", pixel_array_offset);
        println!("image width       : {}", image_width);
        println!("image height      : {}", image_height);
        println!("bits per pixel    : {}", bits_per_pixel);
        println!("image size        : {}", image_size);
    }

    if !SUPPORTED_BITS_PER_PIXEL.contains(&(bits_per_pixel as usize)) {
        eprintln!(
            "image bits per pixel ({}) not supported. supported: {}",
            bits_per_pixel,
            SUPPORTED_BITS_PER_PIXEL
                .iter()
                .map(|&id| id.to_string() + ",")
                .collect::<String>()
        );
        return;
    }

    let mut raw_pixels: Vec<Pixel> = Vec::with_capacity((image_height * image_width) as usize);
    // +1 because '\n' are added
    let mut ascii_art: Vec<u8> = Vec::with_capacity((image_height * (image_width + 1)) as usize);
    ascii_art.fill(0);

    let mut i = 0;
    let mut padding_offset = 0;
    loop {
        if i / bytes_per_pixel >= image_height * image_width {
            break;
        }

        let bytes_offset = (i + padding_offset + pixel_array_offset) as usize;

        // read LE rgb pixel
        let r = bytes[bytes_offset + 2];
        let g = bytes[bytes_offset + 1];
        let b = bytes[bytes_offset];
        let pixel = Pixel { r, g, b };

        // keeps the bmp orientation (flipped vertically)
        let pixel_idx = (i / bytes_per_pixel) as usize;
        raw_pixels.insert(pixel_idx, pixel);

        if i != 0 && i * bytes_per_pixel % (bytes_per_row - padding_bytes) == bytes_per_pixel {
            padding_offset += padding_bytes;
        }

        i += bytes_per_pixel;
    }

    let char_map: Vec<u8> = match uses_custom_char_map {
        true => custom_charmap,
        false => DEFAULT_CHAR_MAP.to_vec(),
    };
    let mut newline_offset = 0;
    for y in (0..image_height).rev() {
        for x in 0..image_width {
            let pixel_idx = (y * image_width + x) as usize;
            let ascii_char_idx =
                ((image_height - y - 1) * (image_width) + x) as usize + newline_offset;

            // println!(
            //     "y:{} x:{} pixel:{} asciidx:{}",
            //     y, x, pixel_idx, ascii_char_idx
            // );

            let brightness = &raw_pixels[pixel_idx].brightness();
            let char_idx = ((char_map.len() - 1) as f64 * brightness).round() as usize;
            let current_char = match args.inverse_brightness {
                true => char_map[char_map.len() - 1 - char_idx],
                false => char_map[char_idx],
            };

            ascii_art.insert(ascii_char_idx, current_char as u8);

            if x + 1 == image_width {
                ascii_art.insert(ascii_char_idx + 1, b'\n');
            }
        }
        newline_offset += 1;
    }

    if args.output_file.is_some() {
        let output_path = args.output_file.unwrap();

        match fs::metadata(&output_path) {
            Ok(_) => {
                eprintln!("file {} already exists", output_path);
                return;
            }
            Err(_) => (),
        }

        match fs::write(&output_path, ascii_art) {
            Ok(_) => println!("wrote art to {}", output_path),
            Err(err) => eprintln!("io error: {}", err.to_string()),
        };
        return;
    }

    for c in ascii_art {
        print!("{}", c as char);
    }
}
