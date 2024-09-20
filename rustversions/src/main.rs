use gtk4::prelude::*;
use gtk4::prelude::BoxExt;
use gtk4::{Application, ApplicationWindow, Button, Entry, ScrolledWindow, TextView};
use serde::{Deserialize, Serialize};
use std::cell::RefCell;
use std::env;
use std::rc::Rc;

#[derive(Serialize, Deserialize, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize)]
struct ChatCompletionRequest {
    messages: Vec<Message>,
    model: String,
}

#[derive(Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: MessageContent,
}

#[derive(Deserialize)]
struct MessageContent {
    content: String,
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
    let conversation_history = Rc::new(RefCell::new(Vec::new()));

    // Clone variables for closure
    let text_view_clone = text_view.clone();
    let entry_clone = entry.clone();
    let conversation_history_clone = conversation_history.clone();

    send_button.connect_clicked(move |_| {
        let user_input = entry_clone.text().to_string();

        // Clear the entry
        entry_clone.set_text("");

        // Append user's message to conversation history
        conversation_history_clone.borrow_mut().push(Message {
            role: "user".to_string(),
            content: user_input.clone(),
        });

        // Update the text view
        let buffer = text_view_clone.buffer();
        let current_text = buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
        let new_text = format!("{}\nYou: {}", current_text, user_input);
        buffer.set_text(&new_text);

        // Clone for asynchronous block
        let conversation_history_async = conversation_history_clone.clone();
        let text_view_async = text_view_clone.clone();

        // Perform HTTP request in a new thread
        std::thread::spawn(move || {
            // Get the API key from environment variable
            let api_key = env::var("CEREBRAS_API_KEY").unwrap_or_else(|_| "".to_string());

            // Prepare the request body
            let request_body = ChatCompletionRequest {
                messages: conversation_history_async.borrow().clone(),
                model: "llama3.1-8b".to_string(),
            };

            // Create a blocking HTTP client
            let client = reqwest::blocking::Client::new();

            // Send the POST request
            let response = client
                .post("https://api.cerebras.net/chat/completions")
                .bearer_auth(api_key)
                .json(&request_body)
                .send();

            match response {
                Ok(resp) => {
                    if let Ok(resp_json) = resp.json::<ChatCompletionResponse>() {
                        // Extract assistant's response
                        let assistant_response = resp_json.choices[0].message.content.clone();

                        // Append assistant's message to conversation history
                        conversation_history_async.borrow_mut().push(Message {
                            role: "assistant".to_string(),
                            content: assistant_response.clone(),
                        });

                        // Update the TextView in the main thread
                        gtk4::glib::MainContext::default().invoke(move || {
                            let buffer = text_view_async.buffer();
                            let current_text =
                                buffer.text(&buffer.start_iter(), &buffer.end_iter(), false);
                            let new_text = format!(
                                "{}\nAssistant: {}",
                                current_text, assistant_response
                            );
                            buffer.set_text(&new_text);
                        });
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
