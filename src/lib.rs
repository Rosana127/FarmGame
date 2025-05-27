use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{window, HtmlCanvasElement, CanvasRenderingContext2d, HtmlImageElement, HtmlSelectElement, Element, MouseEvent};
use std::cell::RefCell;
use std::rc::Rc;

mod farm;
mod tile;
mod inventory;
mod shop;
use crate::farm::Farm;
use crate::tile::CropType;
use crate::shop::Shop;

thread_local! {
    static FARM: RefCell<Farm> = RefCell::new(Farm::new(10, 10));
    static SHOP: RefCell<Shop> = RefCell::new(Shop::new());
    static WHEAT_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static CORN_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static CARROT_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static SEED_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static SHOP_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
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
    let success = FARM.with(|farm| farm.borrow_mut().plant(row, col, crop_type));
    if !success {
        web_sys::console::log_1(&"种植失败：没有足够的种子或地块不为空".into());
    }
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

#[wasm_bindgen]
pub fn get_inventory() -> JsValue {
    FARM.with(|farm| {
        let inventory = farm.borrow().get_inventory();
        serde_wasm_bindgen::to_value(&inventory).unwrap()
    })
}

#[wasm_bindgen]
pub fn get_balance() -> u32 {
    SHOP.with(|shop| shop.borrow().get_balance())
}

#[wasm_bindgen]
pub fn buy_seed(seed_type: String) -> bool {
    SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        if shop.buy_seed(&seed_type) {
            FARM.with(|farm| {
                farm.borrow_mut().inventory.add_seed(&seed_type);
            });
            true
        } else {
            false
        }
    })
}

#[wasm_bindgen]
pub fn sell_crop(crop_type: String) {
    SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        shop.sell_crop(&crop_type);
    });
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

        // 绘制农田网格
        for row in 0..10 {
            for col in 0..10 {
                let state = get_state(row, col);

                closure_ctx.set_fill_style(&JsValue::from_str("#ddd"));
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

        // 更新背包显示
        if let Some(inventory_el) = document.get_element_by_id("inventory") {
            let (seeds, crops) = FARM.with(|farm| farm.borrow().get_inventory());
            let balance = SHOP.with(|shop| shop.borrow().get_balance());

            let inventory_html = format!(
                r#"
                <div class="balance">金币: {}</div>
                <div class="inventory-section">
                    <h3>种子</h3>
                    <div class="inventory-items">
                        {}
                    </div>
                </div>
                <div class="inventory-section">
                    <h3>农作物</h3>
                    <div class="inventory-items">
                        {}
                    </div>
                </div>
                "#,
                balance,
                seeds.iter().map(|(item, count)| {
                    let img_src = format!("{}.png", item);
                    format!(
                        r#"<div class="inventory-item"><img src="{}" /><div>x{}</div></div>"#,
                        img_src, count
                    )
                }).collect::<Vec<_>>().join(""),
                crops.iter().map(|(item, count)| { //
                    let img_src = format!("{}.png", item); //
                    let sell_price = SHOP.with(|s| s.borrow().get_crop_price(item).unwrap_or(0)); //
                    let sell_fn_call = format!("window.wasmBindings.try_sell_crop('{}')", item); //
                    format!( //
                        r#"<div class="inventory-item">
                            <img src="{}" />
                            <div>x{}</div>
                            <button onclick="{}">Sell 1 ({}金币)</button>
                        </div>"#,
                        img_src, count, sell_fn_call, sell_price
                    )
                }).collect::<Vec<_>>().join("")
            );

            inventory_el.set_inner_html(&inventory_html);
        }

        // 更新商城显示
        if let Some(shop_el) = document.get_element_by_id("shop-items") {
            let balance = SHOP.with(|shop| shop.borrow().get_balance());
            let shop_html = format!(
                r#"
                <div class="balance">金币: {}</div>
                <div class="shop-item">
                    <img src="wheat.png" />
                    <div>小麦种子</div>
                    <button onclick="window.wasmBindings.buy_seed('wheat')">购买 (10金币)</button>
                </div>
                <div class="shop-item">
                    <img src="corn.png" />
                    <div>玉米种子</div>
                    <button onclick="window.wasmBindings.buy_seed('corn')">购买 (20金币)</button>
                </div>
                <div class="shop-item">
                    <img src="carrot.png" />
                    <div>胡萝卜种子</div>
                    <button onclick="window.wasmBindings.buy_seed('carrot')">购买 (15金币)</button>
                </div>
                "#,
                balance
            );
            shop_el.set_inner_html(&shop_html);
        }

        let _ = window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
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
    let _ = window().unwrap().set_timeout_with_callback_and_timeout_and_arguments_0(
        f.borrow().as_ref().unwrap().as_ref().unchecked_ref(),
        1000,
    );

    Ok(())
}

fn load_image(src: &str, setter: fn(HtmlImageElement)) -> Result<(), JsValue> {
    let document = window().unwrap().document().unwrap();
    let img = document.create_element("img")?.dyn_into::<HtmlImageElement>()?;

    let img_clone = img.clone();
    let closure = Closure::wrap(Box::new(move || {
        setter(img_clone.clone());
        LOADED_COUNT.with(|count| {
            let mut count = count.borrow_mut();
            *count += 1;
            if *count == 4 { // 仅加载 4 张图片（移除 bag.png）
                start_render_loop().unwrap();
            }
        });
    }) as Box<dyn FnMut()>);

    img.set_onload(Some(closure.as_ref().unchecked_ref()));
    img.set_src(src);
    closure.forget();

    Ok(())
}
// 在 src/lib.rs 文件中
#[wasm_bindgen]
pub fn try_sell_crop(crop_type: String) -> bool { //
    let mut sold = false;
    FARM.with(|farm_cell| { //
        let mut farm = farm_cell.borrow_mut();
        if farm.inventory.remove_crop(&crop_type) { // 从物品栏消耗
            SHOP.with(|shop_cell| { //
                // shop.sell_crop 已经增加了余额
                shop_cell.borrow_mut().sell_crop(&crop_type); //
            });
            sold = true;
        }
    });
    if !sold {
        web_sys::console::log_1(&format!("Failed to sell {}: Not in inventory.", crop_type).into());
    }
    sold
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let win = window().ok_or_else(|| JsValue::from_str("无法获取 window"))?;
    let document = win.document().ok_or_else(|| JsValue::from_str("无法获取 document"))?;

    let canvas = document.get_element_by_id("canvas")
        .ok_or_else(|| JsValue::from_str("找不到 canvas 元素"))?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    // Canvas 点击事件：处理种植/收获
    {
        let size = 40;
        let canvas = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let col = (event.offset_x() / size as i32) as usize;
            let row = (event.offset_y() / size as i32) as usize;

            // 处理农田网格点击
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

    // 背包图标点击事件
    {
        let bag_icon = document.get_element_by_id("bag-icon")
            .ok_or_else(|| JsValue::from_str("找不到 bag-icon 元素"))?;
        let bag_icon: Element = bag_icon.dyn_into()?;
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            if let Some(panel_el) = window().unwrap().document().unwrap().get_element_by_id("inventory-panel") {
                let panel: web_sys::Element = panel_el.dyn_into().unwrap();
                let html_panel = panel.dyn_into::<web_sys::HtmlElement>().unwrap();
                let class_list = html_panel.class_list();
                let _ = if class_list.contains("show") {
                    class_list.remove_1("show")
                } else {
                    class_list.add_1("show")
                };
            }
        }) as Box<dyn FnMut(_)>);
        bag_icon.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 作物选择下拉框事件
    {
        let select = document.get_element_by_id("crop-select")
            .ok_or_else(|| JsValue::from_str("找不到 crop-select 元素"))?;
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

    // 商城按钮点击事件
    {
        let shop_button = document.get_element_by_id("shop-button")
            .ok_or_else(|| JsValue::from_str("找不到 shop-button 元素"))?;
        let shop_button: Element = shop_button.dyn_into()?;
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            let balance = SHOP.with(|shop| shop.borrow().get_balance());
            let shop_html = format!(
                r#"
                <div class="balance">金币: {}</div>
                <div class="shop-item">
                    <img src="wheat.png" />
                    <div>小麦种子</div>
                    <button onclick="window.wasmBindings.buy_seed('wheat')">购买 (10金币)</button>
                </div>
                <div class="shop-item">
                    <img src="corn.png" />
                    <div>玉米种子</div>
                    <button onclick="window.wasmBindings.buy_seed('corn')">购买 (20金币)</button>
                </div>
                <div class="shop-item">
                    <img src="carrot.png" />
                    <div>胡萝卜种子</div>
                    <button onclick="window.wasmBindings.buy_seed('carrot')">购买 (15金币)</button>
                </div>
                "#,
                balance
            );
            if let Some(shop_items_el) = window().unwrap().document().unwrap().get_element_by_id("shop-items") {
                shop_items_el.set_inner_html(&shop_html);
            }
            if let Some(panel_el) = window().unwrap().document().unwrap().get_element_by_id("shop-panel") {
                let panel: web_sys::Element = panel_el.dyn_into().unwrap();
                let html_panel = panel.dyn_into::<web_sys::HtmlElement>().unwrap();
                let class_list = html_panel.class_list();
                let _ = class_list.add_1("show");
            }
        }) as Box<dyn FnMut(_)>);
        shop_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 点击事件处理
    {
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let document = web_sys::window().unwrap().document().unwrap();
            let click_target = event
                .target()
                .unwrap()
                .dyn_into::<web_sys::Element>()
                .unwrap();
    
            let is_inside_inventory = click_target.closest("#inventory-panel").unwrap().is_some();
            let is_bag_icon = click_target.closest("#bag-icon").unwrap().is_some();
            let is_canvas = click_target.closest("#canvas").unwrap().is_some();
            let is_shop_button = click_target.closest("#shop-button").unwrap().is_some();
            let is_inside_shop = click_target.closest("#shop-panel").unwrap().is_some();
            
            if !is_inside_inventory && !is_bag_icon && !is_canvas && !is_shop_button && !is_inside_shop {
                if let Some(panel_el) = document.get_element_by_id("inventory-panel") {
                    let panel = panel_el.dyn_into::<web_sys::HtmlElement>().unwrap();
                    let _ = panel.class_list().remove_1("show");
                }
                if let Some(panel_el) = document.get_element_by_id("shop-panel") {
                    let panel = panel_el.dyn_into::<web_sys::HtmlElement>().unwrap();
                    let _ = panel.class_list().remove_1("show");
                }
            }
        }) as Box<dyn FnMut(_)>);
    
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
    
        closure.forget();
    }

    {
        let panel_el = document.get_element_by_id("shop-panel")
            .ok_or_else(|| JsValue::from_str("找不到 shop-panel 元素"))?;
        let panel_el: web_sys::Element = panel_el.dyn_into()?;
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            event.stop_propagation();
        }) as Box<dyn FnMut(_)>);
        panel_el.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 加载图片
    load_image("seed.png", |img| {
        SEED_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    load_image("wheat.png", |img| {
        WHEAT_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    load_image("corn.png", |img| {
        CORN_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    load_image("carrot.png", |img| {
        CARROT_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;

    Ok(())
}