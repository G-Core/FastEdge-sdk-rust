// this example reads file from S3 storage, which must be confgiured for the app like this:
//   "env": {
//    "ACCESS_KEY": "<access_key>",
//    "BASE_HOSTNAME": "<base_hostname>, e.g. cloud.gcore.lu",
//    "BUCKET": "<bucket>",
//    "REGION": "<region>",
//    "SECRET_KEY": "<secret_key>"
//  }
// then apply watermark from file "sample.png", which is embedded during compilation process
// and return resulting image as PNG.
// if file from S3 cannot be recognised as valid image, it is passed to caller as is

const DEFAULT_OPACITY: f32 = 1.0; // to use non-default opacity, specify OPACITY in 0-1.0 range in app env

use fastedge::{
    body::Body,
    http::{header, Error, Method, Request, Response, StatusCode},
};
use image::*;
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use std::{env, io::Cursor, time::Duration};
use url::Url;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    // embed watermark file - file must be present during compilation
    let wm_buf = include_bytes!("sample.png");

    // Filter request methods
    match req.method() {
        // Allow only GET and HEAD requests.
        &Method::GET | &Method::HEAD => (),

        // Deny anything else.
        _ => {
            return Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(header::ALLOW, "GET, HEAD")
                .body(Body::from("This method is not allowed\n"));
        }
    };

    // get filename from URL with has format <scheme>://<host>/<filename>
    let filename = req.uri().path().trim_start_matches('/');
    if filename.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Malformed request - filename expected\n"));
    }

    // construct S3 signed URL
    let (signed_url, host) = match sign_s3(filename) {
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("App misconfigured\n"))
        }
        Ok((u, h)) => (u, h),
    };

    /* Actual request to S3 */
    let s3_req = Request::builder()
        .method(Method::GET)
        .uri(signed_url.as_str())
        .header("Host", host)
        .body(Body::empty())
        .expect("error building the request");
    let rsp = match fastedge::send_request(s3_req) {
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
        }
        Ok(r) => r,
    };

    // if response is not 200, just forward it to the caller
    let (parts, body) = rsp.into_parts();
    if parts.status != StatusCode::OK {
        return Ok(Response::from_parts(parts, body));
        // if you don't want to expose S3 error to the caller, just use
        // return Response::builder()
        //     .status(StatusCode::INTERNAL_SERVER_ERROR)
        //     .body(Body::empty())
    }

    // load response as image
    let buf = body.as_bytes();
    let out_format = match guess_format(buf) {
        Ok(f) => f,
        Err(_e) =>
        // response body is not a valid image, just return it to the caller without changes
        {
            return Ok(Response::from_parts(parts, body))
        }
    };
    let img = match load_from_memory(buf) {
        Ok(i) => i,
        Err(_e) =>
        // response body is not a valid image, just return it to the caller without changes
        {
            return Ok(Response::from_parts(parts, body))
        }
    };

    // load watermark as image
    let wm_img = match load_from_memory(wm_buf.as_slice()) {
        Ok(i) => i,
        Err(_e) =>
        // should never happen
        {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Invalid watermark format\n"))
        }
    };

    // get opacity from env
    let opacity = match env::var("OPACITY").ok() {
        None => DEFAULT_OPACITY,
        Some(l) => match l.parse::<f32>() {
            Err(_) => {
                return Response::builder() // opacity is not a number
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Invalid opacity value\n"));
            }
            Ok(v) if !(0.0..=1.0).contains(&v) =>
            // opacity is not in 0-1.0 range
            {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(Body::from("Invalid opacity value\n"))
            }
            Ok(v) => v,
        },
    };

    let result = watermark(
        &img, &wm_img, 0, // X offset for watermark placement
        0, // Y offset for watermark placement
        opacity,
    );

    // convert resulting image to original format
    let mut out = Vec::new();
    let mut c = Cursor::new(&mut out);
    let _ = result.write_to(&mut c, out_format);

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, out_format.to_mime_type())
        .body(Body::from(out))
}

// Apply watermark using alpha blending
fn watermark(
    img: &DynamicImage,
    wm: &DynamicImage,
    offset_x: u32,
    offset_y: u32,
    opacity: f32,
) -> DynamicImage {
    let opacity = match opacity {
        o if o > 1.0 => 1.0,
        o if o < 0.0 => 0.0,
        _ => opacity,
    };

    let img_width = img.width();
    let img_height = img.height();

    let mut wm_width = wm.width();
    let mut wm_height = wm.height();

    if offset_x + wm_width > img_width {
        wm_width = img_width - offset_x;
    }

    if offset_y + wm_height > img_height {
        wm_height = img_height - offset_y;
    }

    let mut canvas = img.clone();

    for y in 0..wm_height {
        for x in 0..wm_width {
            let img_x = x + offset_x;
            let img_y = y + offset_y;

            let mut img_pixel = img.get_pixel(img_x, img_y);
            let wm_pixel = wm.get_pixel(x, y);

            let img_alpha = img_pixel.0[3] as f32 / 255.0;
            let img_red = img_pixel.0[0] as f32 * img_alpha;
            let img_green = img_pixel.0[1] as f32 * img_alpha;
            let img_blue = img_pixel.0[2] as f32 * img_alpha;

            let wm_alpha = wm_pixel.0[3] as f32 / 255.0 * opacity;
            let wm_red = wm_pixel.0[0] as f32 * wm_alpha;
            let wm_green = wm_pixel.0[1] as f32 * wm_alpha;
            let wm_blue = wm_pixel.0[2] as f32 * wm_alpha;

            img_pixel.0[0] = (wm_red + (1.0 - wm_alpha) * img_red) as u8;
            img_pixel.0[1] = (wm_green + (1.0 - wm_alpha) * img_green) as u8;
            img_pixel.0[2] = (wm_blue + (1.0 - wm_alpha) * img_blue) as u8;
            img_pixel.0[3] = 255;

            canvas.put_pixel(img_x, img_y, img_pixel);
        }
    }

    canvas
}

// Calculate S3 signature
fn sign_s3(fname: &str) -> anyhow::Result<(Url, String)> {
    /* read S3 access params from env */
    let access_key = env::var("ACCESS_KEY")?;
    let secret_key = env::var("SECRET_KEY")?;
    let region = env::var("REGION")?;
    let base_hostname = env::var("BASE_HOSTNAME")?;
    let bucket = env::var("BUCKET")?;
    let scheme = env::var("SCHEME").unwrap_or_else(|_| "http".to_string());

    /* set S3 request params */
    let host = region.clone() + "." + base_hostname.as_str();
    let upload_url = scheme + "://" + host.as_str();
    let parsed_url = upload_url.parse()?;
    let bucket = Bucket::new(parsed_url, UrlStyle::Path, bucket, region)?;

    let creds = Credentials::new(access_key, secret_key);
    let action = bucket.get_object(Some(&creds), fname);
    let signed_url = action.sign(Duration::from_secs(60 * 60));

    Ok((signed_url, host))
}
