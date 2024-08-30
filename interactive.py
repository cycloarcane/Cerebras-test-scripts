import os
from cerebras.cloud.sdk import Cerebras

def main():
    # Initialize the Cerebras client
    client = Cerebras(
        api_key=os.environ.get("CEREBRAS_API_KEY"),
    )

    # Initialize an empty list to store the conversation history
    conversation_history = []

    print("Welcome to the Interactive Cerebras Chat!")
    print("Type 'exit' to end the conversation.")

    while True:
        # Get user input
        user_input = input("\nYou: ")

        # Check if the user wants to exit
        if user_input.lower() == 'exit':
            print("Thank you for chatting. Goodbye!")
            break

        # Add the user's message to the conversation history
        conversation_history.append({"role": "user", "content": user_input})

        # Create a chat completion
        chat_completion = client.chat.completions.create(
            messages=conversation_history,
            model="llama3.1-8b",
        )

        # Extract the assistant's response
        assistant_response = chat_completion.choices[0].message.content

        # Print the assistant's response
        print(f"\nAssistant: {assistant_response}")

        # Add the assistant's response to the conversation history
        conversation_history.append({"role": "assistant", "content": assistant_response})

if __name__ == "__main__":
    main()