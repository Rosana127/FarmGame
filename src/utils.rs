use wasm_bindgen::JsCast;
use web_sys::window;
use wasm_bindgen::closure::Closure;
use web_sys::HtmlAudioElement;


pub fn show_message(msg: &str) {
    if let Some(win) = window() {
        if let Some(doc) = win.document() {
            if let Some(box_div) = doc.get_element_by_id("message-box") {
                box_div.set_inner_html(msg);
                box_div.set_attribute("style", "
                    position: fixed;
                    bottom: 100px;
                    left: 50%;
                    transform: translateX(-50%);
                    background: rgba(0, 0, 0, 0.7);
                    color: white;
                    padding: 12px 24px;
                    border-radius: 12px;
                    font-size: 1rem;
                    font-weight: 500;
                    display: block;
                    z-index: 999;
                    box-shadow: 0 4px 12px rgba(0,0,0,0.3);
                    transition: opacity 0.3s ease;
                ").ok();

                // 设置3秒后自动隐藏
                let box_clone = box_div.clone();
                let closure = Closure::once_into_js(move || {
                    let _ = box_clone.set_attribute("style", "display: none;");
                });

                let _ = win
                    .set_timeout_with_callback_and_timeout_and_arguments_0(
                        closure.as_ref().unchecked_ref(),
                        3000,
                    );
            }
        }
    }
}



pub fn play_background_music() {
    let audio = HtmlAudioElement::new_with_src("audio/background_music.mp3").unwrap();
    audio.set_loop(true); // 设置为循环播放
    let _ = audio.play(); // 播放
}

pub fn play_sound(file: &str) {
    if let Ok(audio) = HtmlAudioElement::new_with_src(&format!("audio/{}", file)) {
        let _ = audio.play(); // 播放一次，不循环
    }
}
