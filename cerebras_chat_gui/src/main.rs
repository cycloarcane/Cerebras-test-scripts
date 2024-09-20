use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, Button, Entry, ScrolledWindow, TextView};
use serde::{Deserialize, Serialize};
use std::env;
use std::sync::{Arc, Mutex};
use log::debug;

#[derive(Serialize, Deserialize, Debug, Clone)]
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

    // Create a new application
    let app = Application::builder()
        .application_id("com.example.cerebras_chat_gui")
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
        .default_width(500)
        .default_height(600)
        .build();

    // Vertical box to hold all widgets
    let vbox = gtk4::Box::new(gtk4::Orientation::Vertical, 5);

    // TextView for conversation history
    let text_view = TextView::builder()
        .editable(false)
        .wrap_mode(gtk4::WrapMode::Word)
        .vexpand(true)
        .build();

    // ScrolledWindow to make TextView scrollable
    let scrolled_window = ScrolledWindow::builder()
        .child(&text_view)
        .vexpand(true)
        .build();

    // Entry for user input
    let entry = Entry::new();

    // Send button
    let send_button = Button::with_label("Send");

    // Horizontal box for entry and button
    let hbox = gtk4::Box::new(gtk4::Orientation::Horizontal, 5);
    hbox.append(&entry);
    hbox.append(&send_button);

    // Add widgets to the vertical box
    vbox.append(&scrolled_window);
    vbox.append(&hbox);

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
            let api_key = env::var("CEREBRAS_API_KEY").expect("CEREBRAS_API_KEY not set");

            // Prepare the request body
            let messages = {
                let history = conversation_history_async.lock().unwrap();
                history.clone()
            };

            let request_body = ChatCompletionRequest {
                model: "llama3.1-8b".to_string(),
                stream: false,
                messages,
                temperature: 0,
                max_tokens: -1,
                seed: 0,
                top_p: 1,
            };

            // Serialize the request body to JSON
            let request_body_json = serde_json::to_string(&request_body).unwrap();
            debug!("Request Body JSON: {}", request_body_json);

            // Build the HTTP client
            let client = reqwest::blocking::Client::builder()
                .http1_only()
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
                    let resp_text = resp.text().unwrap_or_default();

                    if status.is_success() {
                        // Attempt to parse the response JSON
                        match serde_json::from_str::<serde_json::Value>(&resp_text) {
                            Ok(json_value) => {
                                // Extract the assistant's response
                                if let Some(assistant_response) = json_value
                                    .get("choices")
                                    .and_then(|choices| choices.get(0))
                                    .and_then(|choice| choice.get("message"))
                                    .and_then(|message| message.get("content"))
                                    .and_then(|content| content.as_str())
                                {
                                    // Send the assistant's response back to the main thread
                                    sender_async
                                        .send(assistant_response.to_string())
                                        .unwrap();
                                } else {
                                    eprintln!("Failed to extract assistant response");
                                }
                            }
                            Err(err) => {
                                eprintln!("Failed to parse response JSON: {:?}", err);
                                eprintln!("Response Text: {}", resp_text);
                            }
                        }
                    } else {
                        eprintln!("Request failed with status: {}", status);
                        eprintln!("Response Text: {}", resp_text);
                    }
                }
                Err(err) => {
                    eprintln!("Error making request: {:?}", err);
                }
            }
        });
    });

    // Show the window
    window.show();
}
