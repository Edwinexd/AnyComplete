use enigo::Enigo;
use enigo::Key;
use enigo::KeyboardControllable;
use leptess::LepTess;
use win_screenshot::prelude::capture_window;
use win_screenshot::prelude::Area;
use image::{ImageBuffer, Rgba};
use winapi::shared::ntdef::LPCSTR;
use winapi::um::winbase::{GMEM_MOVEABLE, GlobalLock, GlobalUnlock};
use winapi::um::winuser::{CF_TEXT, CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData};
use std::env;
use std::ffi::CStr;
use std::ffi::CString;
use std::io::Cursor;
use std::ptr;
use chat_gpt_rs::prelude::*;
use active_win_pos_rs::get_active_window;
use dotenv::dotenv;

fn capture_window_screenshot() -> ImageBuffer<Rgba<u8>, Vec<u8>> {

    let hwnd;
    match get_active_window() {
        Ok(active_window) => {
            hwnd = active_window.window_id;
        },
        Err(_e) => {
            return ImageBuffer::new(0, 0);
        }
    }

    let re = regex::Regex::new(r"HWND\((\d+)\)").unwrap();
    let caps = re.captures(&hwnd).unwrap();
    let hwnd = caps.get(1).unwrap().as_str();

    let hwnd_isize = hwnd.parse::<isize>().unwrap();

    let window_image = capture_window(hwnd_isize, Area::ClientOnly).unwrap();

    let img: ImageBuffer<Rgba<u8>, Vec<u8>> =
        ImageBuffer::from_raw(window_image.width, window_image.height, window_image.pixels).unwrap();

    return img;
}

fn perform_ocr(img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
    let mut tiff_buffer = Vec::new();
    img.write_to(
        &mut Cursor::new(&mut tiff_buffer),
        image::ImageOutputFormat::Tiff,
    )
    .unwrap();

    let mut ocr = LepTess::new(None, "eng").unwrap();

    ocr.set_image_from_mem(&tiff_buffer).unwrap();

    ocr.get_utf8_text().unwrap()
}

fn execute_ctrl_a_c() {
    let mut enigo = Enigo::new();

    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('a'));
    enigo.key_up(Key::Control);
    std::thread::sleep(std::time::Duration::from_millis(100));

    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('c'));
    enigo.key_up(Key::Control);
    std::thread::sleep(std::time::Duration::from_millis(100));

    enigo.key_click(Key::RightArrow); // Deselect the text
}

fn read_clipboard() -> Option<String> {
    unsafe {
        // Open the clipboard
        if OpenClipboard(ptr::null_mut()) == 0 {
            return None;
        }

        // Get a handle to the clipboard data
        let handle = GetClipboardData(CF_TEXT);
        if handle.is_null() {
            CloseClipboard();
            return None;
        }

        // Lock the handle to get a pointer to the clipboard data
        let ptr = GlobalLock(handle);
        if ptr.is_null() {
            GlobalUnlock(handle);
            CloseClipboard();
            return None;
        }

        // Convert the clipboard data to a string
        let string = CStr::from_ptr(ptr as LPCSTR)
            .to_string_lossy()
            .into_owned();

        // Unlock the handle and close the clipboard
        GlobalUnlock(handle);
        CloseClipboard();

        Some(string)
    }
}

// Complete the given text and context with openai chatgpt
async fn complete_text(text: &str, context: &str) -> String {
    let token = Token::new(env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set"));
    let api = Api::new(token);
    let request = Request {
        model: Model::Gpt35Turbo,
        messages: vec![Message {
            role: "system".to_string(),
            content: "You are a text completion bot.\nYou expect to receive two messages: an OCR read of the application window and the user's text to be completed.\nIf the user includes \"#\" followed by instructions in their text, You should consider any instructions that follow it.\nYou should only append to the user's existing text to complete it, starting where the user left off.\nYou should not add new information beyond what is in the OCR read or user's provided text.\nYou should not reference yourself in its responses.".to_string(),
        }, Message {
            role: "user".to_string(),
            content: format!("OCR: {}", context),
        }, Message {
            role: "user".to_string(),
            content: format!("Complete: {}", text),
        }
        ],
        temperature: Some(0.4),
        frequency_penalty: Some(1.5),

        ..Default::default()
    };
    let response = api.chat(request).await;
    let result = response.unwrap().choices[0].message.content.clone();
    return result;
}

fn update_clipboard(content: &str) {
    unsafe {
        if OpenClipboard(ptr::null_mut()) == 0 {
            return;
        }
        
        EmptyClipboard();
        
        let cstring = CString::new(content).unwrap();
        let size = (cstring.as_bytes().len() + 1) as usize;
        let handle = winapi::um::winbase::GlobalAlloc(GMEM_MOVEABLE, size);
        let data = winapi::um::winbase::GlobalLock(handle) as *mut u8;
        ptr::copy_nonoverlapping(cstring.as_bytes().as_ptr(), data, size);
        winapi::um::winbase::GlobalUnlock(handle);
        
        SetClipboardData(CF_TEXT, handle);
        
        CloseClipboard();
    }
}

fn paste_clipboard() {
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Control);
}

fn main() {
    // Load .env
    dotenv().ok();

    let mut hk = hotkey::Listener::new();
    hk.register_hotkey(hotkey::modifiers::SHIFT |  hotkey::modifiers::SUPER, 'A' as u32, ||{
        let screenshot = capture_window_screenshot();
        let ocr_result = perform_ocr(&screenshot);

        // TODO Save clipboard contents and restore them after completion
        execute_ctrl_a_c();
        match read_clipboard() {
            Some(s) => {
                // Run complete_text async as sync
                let rt = tokio::runtime::Runtime::new().unwrap();
                let response = rt.block_on(complete_text(&s, &ocr_result));
                println!("Response: {}", response);
                update_clipboard(&response);
                paste_clipboard();
    
            },
            None => println!("Failed to read clipboard"),
        }
    }).unwrap();

    hk.listen();
}
