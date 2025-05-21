use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d, HtmlImageElement, HtmlSelectElement};
use std::cell::RefCell;
use std::rc::Rc;

mod farm;
mod tile;
use crate::farm::Farm;
use crate::tile::CropType;

thread_local! {
    static FARM: RefCell<Farm> = RefCell::new(Farm::new(10, 10));
    static WHEAT_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static CORN_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static CARROT_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static SEED_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static SELECTED_CROP: RefCell<CropType> = RefCell::new(CropType::Wheat);
    static LOADED_COUNT: RefCell<u32> = RefCell::new(0);
}

#[wasm_bindgen]
pub fn tick() {
    FARM.with(|farm| farm.borrow_mut().tick());
}

#[wasm_bindgen]
pub fn plant(row: usize, col: usize, crop: String) {
    let crop_type = match crop.as_str() {
        "wheat" => CropType::Wheat,
        "corn" => CropType::Corn,
        "carrot" => CropType::Carrot,
        _ => CropType::Wheat,
    };
    SELECTED_CROP.with(|selected| *selected.borrow_mut() = crop_type);
    FARM.with(|farm| farm.borrow_mut().plant(row, col, crop_type));
}

#[wasm_bindgen]
pub fn harvest(row: usize, col: usize) {
    FARM.with(|farm| farm.borrow_mut().harvest(row, col));
}

#[wasm_bindgen]
pub fn get_state(row: usize, col: usize) -> String {
    FARM.with(|farm| {
        let tile = &farm.borrow().grid[row][col];
        match tile.state {
            tile::TileState::Empty => "empty".into(),
            tile::TileState::Planted { crop, .. } => match crop {
                tile::CropType::Wheat => "planted_wheat".into(),
                tile::CropType::Corn => "planted_corn".into(),
                tile::CropType::Carrot => "planted_carrot".into(),
            },
            tile::TileState::Mature { crop } => match crop {
                tile::CropType::Wheat => "mature_wheat".into(),
                tile::CropType::Corn => "mature_corn".into(),
                tile::CropType::Carrot => "mature_carrot".into(),
            },
        }
    })
}

fn start_render_loop() -> Result<(), JsValue> {
    let win = window().unwrap();
    let document = win.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")?
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()?;
    let size: usize = 40;

    let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let f_clone = f.clone();
    let closure_ctx = ctx.clone();

    let closure = Closure::wrap(Box::new(move || {
        tick();

        for row in 0..10 {
            for col in 0..10 {
                let state = get_state(row, col);

                // 改为这样避免弃用警告
                closure_ctx.set_fill_style(&JsValue::from("#ddd"));
                closure_ctx.fill_rect(
                    (col * size) as f64,
                    (row * size) as f64,
                    (size - 2) as f64,
                    (size - 2) as f64,
                );

                let image = match state.as_str() {
                    "planted_wheat" | "planted_corn" | "planted_carrot" => {
                        SEED_IMAGE.with(|img| img.borrow().clone())
                    }
                    "mature_wheat" => WHEAT_IMAGE.with(|img| img.borrow().clone()),
                    "mature_corn" => CORN_IMAGE.with(|img| img.borrow().clone()),
                    "mature_carrot" => CARROT_IMAGE.with(|img| img.borrow().clone()),
                    _ => None,
                };

                if let Some(img) = image {
                    // 调用正确的带宽高参数的方法
                    let _ = closure_ctx.draw_image_with_html_image_element_and_dw_and_dh(
                        &img,
                        (col * size) as f64,
                        (row * size) as f64,
                        (size - 2) as f64,
                        (size - 2) as f64,
                    );
                }
            }
        }

        let _ = window()
            .unwrap()
            .set_timeout_with_callback_and_timeout_and_arguments_0(
                f_clone
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .unchecked_ref(),
                1000,
            );
    }) as Box<dyn FnMut()>);

    f.borrow_mut().replace(closure);
    let _ = window()
        .unwrap()
        .set_timeout_with_callback_and_timeout_and_arguments_0(
            f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
            1000,
        );

    Ok(())
}

fn load_image(src: &str, setter: fn(HtmlImageElement)) -> Result<(), JsValue> {
    let document = window().unwrap().document().unwrap();
    let img = document.create_element("img")?.dyn_into::<HtmlImageElement>()?;

    let img_clone = img.clone(); // 这里clone一份给closure用

    let closure = Closure::wrap(Box::new(move || {
        setter(img_clone.clone());

        LOADED_COUNT.with(|count| {
            let mut count = count.borrow_mut();
            *count += 1;
            if *count == 4 {
                // 所有图片加载完毕后启动渲染循环
                start_render_loop().unwrap();
            }
        });
    }) as Box<dyn FnMut()>);

    img.set_onload(Some(closure.as_ref().unchecked_ref()));
    img.set_src(src);
    closure.forget();

    Ok(())
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let win = window().unwrap();
    let document = win.document().unwrap();

    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    // 点击事件
    {
        let size = 40;
        let canvas = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let col = (event.offset_x() / size as i32) as usize;
            let row = (event.offset_y() / size as i32) as usize;
            let state = get_state(row, col);
            if state.starts_with("empty") {
                SELECTED_CROP.with(|selected| {
                    FARM.with(|farm| farm.borrow_mut().plant(row, col, *selected.borrow()));
                });
            } else if state.starts_with("mature") {
                harvest(row, col);
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 下拉框事件
    {
        let select = document.get_element_by_id("crop-select").unwrap();
        let select: HtmlSelectElement = select.dyn_into()?;
        let closure = Closure::wrap(Box::new(move |event: web_sys::Event| {
            let target = event.target().unwrap();
            let select = target.dyn_into::<HtmlSelectElement>().unwrap();
            let crop = select.value();
            SELECTED_CROP.with(|selected| {
                *selected.borrow_mut() = match crop.as_str() {
                    "wheat" => CropType::Wheat,
                    "corn" => CropType::Corn,
                    "carrot" => CropType::Carrot,
                    _ => CropType::Wheat,
                };
            });
        }) as Box<dyn FnMut(_)>);
        select.add_event_listener_with_callback("change", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 载入图片，渲染循环会在全部图片加载后自动启动
    load_image("wheat.png", |img| {
        WHEAT_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    load_image("corn.png", |img| {
        CORN_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    load_image("carrot.png", |img| {
        CARROT_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    load_image("seed.png", |img| {
        SEED_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    Ok(())
}
