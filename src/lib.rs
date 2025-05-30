use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    window,
    HtmlCanvasElement,
    CanvasRenderingContext2d,
    HtmlImageElement,
    Element,
    MouseEvent,
    HtmlElement,
};
use std::cell::RefCell;
use std::rc::Rc;
use serde::{Serialize, Deserialize};
use serde_json;

mod farm;
mod tile;
mod inventory;
mod shop;
use crate::farm::Farm;
use crate::tile::CropType;
use crate::shop::Shop;

#[derive(Serialize, Deserialize)]
struct GameState {
    farm_grid: Vec<Vec<tile::TileState>>,
    inventory_seeds: std::collections::HashMap<String, u32>,
    inventory_crops: std::collections::HashMap<String, u32>,
    balance: u32,
}

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
    } else {
        let _ = save_game();
    }
}

#[wasm_bindgen]
pub fn harvest(row: usize, col: usize) {
    FARM.with(|farm| farm.borrow_mut().harvest(row, col));
    let _ = save_game();
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
    let result = SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        if shop.buy_seed(&seed_type) {
            FARM.with(|farm| {
                farm.borrow_mut().inventory.add_seed(&seed_type);
            });
            true
        } else {
            false
        }
    });
    if result {
        let _ = save_game();
    }
    result
}

#[wasm_bindgen]
pub fn sell_crop(crop_type: String) {
    SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        shop.sell_crop(&crop_type);
    });
    let _ = save_game();
}

#[wasm_bindgen]
pub fn save_game() -> Result<(), JsValue> {
    let game_state = FARM.with(|farm| {
        let farm = farm.borrow();
        let grid = farm.grid.iter().map(|row| {
            row.iter().map(|tile| tile.state).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        
        let (seeds, crops) = farm.get_inventory();
        let balance = SHOP.with(|shop| shop.borrow().get_balance());
        
        GameState {
            farm_grid: grid,
            inventory_seeds: seeds,
            inventory_crops: crops,
            balance,
        }
    });

    let storage = window().unwrap().local_storage()?.unwrap();
    let json = serde_json::to_string(&game_state).map_err(|e| JsValue::from_str(&e.to_string()))?;
    storage.set_item("farm_game_state", &json)?;
    Ok(())
}

#[wasm_bindgen]
pub fn load_game() -> Result<(), JsValue> {
    let storage = window().unwrap().local_storage()?.unwrap();
    if let Some(json) = storage.get_item("farm_game_state")? {
        let game_state: GameState = serde_json::from_str(&json)
            .map_err(|e| JsValue::from_str(&e.to_string()))?;
        
        FARM.with(|farm| {
            let mut farm = farm.borrow_mut();
            for (row_idx, row) in game_state.farm_grid.iter().enumerate() {
                for (col_idx, &state) in row.iter().enumerate() {
                    farm.grid[row_idx][col_idx].state = state;
                }
            }
            farm.inventory.seeds = game_state.inventory_seeds;
            farm.inventory.crops = game_state.inventory_crops;
        });
        
        SHOP.with(|shop| {
            let mut shop = shop.borrow_mut();
            shop.balance = game_state.balance;
        });
    }
    Ok(())
}

#[wasm_bindgen]
pub fn clear_save() -> Result<(), JsValue> {
    let storage = window().unwrap().local_storage()?.unwrap();
    storage.remove_item("farm_game_state")?;
    
    FARM.with(|farm| {
        let mut farm = farm.borrow_mut();
        farm.grid = vec![vec![tile::Tile { state: tile::TileState::Empty }; 10]; 10];
        farm.inventory = inventory::Inventory::new();
    });
    
    SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        shop.balance = 100;
    });
    
    Ok(())
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

        static mut TICK_COUNT: u32 = 0;
        unsafe {
            TICK_COUNT += 1;
            if TICK_COUNT >= 60 {
                TICK_COUNT = 0;
                let _ = save_game();
            }
        }

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

        if let Some(inventory_el) = document.get_element_by_id("inventory") {
            let inventory_el = inventory_el.dyn_into::<HtmlElement>().unwrap();
            if inventory_el.class_list().contains("active") {
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
                            r#"<div class="inventory-item" draggable="true" data-seed-type="{}">
                                <img src="{}" />
                                <div>x{}</div>
                            </div>"#,
                            item, img_src, count
                        )
                    }).collect::<Vec<_>>().join(""),
                    crops.iter().map(|(item, count)| {
                        let img_src = format!("{}.png", item);
                        let sell_price = SHOP.with(|s| s.borrow().get_crop_price(item).unwrap_or(0));
                        let sell_fn_call = format!("window.wasmBindings.try_sell_crop('{}')", item);
                        format!(
                            r#"<div class="inventory-item">
                                <img src="{}" />
                                <div>x{}</div>
                                <button onclick="{}">出售 ({}金币)</button>
                            </div>"#,
                            img_src, count, sell_fn_call, sell_price
                        )
                    }).collect::<Vec<_>>().join("")
                );

                inventory_el.set_inner_html(&inventory_html);

                let seed_items = inventory_el.get_elements_by_class_name("inventory-item");
                for i in 0..seed_items.length() {
                    if let Some(item) = seed_items.get_with_index(i) {
                        let item = item.dyn_into::<HtmlElement>().unwrap();
                        let seed_type = item.get_attribute("data-seed-type").unwrap_or_default();
                        
                        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
                            let data_transfer = event.data_transfer().unwrap();
                            let _ = data_transfer.set_data("text/plain", &seed_type);
                            let target = event.target().unwrap();
                            let element = target.dyn_into::<HtmlElement>().unwrap();
                            let _ = element.class_list().add_1("dragging");
                        }) as Box<dyn FnMut(_)>);
                        let _ = item.add_event_listener_with_callback("dragstart", closure.as_ref().unchecked_ref());
                        closure.forget();

                        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
                            let target = event.target().unwrap();
                            let element = target.dyn_into::<HtmlElement>().unwrap();
                            let _ = element.class_list().remove_1("dragging");
                        }) as Box<dyn FnMut(_)>);
                        let _ = item.add_event_listener_with_callback("dragend", closure.as_ref().unchecked_ref());
                        closure.forget();
                    }
                }
            }
        }

        if let Some(shop_el) = document.get_element_by_id("shop-items") {
            let balance = SHOP.with(|shop| shop.borrow().get_balance());
            let shop_html = format!(
                r#"
                <div class="balance">金币: {}</div>
                <div class="shop-section">
                    <h3>基础种子</h3>
                    <div class="shop-items-grid">
                        <div class="shop-item">
                            <img src="wheat.png" />
                            <div>小麦种子</div>
                            <div class="price">10金币</div>
                            <button onclick="window.wasmBindings.buy_seed('wheat')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="corn.png" />
                            <div>玉米种子</div>
                            <div class="price">20金币</div>
                            <button onclick="window.wasmBindings.buy_seed('corn')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="carrot.png" />
                            <div>胡萝卜种子</div>
                            <div class="price">15金币</div>
                            <button onclick="window.wasmBindings.buy_seed('carrot')">购买</button>
                        </div>
                    </div>
                </div>
                <div class="shop-section">
                    <h3>高级种子</h3>
                    <div class="shop-items-grid">
                        <div class="shop-item">
                            <img src="wheat.png" />
                            <div>优质小麦种子</div>
                            <div class="price">25金币</div>
                            <button onclick="window.wasmBindings.buy_seed('wheat')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="corn.png" />
                            <div>优质玉米种子</div>
                            <div class="price">35金币</div>
                            <button onclick="window.wasmBindings.buy_seed('corn')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="carrot.png" />
                            <div>优质胡萝卜种子</div>
                            <div class="price">30金币</div>
                            <button onclick="window.wasmBindings.buy_seed('carrot')">购买</button>
                        </div>
                    </div>
                </div>
                <div class="shop-section">
                    <h3>特殊种子</h3>
                    <div class="shop-items-grid">
                        <div class="shop-item">
                            <img src="wheat.png" />
                            <div>金色小麦种子</div>
                            <div class="price">50金币</div>
                            <button onclick="window.wasmBindings.buy_seed('wheat')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="corn.png" />
                            <div>金色玉米种子</div>
                            <div class="price">60金币</div>
                            <button onclick="window.wasmBindings.buy_seed('corn')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="carrot.png" />
                            <div>金色胡萝卜种子</div>
                            <div class="price">55金币</div>
                            <button onclick="window.wasmBindings.buy_seed('carrot')">购买</button>
                        </div>
                    </div>
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
            if *count == 4 {
                start_render_loop().unwrap();
            }
        });
    }) as Box<dyn FnMut()>);

    img.set_onload(Some(closure.as_ref().unchecked_ref()));
    img.set_src(src);
    closure.forget();

    Ok(())
}

#[wasm_bindgen]
pub fn try_sell_crop(crop_type: String) -> bool {
    let mut sold = false;
    FARM.with(|farm_cell| {
        let mut farm = farm_cell.borrow_mut();
        if farm.inventory.remove_crop(&crop_type) {
            SHOP.with(|shop_cell| {
                shop_cell.borrow_mut().sell_crop(&crop_type);
            });
            sold = true;
        }
    });
    if sold {
        let _ = save_game();
    } else {
        web_sys::console::log_1(&format!("Failed to sell {}: Not in inventory.", crop_type).into());
    }
    sold
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let _ = load_game();

    let win = window().ok_or_else(|| JsValue::from_str("无法获取 window"))?;
    let document = win.document().ok_or_else(|| JsValue::from_str("无法获取 document"))?;

    let canvas = document.get_element_by_id("canvas")
        .ok_or_else(|| JsValue::from_str("找不到 canvas 元素"))?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    // 创建 tooltip 元素
    let tooltip = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    tooltip.set_id("crop-tooltip");
    tooltip.set_attribute("style", r#"
        position: absolute;
        background: rgba(0, 0, 0, 0.8);
        color: white;
        padding: 8px;
        border-radius: 4px;
        font-size: 12px;
        pointer-events: none;
        z-index: 1000;
        display: none;
    "#)?;
    document.body().unwrap().append_child(&tooltip)?;

    // Canvas 点击事件：处理种植/收获
    {
        let size = 40;
        let canvas = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
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

    // Canvas 鼠标悬停事件：显示作物信息
    {
        let size = 40;
        let canvas = canvas.clone();
        let tooltip = tooltip.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let col = (event.offset_x() / size as i32) as usize;
            let row = (event.offset_y() / size as i32) as usize;

            let state = get_state(row, col);
            let tooltip_text = match state.as_str() {
                "empty" => "".to_string(),
                "planted_wheat" => "小麦 - 生长中".to_string(),
                "planted_corn" => "玉米 - 生长中".to_string(),
                "planted_carrot" => "胡萝卜 - 生长中".to_string(),
                "mature_wheat" => "小麦 - 可收获".to_string(),
                "mature_corn" => "玉米 - 可收获".to_string(),
                "mature_carrot" => "胡萝卜 - 可收获".to_string(),
                _ => "".to_string(),
            };

            if !tooltip_text.is_empty() {
                tooltip.set_inner_html(&tooltip_text);
                let x = event.client_x() + 10; // 偏移以避免遮挡鼠标
                let y = event.client_y() + 10;
                tooltip.set_attribute("style", &format!(
                    r#"
                    position: absolute;
                    background: rgba(0, 0, 0, 0.8);
                    color: white;
                    padding: 8px;
                    border-radius: 4px;
                    font-size: 12px;
                    pointer-events: none;
                    z-index: 1000;
                    display: block;
                    left: {}px;
                    top: {}px;
                    "#,
                    x, y
                )).unwrap();
            } else {
                tooltip.set_attribute("style", r#"
                    position: absolute;
                    background: rgba(0, 0, 0, 0.8);
                    color: white;
                    padding: 8px;
                    border-radius: 4px;
                    font-size: 12px;
                    pointer-events: none;
                    z-index: 1000;
                    display: none;
                "#).unwrap();
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // Canvas 鼠标离开事件：隐藏 tooltip
    {
        let canvas = canvas.clone();
        let tooltip = tooltip.clone();
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            tooltip.set_attribute("style", r#"
                position: absolute;
                background: rgba(0, 0, 0, 0.8);
                color: white;
                padding: 8px;
                border-radius: 4px;
                font-size: 12px;
                pointer-events: none;
                z-index: 1000;
                display: none;
            "#).unwrap();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mouseout", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // 背包图标点击事件
    {
        let bag_icon = document.get_element_by_id("bag-icon")
            .ok_or_else(|| JsValue::from_str("找不到 bag-icon 元素"))?;
        let bag_icon: Element = bag_icon.dyn_into()?;
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            if let Some(panel_el) = window().unwrap().document().unwrap().get_element_by_id("inventory-panel") {
                let panel = panel_el.dyn_into::<HtmlElement>().unwrap();
                let class_list = panel.class_list();
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

    // 添加面板切换功能
    {
        let tabs = document.get_elements_by_class_name("panel-tab");
        for i in 0..tabs.length() {
            if let Some(tab) = tabs.get_with_index(i) {
                let tab = tab.dyn_into::<HtmlElement>().unwrap();
                let tab_clone = tab.clone();
                let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
                    let document = window().unwrap().document().unwrap();
                    
                    let tabs = document.get_elements_by_class_name("panel-tab");
                    for j in 0..tabs.length() {
                        if let Some(t) = tabs.get_with_index(j) {
                            let t = t.dyn_into::<HtmlElement>().unwrap();
                            let _ = t.class_list().remove_1("active");
                        }
                    }
                    
                    if let Some(inventory) = document.get_element_by_id("inventory") {
                        let inventory = inventory.dyn_into::<HtmlElement>().unwrap();
                        let _ = inventory.set_attribute("style", "display: none");
                    }
                    if let Some(shop) = document.get_element_by_id("shop-items") {
                        let shop = shop.dyn_into::<HtmlElement>().unwrap();
                        let _ = shop.set_attribute("style", "display: none");
                    }
                    
                    let _ = tab_clone.class_list().add_1("active");
                    
                    let tab_name = tab_clone.get_attribute("data-tab").unwrap_or_default();
                    match tab_name.as_str() {
                        "inventory" => {
                            if let Some(inventory) = document.get_element_by_id("inventory") {
                                let inventory = inventory.dyn_into::<HtmlElement>().unwrap();
                                let _ = inventory.set_attribute("style", "display: block");
                            }
                        },
                        "shop" => {
                            if let Some(shop) = document.get_element_by_id("shop-items") {
                                let shop = shop.dyn_into::<HtmlElement>().unwrap();
                                let _ = shop.set_attribute("style", "display: block");
                            }
                        },
                        _ => {}
                    }
                }) as Box<dyn FnMut(_)>);
                tab.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
                closure.forget();
            }
        }
    }

    // 点击事件处理
    {
        let closure = Closure::wrap(Box::new(move |event: web_sys::MouseEvent| {
            let document = web_sys::window().unwrap().document().unwrap();
            let click_target = event
                .target()
                .unwrap()
                .dyn_into::<Element>()
                .unwrap();
    
            let is_inside_panel = click_target.closest("#inventory-panel").unwrap().is_some();
            let is_bag_icon = click_target.closest("#bag-icon").unwrap().is_some();
            let is_canvas = click_target.closest("#canvas").unwrap().is_some();
            
            if !is_inside_panel && !is_bag_icon && !is_canvas {
                if let Some(panel_el) = document.get_element_by_id("inventory-panel") {
                    let panel = panel_el.dyn_into::<HtmlElement>().unwrap();
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

    // 添加拖拽相关事件处理
    {
        let canvas_rc = Rc::new(canvas.clone());
        let canvas_clone = canvas_rc.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            event.prevent_default();
            let canvas = canvas_clone.dyn_ref::<HtmlElement>().unwrap();
            let _ = canvas.class_list().add_1("drag-over");
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("dragover", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let canvas_rc = Rc::new(canvas.clone());
        let canvas_clone = canvas_rc.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            event.prevent_default();
            let canvas = canvas_clone.dyn_ref::<HtmlElement>().unwrap();
            let _ = canvas.class_list().remove_1("drag-over");
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("dragleave", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let canvas_rc = Rc::new(canvas.clone());
        let canvas_clone = canvas_rc.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            event.prevent_default();
            let canvas = canvas_clone.dyn_ref::<HtmlElement>().unwrap();
            let _ = canvas.class_list().remove_1("drag-over");

            let data_transfer = event.data_transfer().unwrap();
            let seed_type = data_transfer.get_data("text/plain").unwrap();
            let crop_type = match seed_type.as_str() {
                "wheat" => CropType::Wheat,
                "corn" => CropType::Corn,
                "carrot" => CropType::Carrot,
                _ => CropType::Wheat,
            };

            let col = (event.offset_x() / 40 as i32) as usize;
            let row = (event.offset_y() / 40 as i32) as usize;

            SELECTED_CROP.with(|selected| {
                *selected.borrow_mut() = crop_type;
            });

            let success = FARM.with(|farm| farm.borrow_mut().plant(row, col, crop_type));
            if success {
                let _ = save_game();
            } else {
                web_sys::console::log_1(&"种植失败：没有足够的种子或地块不为空".into());
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("drop", closure.as_ref().unchecked_ref())?;
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

    // 添加清空存档按钮
    {
        let document = window().unwrap().document().unwrap();
        let clear_button = document.create_element("button")?;
        clear_button.set_attribute("id", "clear-save")?;
        clear_button.set_attribute("style", r#"
            position: absolute;
            bottom: 10px;
            left: 10px;
            padding: 8px 16px;
            background: #dc3545;
            color: white;
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 14px;
            transition: all 0.2s ease;
        "#)?;
        clear_button.set_text_content(Some("清空存档"));
        
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            let _ = clear_save();
            window().unwrap().location().reload().unwrap();
        }) as Box<dyn FnMut(_)>);
        clear_button.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
        
        document.body().unwrap().append_child(&clear_button)?;
    }

    Ok(())
}