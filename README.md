# AnyComplete - Complete messages anywhere with GPT
AnyComplete is a tool designed to make typing easier by predicting what the user is going to type next. Uses Tesseract OCR to OCR the current application window, then passes it through GPT3.5-turbo along with text that has been entered to predict the rest of a message. The program is bound to the shortcut win-shift-a and works on any application window where the textbox support ctrl a & ctrl c.

# Usage
AnyComplete is written in Rust and requires Leptess to be set up correctly. Please follow the instructions on the [Leptess](https://github.com/houqp/leptess) page to set up Leptess for use with Rust.

To build AnyComplete, run `cargo build --release` in the project directory.

AnyComplete requires an OpenAI API key to be set in a dotenv file in the project directory. The file should be named `.env` and contain the following line:
```OPENAI_API_KEY=<your api key>```

# Usage
Once AnyComplete is built, simply press win-shift-a while typing in any application window with text input. The program will use OCR to capture the current window and predict what you're going to type next. The result will be pasted next to the text you've already entered.

# Known limitations
- Non-ASCII characters doesn't get pasted correctly
- Only works on Windows
- Model and language are hardcoded.
- The program may crash if the openai api call fails.

# License
AnyComplete is licensed under the MIT license. See [LICENSE](LICENSE.md) for more information.