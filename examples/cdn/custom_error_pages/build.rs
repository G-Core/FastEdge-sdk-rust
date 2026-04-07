use base64::Engine;
use std::env;
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::Path;

/*
    Build script used to embed images and messages from the /public folder into the wasm binary.
    This happens at compile time, so the images and messages are available at runtime, as we have no filesystem access in the wasm environment.
*/

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();

    let images_dir = "./public/images";
    let messages_dir = "./public/messages";
    let image_output_file = Path::new(&out_dir).join("image_map.rs");
    let message_output_file = Path::new(&out_dir).join("message_map.rs");

    // Generate image map
    let mut image_map = String::from(
        "\npub fn get_image_map() -> HashMap<u16, &'static str> {\n    let mut map = HashMap::new();\n",
    );

    for entry in fs::read_dir(images_dir).expect("Failed to read images directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                let status_code = if file_name == "4xx" {
                    4000 // Use integer key for 4xx fallback
                } else if file_name == "5xx" {
                    5000 // Use integer key for 5xx fallback
                } else {
                    file_name
                        .parse::<u16>()
                        .expect("Failed to parse status code")
                };
                let image_bytes = fs::read(&path).expect("Failed to read image file");
                let base64_image = base64::engine::general_purpose::STANDARD.encode(image_bytes);
                let mime_type = match path.extension().and_then(|ext| ext.to_str()) {
                    Some("jpg") | Some("jpeg") => "image/jpeg",
                    Some("png") => "image/png",
                    Some("gif") => "image/gif",
                    _ => "application/octet-stream", // Fallback for unknown types
                };
                image_map.push_str(&format!(
                    "    map.insert({status_code}, \"data:{mime_type};base64,{base64_image}\");\n",
                ));
            }
        }
    }

    image_map.push_str("    map\n}\n");

    let mut image_output = File::create(image_output_file).expect("Failed to create image_map.rs");
    image_output
        .write_all(image_map.as_bytes())
        .expect("Failed to write to image_map.rs");

    // Generate message map
    let mut message_map = String::from("\npub fn get_message_map() -> HashMap<u16, (&'static str, &'static str)> {\n    let mut map = HashMap::new();\n");

    for entry in fs::read_dir(messages_dir).expect("Failed to read messages directory") {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        if path.is_file() {
            if let Some(file_name) = path.file_stem().and_then(|s| s.to_str()) {
                let status_code = if file_name == "4xx" {
                    4000 // Use integer key for 4xx fallback
                } else if file_name == "5xx" {
                    5000 // Use integer key for 5xx fallback
                } else {
                    file_name
                        .parse::<u16>()
                        .expect("Failed to parse status code")
                };

                let file = File::open(&path).expect("Failed to open message file");
                let mut lines = BufReader::new(file).lines();
                let message = lines
                    .next()
                    .unwrap_or_else(|| Ok("Unknown Error".to_string()))
                    .unwrap();
                let description = lines
                    .next()
                    .unwrap_or_else(|| Ok("No description available.".to_string()))
                    .unwrap();
                message_map.push_str(&format!(
                    "    map.insert({status_code}, (\"{message}\", \"{description}\"));\n",
                ));
            }
        }
    }

    message_map.push_str("    map\n}\n");

    let mut message_output =
        File::create(message_output_file).expect("Failed to create message_map.rs");
    message_output
        .write_all(message_map.as_bytes())
        .expect("Failed to write to message_map.rs");
}
