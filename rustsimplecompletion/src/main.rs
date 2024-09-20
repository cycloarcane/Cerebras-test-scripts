use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    content: String,
    role: String,
}

#[derive(Serialize, Debug)]
struct ChatCompletionRequest {
    model: String,
    stream: bool,
    messages: Vec<Message>,
    temperature: i32,
    #[serde(rename = "max_tokens")]
    max_tokens: i32,
    seed: i32,
    #[serde(rename = "top_p")]
    top_p: i32,
}

fn main() {
    // Initialize the logger
    env_logger::init();

    // Get the API key from the environment variable
    let api_key = env::var("CEREBRAS_API_KEY").expect("CEREBRAS_API_KEY not set");

    // Prepare the request body
    let request_body = ChatCompletionRequest {
        model: "llama3.1-8b".to_string(),
        stream: false,
        messages: vec![Message {
            content: "Hello!".to_string(),
            role: "user".to_string(),
        }],
        temperature: 0,
        max_tokens: -1,
        seed: 0,
        top_p: 1,
    };

    // Serialize the request body to JSON
    let request_body_json = serde_json::to_string(&request_body).unwrap();
    println!("Request Body JSON: {}", request_body_json);

    // Build the HTTP client
    let client = Client::builder()
        .http1_only() // Ensure HTTP/1.1
        .build()
        .expect("Failed to build client");

    // Send the POST request
    let response = client
        .post("https://api.cerebras.ai/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("User-Agent", "YourAppName/1.0")
        .body(request_body_json)
        .send();

    // Handle the response
    match response {
        Ok(resp) => {
            let status = resp.status();
            let headers = resp.headers().clone();
            let resp_text = resp.text().unwrap_or_default();

            println!("Status: {}", status);
            println!("Headers:\n{:#?}", headers);
            println!("Response Text:\n{}", resp_text);

            if status.is_success() {
                // Attempt to parse the response JSON
                match serde_json::from_str::<serde_json::Value>(&resp_text) {
                    Ok(json) => {
                        println!("Parsed JSON Response:\n{:#}", json);
                    }
                    Err(err) => {
                        eprintln!("Failed to parse response JSON: {:?}", err);
                    }
                }
            } else {
                eprintln!("Request failed with status: {}", status);
            }
        }
        Err(err) => {
            eprintln!("Error making request: {:?}", err);
        }
    }
}
