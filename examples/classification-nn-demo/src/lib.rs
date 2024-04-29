use std::borrow::Cow;
use std::time::Instant;

use fastedge::http::{header, Method};
use fastedge::{
    body::Body,
    http::{Error, Request, Response, StatusCode},
};

use fastedge::wasi_nn::wasi::nn::graph;
use fastedge::wasi_nn::wasi::nn::inference;
use fastedge::wasi_nn::wasi::nn::tensor;

use crate::imagenet_classes::IMAGENET_CLASSES;
use fastedge::wasi_nn::wasi::nn::inference::{GraphExecutionContext};

#[allow(dead_code)]
mod image2tensor;
mod imagenet_classes;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    match req.method() {
        // Allow POST and PUT requests.
        &Method::POST | &Method::PUT => (),

        &Method::OPTIONS => {
            return Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty());
        }

        // Deny anything else.
        _ => {
            return Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(header::ALLOW, "PUT, POST")
                .body(Body::from("This method is not allowed\n"));
        }
    };

    const MB: usize = 1 << 20;
    if req.body().len() > 2 * MB {
        return Response::builder()
            .status(StatusCode::PAYLOAD_TOO_LARGE)
            .body(Body::from("Image too large\n"));
    }

    let start = Instant::now();
    let model_name = req
        .uri()
        .query()
        .and_then(|query| {
            form_urlencoded::parse(query.as_bytes()).find_map(|(k, v)| {
                if k == "model" {
                    Some(v)
                } else {
                    None
                }
            })
        })
        .unwrap_or(Cow::Borrowed("mobilenet-v2"));

    println!("model name: {}", model_name);

    let output_buffer = match inference(model_name, req.body()) {
        Ok(ret) => ret,
        Err(error) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from(error.to_string()));
        }
    };
    // transmute array of bytes to floats
    let output_buffer = unsafe {
        core::slice::from_raw_parts(
            output_buffer.as_ptr().cast::<f32>(),
            output_buffer.len() / std::mem::size_of::<f32>(),
        )
    };

    let results = sort_results(output_buffer);
    let elapsed = Instant::now().duration_since(start);
    println!("Total execution time: {:.0?}", elapsed);

    let mut response = json::array![];
    for i in 0..results.len() {
        if results[i].1 > 0.01 {
            response
                .push(json::object! {
                    name: IMAGENET_CLASSES[results[i].0],
                    probability: format!("{0:.5}", results[i].1)
                })
                .expect("response push");
        } else {
            break;
        }
    }

    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(response.dump()))
}

/// perform inference
fn inference(model_name: Cow<str>, input: &[u8]) -> Result<tensor::TensorData, inference::Error> {
    //load graph by name already loaded and initialized in FastEdge runtime
    let graph_handle = graph::load_by_name(&model_name)?;
    let context = inference::init_execution_context(graph_handle)?;

    // Load a tensor that precisely matches the graph input tensor
    // Convert the image. If it fails just exit
    let tensor_data = image2tensor::convert_image_bytes_to_tensor_bytes(
        input,
        224,
        224,
        image2tensor::TensorType::F32,
        image2tensor::ColorOrder::Bgr,
    )
    .map_err(|error| {
        println!("convert_image_bytes_to_tensor_bytes: {}", error);
        inference::Error::RuntimeError
    })?;

    // Set inference input.
    let dimensions = [1, 3, 224, 224];
    set_input(
        context,
        0,
        tensor::TensorType::Fp32,
        &dimensions,
        tensor_data,
    )?;

    // Execute the inference.
    inference::compute(context)?;

    // Retrieve the output.
    inference::get_output(context, 0)
}

/// Set input uses the `data`, not only [u8], but also [f32], [i32], etc.
pub fn set_input<T: Sized>(
    context: GraphExecutionContext,
    index: u32,
    tensor_type: tensor::TensorType,
    dimensions: &[u32],
    data: impl AsRef<[T]>,
) -> Result<(), inference::Error> {
    let data_slice = data.as_ref();
    let buf = unsafe {
        core::slice::from_raw_parts(
            data_slice.as_ptr().cast::<u8>(),
            std::mem::size_of_val(data_slice),
        )
    };
    let tensor_handle = tensor::Tensor {
        dimensions: dimensions.to_vec(),
        tensor_type,
        data: buf.to_vec(),
    };

    inference::set_input(context, index, &tensor_handle)
}

// Sort the buffer of probabilities. The graph places the match probability for each class at the
// index for that class (e.g. the probability of class 42 is placed at buffer[42]). Here we convert
// to a wrapping InferenceResult and sort the results.
fn sort_results(buffer: &[f32]) -> Vec<InferenceResult> {
    let mut results: Vec<InferenceResult> = buffer
        .iter()
        .skip(1)
        .enumerate()
        .map(|(c, p)| InferenceResult(c, *p))
        .collect();
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).expect("sort results"));
    results
}

// A wrapper for class ID and match probabilities.
#[derive(Debug, PartialEq)]
struct InferenceResult(usize, f32);
