use lambda_http::{run, service_fn, tracing, Body, Error, Request, RequestExt, Response};
use std::convert::Infallible;
use std::io::Write;
use std::path::PathBuf;

fn generate_response(prompt: String) -> Result<String, Box<dyn std::error::Error>> {
    // Choose the specific tokenizer and the architecture of the used model
    let tokenizer = llm::TokenizerSource::Embedded;
    let model_architecture = llm::ModelArchitecture::Bloom;
    // extract model, use model bloom-560m-q5_1-ggjt to output coherent text based on input text
    let model = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/bloom-560m-q5_1-ggjt.bin");
    //let model = PathBuf::from("/mini10/bloom-560m-q5_1-ggjt.bin");

    // dynamically load a language model and set it with proper arguments
    let model = llm::load_dynamic(
        Some(model_architecture),
        &model,
        tokenizer,
        Default::default(),
        llm::load_progress_callback_stdout,
    )?;

    // Start a session for the loaded model, preparing it for inference.
    let mut model_session = model.start_session(Default::default());
    let mut resp_content = String::new();
    let inference = model_session.infer::<Infallible>(
        model.as_ref(),
        &mut rand::thread_rng(), // randomize the generators to generate different text
        &llm::InferenceRequest {
            prompt: (&prompt).into(),
            parameters: &llm::InferenceParameters::default(),
            play_back_previous_tokens: false,
            maximum_token_count: Some(10),
        },
        &mut Default::default(),
        |response| match response {
            llm::InferenceResponse::PromptToken(token) | llm::InferenceResponse::InferredToken(token) => {
                print!("{token}");
                std::io::stdout().flush().unwrap();
                resp_content.push_str(&token);
                Ok(llm::InferenceFeedback::Continue)
            }
            _ => Ok(llm::InferenceFeedback::Continue),
        },
    );

    // handle the inference result
    match inference {
        Ok(_) => Ok(resp_content),
        Err(e) => Err(Box::new(e)),
    }
}

async fn handle_request(req: Request) -> Result<Response<Body>, Error> {
    // set the request from users or the default input
    // let request = req.query_string_parameters_ref().and_then(|params| params.first("text"))   // set the keyword "text" for customized input text 
    //     .unwrap_or("This is a new story");  // set the default input text for generation

    let request = req.query_string_parameters_ref().and_then(|params| params.first("text")).unwrap();   

    // print related messages for success or error
    let message = match generate_response(request.to_string()) {
        Ok(result) => result,
        Err(e) => format!("Inference error: {:?}", e),
    };
    // print out the generated response for visualzation
    println!("Response from model: {:?}", message);

    // build HTTP Response Body 
    let response = Response::builder().status(200).header("content-type", "text/html").body(Body::from(message)).map_err(Box::new)?;

    Ok(response)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing::init_default_subscriber();
    run(service_fn(handle_request)).await
}
