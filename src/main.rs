use enigo::Enigo;
use enigo::Key;
use enigo::KeyboardControllable;
use leptess::LepTess;
use reqwest::Url;
use directories::{ProjectDirs};
use win_screenshot::prelude::capture_window;
use win_screenshot::prelude::Area;
use image::{ImageBuffer, Rgba};
use winapi::um::winbase::{GMEM_MOVEABLE, GlobalLock, GlobalUnlock, GlobalAlloc, GlobalSize};
use winapi::um::winuser::CF_UNICODETEXT;
use winapi::um::winuser::{CloseClipboard, EmptyClipboard, GetClipboardData, OpenClipboard, SetClipboardData};
use std::env;
use std::ffi::OsStr;
use std::ffi::OsString;
use std::io::Cursor;
use std::os::windows::prelude::OsStrExt;
use std::os::windows::prelude::OsStringExt;
use std::path::PathBuf;
use std::ptr;
use std::sync::Mutex;
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

struct OCR {
    leptess: LepTess,
}
impl OCR {
    fn new(data_path: &str, language: &str) -> Self {
        let leptess = LepTess::new(Some(data_path), language).unwrap();
        Self { leptess }
    }
    
    fn perform_ocr(&mut self, img: &ImageBuffer<Rgba<u8>, Vec<u8>>) -> String {
        let mut tiff_buffer = Vec::new();
        img.write_to(
            &mut Cursor::new(&mut tiff_buffer),
            image::ImageOutputFormat::Tiff,
        )
        .unwrap();

        self.leptess.set_image_from_mem(&tiff_buffer).unwrap();

        self.leptess.get_utf8_text().unwrap()
    }
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
        let handle = GetClipboardData(CF_UNICODETEXT);
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
        let wide = std::slice::from_raw_parts(ptr as *const u16, (GlobalSize(handle) / 2) as usize);
        let string = OsString::from_wide(wide).into_string().unwrap_or_default();

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
            content: "You are a text completion bot designed to assist users with writing messages. Abide by the following:
            1. Don't erase or edit anything the user has already typed
            2. Don't repeat anything the user has already typed, instead, continue where they left off
            3. Try to only use information available in the OCR
            4. Always do the best of your ability to autocomplete the user's message even if you are missing information.
            ".to_string(),
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

        let wide: Vec<u16> = OsStr::new(content).encode_wide().chain(Some(0)).collect();
        let size = wide.len() * 2;
        let handle = GlobalAlloc(GMEM_MOVEABLE, size);
        let data = GlobalLock(handle) as *mut u16;
        ptr::copy_nonoverlapping(wide.as_ptr(), data, wide.len());
        GlobalUnlock(handle);

        SetClipboardData(CF_UNICODETEXT, handle);

        CloseClipboard();
    }
}

fn paste_clipboard() {
    let mut enigo = Enigo::new();
    enigo.key_down(Key::Control);
    enigo.key_click(Key::Layout('v'));
    enigo.key_up(Key::Control);
}

fn get_tessdata_path() -> PathBuf {
    let mut data_path = PathBuf::from(ProjectDirs::from("com", "Edwinexd", "AnyComplete").unwrap().data_dir().to_str().unwrap());
    data_path.push("tessdata");
    return data_path;
}

async fn setup() {
    // Load .env
    dotenv().ok();
    // Download tesseract model if it doesn't exist
    let data_path = get_tessdata_path();
    let data_file = data_path.join("eng.traineddata");
    if !data_file.exists() {
        println!("Could not find eng tesseract model at {}, downloading...", data_file.to_str().unwrap());
        if !data_path.exists() {
            std::fs::create_dir_all(&data_path).expect("Failed to create tessdata directory");
        }
        let url = Url::parse("https://github.com/tesseract-ocr/tessdata/raw/main/eng.traineddata").unwrap();
        let response = reqwest::get(url).await;
        let mut file = std::fs::File::create(&data_file).unwrap();
        let mut content = std::io::Cursor::new(response.unwrap().bytes().await.unwrap()); 
        std::io::copy(&mut content, &mut file).unwrap();
        println!("Saved tesseract eng model to {}", data_file.to_str().unwrap());
    } else {
        println!("Found existing tesseract eng model at {}", data_file.to_str().unwrap());
    }
}

#[tokio::main]
async fn main() {
    setup().await;
    let ocr = Mutex::new(OCR::new(get_tessdata_path().to_str().unwrap(), &"eng"));
    
    let mut hk = hotkey::Listener::new();
    hk.register_hotkey(hotkey::modifiers::SHIFT |  hotkey::modifiers::SUPER, 'A' as u32, move || {
        let screenshot = capture_window_screenshot();
        let mut ocr = ocr.lock().unwrap();
        let ocr_result= ocr.perform_ocr(&screenshot);
        execute_ctrl_a_c();
        match read_clipboard() {
            Some(s) => {
                // Run complete_text async as sync
                tokio::spawn(async move {
                    let response = complete_text(&s, &ocr_result).await;
                    println!("Response: {}", &response);
                    update_clipboard(&response);
                    paste_clipboard();
                    // Restore clipboard contents
                    update_clipboard(&s);
                });
            },
            None => println!("Failed to read clipboard"),
        }
    }).unwrap();
    println!("Ready!");
    hk.listen();
}
