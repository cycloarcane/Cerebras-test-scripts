use gtk4::prelude::*;
use gtk4::prelude::BoxExt;
use gtk4::{Application, ApplicationWindow, Button, Entry, ScrolledWindow, TextView};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    model: String,
    stream: bool,
    messages: Vec<Message>,
    temperature: f32,
    max_tokens: i32,
    seed: i32,
    top_p: f32,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    results: Vec<ResultEntry>,
}

#[derive(Deserialize)]
struct ResultEntry {
    generations: Vec<Generation>,
}

#[derive(Deserialize)]
struct Generation {
    text: String,
}

fn main() {
    // Create a new application
    let app = Application::builder()
        .application_id("com.example.cerebras_chat")
        .build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run();
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = ApplicationWindow::builder()
        .application(app)
        .title("Cerebras Chat")
        .default_width(400)
        .default_height(600)
        .build();

    // Vertical box to hold all widgets
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 5);

    // TextView for conversation history
    let text_view = TextView::builder()
        .editable(false)
        .wrap_mode(gtk4::WrapMode::Word)
        .build();

    // ScrolledWindow to make TextView scrollable
    let scrolled_window = ScrolledWindow::builder()
        .vexpand(true)
        .child(&text_view)
        .build();

    // Entry for user input
    let entry = Entry::new();

    // Send button
    let send_button = Button::with_label("Send");

    // Add widgets to the vertical box
    vbox.append(&scrolled_window);
    vbox.append(&entry);
    vbox.append(&send_button);

    // Add the box to the window
    window.set_child(Some(&vbox));

    // Conversation history
    let conversation_history = Arc::new(Mutex::new(Vec::new()));

    // Create a channel to communicate between threads
    let (sender, receiver) = glib::MainContext::channel::<String>(glib::PRIORITY_DEFAULT);

    // Set up the receiver to update the GUI
    {
        let text_view_clone = text_view.clone();
        let conversation_history_clone = Arc::clone(&conversation_history);

        receiver.attach(None, move |assistant_response| {
            // Update the conversation history
            {
                let mut history = conversation_history_clone.lock().unwrap();
                history.push(Message {
                    role: "assistant".to_string(),
                    content: assistant_response.clone(),
                });
            }

            // Update the TextView
            let buffer = text_view_clone.buffer();
            let current_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
            let new_text = format!("{}\nAssistant: {}", current_text, assistant_response);
            buffer.set_text(&new_text);

            glib::Continue(true)
        });
    }

    // Clone variables for closure
    let conversation_history_clone = Arc::clone(&conversation_history);
    let text_view_clone = text_view.clone();
    let sender_clone = sender.clone();
    let entry_clone = entry.clone();

    send_button.connect_clicked(move |_| {
        let user_input = entry_clone.text().to_string();

        // Clear the entry
        entry_clone.set_text("");

        // Append user's message to conversation history
        {
            let mut history = conversation_history_clone.lock().unwrap();
            history.push(Message {
                role: "user".to_string(),
                content: user_input.clone(),
            });
        }

        // Update the text view
        let buffer = text_view_clone.buffer();
        let current_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
        let new_text = format!("{}\nYou: {}", current_text, user_input);
        buffer.set_text(&new_text);

        // Clone for asynchronous block
        let conversation_history_async = Arc::clone(&conversation_history_clone);
        let sender_async = sender_clone.clone();

        // Perform HTTP request in a new thread
        std::thread::spawn(move || {
            // Get the API key from environment variable
            let api_key = env::var("CEREBRAS_API_KEY").unwrap_or_else(|_| "".to_string());

            // Prepare the request body
            let messages = {
                let history = conversation_history_async.lock().unwrap();
                history.clone()
            };

            let request_body = ChatCompletionRequest {
                model: "llama3.1-8b".to_string(),
                stream: false,
                messages,
                temperature: 0.0,
                max_tokens: -1,
                seed: 0,
                top_p: 1.0,
            };

            // Create a blocking HTTP client
            let client = reqwest::blocking::Client::new();

            // Send the POST request
            let response = client
                .post("https://api.cerebras.ai/v1/chat/completions")
                .header("Content-Type", "application/json")
                .header("Authorization", format!("Bearer {}", api_key))
                .json(&request_body)
                .send();

            match response {
                Ok(resp) => {
                    // Read the response text
                    let resp_text = match resp.text() {
                        Ok(text) => text,
                        Err(err) => {
                            eprintln!("Failed to read response text: {:?}", err);
                            return;
                        }
                    };
            
                    // Attempt to parse the JSON
                    match serde_json::from_str::<ChatCompletionResponse>(&resp_text) {
                        Ok(resp_json) => {
                            // Extract assistant's response
                            if let Some(result) = resp_json.results.first() {
                                if let Some(generation) = result.generations.first() {
                                    let assistant_response = generation.text.clone();
            
                                    // Send the assistant's response back to the main thread
                                    sender_async.send(assistant_response).unwrap();
                                } else {
                                    eprintln!("No generations found in response");
                                    eprintln!("Response Text: {:?}", resp_text);
                                }
                            } else {
                                eprintln!("No results found in response");
                                eprintln!("Response Text: {:?}", resp_text);
                            }
                        }
                        Err(err) => {
                            eprintln!("Failed to parse response JSON: {:?}", err);
                            eprintln!("Response Text: {:?}", resp_text);
                        }
                    }
                }
                Err(err) => {
                    eprintln!("Error: {:?}", err);
                }
            }
        });
    });

    // Show the window
    window.show();
}