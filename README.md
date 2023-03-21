# AnyComplete - Complete messages anywhere with GPT
AnyComplete is a tool designed to make typing easier by predicting what the user is going to type next. Uses Tesseract OCR to OCR the current application window, then passes it through GPT3.5-turbo along with text that has been entered to predict the rest of a message. The program is bound to the shortcut win-shift-a and works on any application window where the textbox support ctrl a & ctrl c.

# Building
AnyComplete is written in Rust and requires Leptess to be set up correctly. Please follow the instructions on the [Leptess](https://github.com/houqp/leptess) before building AnyComplete.

Building AnyComplete can be done with cargo by running `cargo build --release` in the project directory.

# Running
## Configuring
To use AnyComplete, you need to set your OpenAI API key in a `.env` file located in the same directory as the AnyComplete executable. The `.env` file should contain the following line:
```OPENAI_API_KEY=sk-yourkeyhere```

## Usage
Once AnyComplete is built and running, simply press win-shift-a while typing in any application window with text input. The program will use OCR to capture the current window and predict what you're going to type next. The result will be pasted next to the text you've already entered.

# Known limitations
- Only works on Windows
- Model and language are hardcoded.
- The program may crash if the openai api call fails.

# License
AnyComplete is licensed under the MIT license. See [LICENSE](LICENSE.md) for more information.