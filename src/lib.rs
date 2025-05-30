use wasm_bindgen::prelude::*;
use crate::tile::TileState;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
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
    farm_grid: Vec<Vec<(tile::TileState, bool)>>,
    inventory_seeds: std::collections::HashMap<String, u32>,
    inventory_crops: std::collections::HashMap<String, u32>,
    balance: u32,
    pesticide: u32,
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
    static LOCK_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);
    static PEST_IMAGE: RefCell<Option<HtmlImageElement>> = RefCell::new(None);

}

#[wasm_bindgen]
pub fn tick() {
    FARM.with(|farm| farm.borrow_mut().tick());
}


/// 🆕 使用后立即刷新画面
pub fn draw_canvas_once() {
    

    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let canvas = document.get_element_by_id("canvas").unwrap();
    let canvas: HtmlCanvasElement = canvas.dyn_into().unwrap();
    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<CanvasRenderingContext2d>()
        .unwrap();

    let size = 40;

    for row in 0..10 {
        for col in 0..10 {
            let (state, is_unlocked) = FARM.with(|farm| {
                let farm = farm.borrow();
                let tile = &farm.grid[row][col];
                (tile.state.clone(), tile.is_unlocked)
            });
            web_sys::console::log_1(&format!("刷新 tile: ({}, {}) -> {:?}", row, col, state).into());
            // 设置背景色
            let fill_color = match state {
                TileState::Pest { .. } => "#b8860b",     // 深黄色
                TileState::Planted { .. } => "#cce5cc",  // 浅绿色
                TileState::Mature { .. } => "#a0dca0",   // 深绿
                TileState::Empty => "#ddd",              // 默认灰色
            };
            ctx.clear_rect(
                (col * size) as f64,
                (row * size) as f64,
                size as f64,
                size as f64,
            );
            ctx.set_fill_style(&JsValue::from_str(fill_color));
            ctx.fill_rect(
                (col * size) as f64,
                (row * size) as f64,
                (size - 2) as f64,
                (size - 2) as f64,
            );
            

            // 绘制图像
            let image = match state {
                TileState::Planted { .. } => SEED_IMAGE.with(|img| img.borrow().clone()),
                TileState::Mature { crop } => match crop {
                    CropType::Wheat => WHEAT_IMAGE.with(|img| img.borrow().clone()),
                    CropType::Corn => CORN_IMAGE.with(|img| img.borrow().clone()),
                    CropType::Carrot => CARROT_IMAGE.with(|img| img.borrow().clone()),
                },
                TileState::Pest { .. } => SEED_IMAGE.with(|img| img.borrow().clone()),
                _ => None,
            };

            if let Some(img) = image {
                let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    &img,
                    (col * size) as f64,
                    (row * size) as f64,
                    (size - 2) as f64,
                    (size - 2) as f64,
                );
            }

            // 如果是虫害状态，叠加 pest.png
            if matches!(state, TileState::Pest { .. }) {
                if let Some(pest_img) = PEST_IMAGE.with(|img| img.borrow().clone()) {
                    let pest_size = size / 2;
                    let _ = ctx.draw_image_with_html_image_element_and_dw_and_dh(
                        &pest_img,
                        (col * size + pest_size) as f64,
                        (row * size + pest_size) as f64,
                        pest_size as f64,
                        pest_size as f64,
                    );
                }
            }
        }
    }
}


#[wasm_bindgen]
pub fn unlock_tile(row: usize, col: usize) -> bool {
    FARM.with(|farm| farm.borrow_mut().unlock_tile(row, col))
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
        // 种植成功后立即保存
        let _ = save_game();
    }
}

#[wasm_bindgen]
pub fn harvest(row: usize, col: usize) {
    FARM.with(|farm| farm.borrow_mut().harvest(row, col));
    // 收获后立即保存
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
            tile::TileState::Pest { crop, .. } => match crop {
                tile::CropType::Wheat => "pest_wheat".into(),
                tile::CropType::Corn => "pest_corn".into(),
                tile::CropType::Carrot => "pest_carrot".into(),
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
        // 购买成功后立即保存
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
    // 出售后立即保存
    let _ = save_game();
}

#[wasm_bindgen]
pub fn save_game() -> Result<(), JsValue> {
    let game_state = FARM.with(|farm| {
        let farm = farm.borrow();
        let grid = farm.grid.iter().map(|row| {
            row.iter().map(|tile| {
                // 保存地块状态和解锁状态
                (tile.state, tile.is_unlocked)
            }).collect::<Vec<_>>()
        }).collect::<Vec<_>>();
        

        let (seeds, crops,pesticide) = farm.get_inventory();
        let balance = SHOP.with(|shop| shop.borrow().get_balance());
        
        GameState {
            farm_grid: grid,
            inventory_seeds: seeds,
            inventory_crops: crops,
            balance,
            pesticide,
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
            // 恢复农田状态
            for (row_idx, row) in game_state.farm_grid.iter().enumerate() {
                for (col_idx, (state, is_unlocked)) in row.iter().enumerate() {
                    farm.grid[row_idx][col_idx].state = state.clone();
                    farm.grid[row_idx][col_idx].is_unlocked = *is_unlocked;
                }
            }
            
            
            // 恢复背包状态
            farm.inventory.seeds = game_state.inventory_seeds;
            farm.inventory.crops = game_state.inventory_crops;
            farm.inventory.pesticide = game_state.pesticide;
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
    
    // 重置游戏状态
    FARM.with(|farm| {
        let mut farm = farm.borrow_mut();
        farm.grid = vec![
            vec![
                tile::Tile {
                    state: tile::TileState::Empty,
                    is_unlocked: false,
                };
                10
            ];
            10
        ];

        farm.inventory = inventory::Inventory::new();
    });
    
    SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        shop.balance = 100;
    });
    
    Ok(())
}

#[wasm_bindgen]
pub fn buy_pesticide() -> bool {
    let mut success = false;
    SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        if shop.buy_item("pesticide") {
            FARM.with(|farm| {
                farm.borrow_mut().inventory.add_pesticide(1);
            });
            success = true;
        }
    });
    if success {
        let _ = save_game(); // 立即保存状态
    }
    success
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

        // 每60秒自动保存一次
        static mut TICK_COUNT: u32 = 0;
        unsafe {
            TICK_COUNT += 1;
            if TICK_COUNT >= 60 {
                TICK_COUNT = 0;
                let _ = save_game();
            }
        }
        // lib.rs 中的 start_render_loop 函数
// 在绘制地块的循环中添加解锁状态的判断
for row in 0..10 {
    for col in 0..10 {
        let (state, is_unlocked) = FARM.with(|farm| {
            let farm = farm.borrow();
            let tile = &farm.grid[row][col];
            (tile.state.clone(), tile.is_unlocked)
        });
        closure_ctx.clear_rect(
            (col * size) as f64,
            (row * size) as f64,
            size as f64,
            size as f64,
        );
        

        // 未解锁：绘制灰色 + lock 图标
        if !is_unlocked {
            closure_ctx.set_fill_style(&"#bbb".into());
            closure_ctx.fill_rect(
                (col * size) as f64,
                (row * size) as f64,
                (size - 2) as f64,
                (size - 2) as f64,
            );

            if let Some(lock_img) = LOCK_IMAGE.with(|img| img.borrow().clone()) {
                let _ = closure_ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    &lock_img,
                    (col * size) as f64 + 6.0,
                    (row * size) as f64 + 6.0,
                    (size - 12) as f64,
                    (size - 12) as f64,
                );
            }

            continue;
        }

        // 根据 tile 状态设置背景色
        let fill_color = match state {
            TileState::Pest { .. } => "#b8860b",     // 深黄色
            TileState::Planted { .. } => "#cce5cc",  // 浅绿色
            TileState::Mature { .. } => "#a0dca0",   // 深绿
            TileState::Empty => "#ddd",              // 默认灰色
        };
        closure_ctx.set_fill_style(&JsValue::from_str(fill_color));
        closure_ctx.fill_rect(
            (col * size) as f64,
            (row * size) as f64,
            (size - 2) as f64,
            (size - 2) as f64,
        );

        // 绘制作物图像（如果有）
        let image = match state {
            TileState::Planted { .. } => {
                SEED_IMAGE.with(|img| img.borrow().clone())
            }
        
            TileState::Mature { crop } => match crop {
                CropType::Wheat => WHEAT_IMAGE.with(|img| img.borrow().clone()),
                CropType::Corn => CORN_IMAGE.with(|img| img.borrow().clone()),
                CropType::Carrot => CARROT_IMAGE.with(|img| img.borrow().clone()),
            },
        
            TileState::Pest { .. } => {
                // 显示种子图
                SEED_IMAGE.with(|img| img.borrow().clone())
            }
        
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
        // ✅ 如果是虫害状态，叠加 pest.png 图标
        if matches!(state, TileState::Pest { .. }) {
            if let Some(pest_img) = PEST_IMAGE.with(|img| img.borrow().clone()) {
                let pest_size = size / 2; // pest 图标大小是格子的一半
                let _ = closure_ctx.draw_image_with_html_image_element_and_dw_and_dh(
                    &pest_img,
                    (col * size + pest_size) as f64, // 叠加在右下角
                    (row * size + pest_size) as f64,
                    pest_size as f64,
                    pest_size as f64,
        );
            }
        }
    }
}


        // 更新背包显示
if let Some(inventory_el) = document.get_element_by_id("inventory") {
    let inventory_el = inventory_el.dyn_into::<HtmlElement>().unwrap();
    if inventory_el.class_list().contains("active") {
        let (seeds, crops, pesticide) = FARM.with(|farm| {
            let inventory = &farm.borrow().inventory;
            (
                inventory.seeds.clone(),
                inventory.crops.clone(),
                inventory.pesticide,
            )
        });

        let balance = SHOP.with(|shop| shop.borrow().get_balance());

        let seed_html = seeds.iter().map(|(item, count)| {
            let img_src = format!("{}.png", item);
            format!(
                r#"<div class="inventory-item" draggable="true" data-seed-type="{}">
                    <img src="{}" />
                    <div>x{}</div>
                </div>"#,
                item, img_src, count
            )
        }).collect::<Vec<_>>().join("");

        let crop_html = crops.iter().map(|(item, count)| {
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
        }).collect::<Vec<_>>().join("");

        let tool_html = if pesticide > 0 {
            format!(
                r#"<div class="inventory-section">
                    <h3>工具</h3>
                    <div class="inventory-items">
                        <div class="inventory-item" draggable="true" data-seed-type="pesticide">
                            <img src="bottle.png" />
                            <div>x{}</div>
                        </div>
                    </div>
                </div>"#,
                pesticide
            )
        } else {
            "".to_string()
        };

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
            {}
            "#,
            balance,
            seed_html,
            crop_html,
            tool_html
        );

        inventory_el.set_inner_html(&inventory_html);

        // 添加拖拽事件监听器
        let seed_items = inventory_el.get_elements_by_class_name("inventory-item");
        for i in 0..seed_items.length() {
            if let Some(item) = seed_items.get_with_index(i) {
                let item = item.dyn_into::<HtmlElement>().unwrap();
                let seed_type = item.get_attribute("data-seed-type").unwrap_or_default();
            
                // 克隆 item 给 drag_start 闭包使用
                let item_for_start = item.clone();
                let drag_start = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
                    if let Some(data_transfer) = event.data_transfer() {
                        let _ = data_transfer.set_data("text/plain", &seed_type);
                    }
                    let _ = item_for_start.class_list().add_1("dragging");
                }) as Box<dyn FnMut(_)>);
                let _ = item.add_event_listener_with_callback("dragstart", drag_start.as_ref().unchecked_ref());
                drag_start.forget();
            
                // 再克隆 item 给 drag_end 闭包使用
                let item_for_end = item.clone();
                let drag_end = Closure::wrap(Box::new(move |_event: web_sys::DragEvent| {
                    let _ = item_for_end.class_list().remove_1("dragging");
                }) as Box<dyn FnMut(_)>);
                let _ = item.add_event_listener_with_callback("dragend", drag_end.as_ref().unchecked_ref());
                drag_end.forget();
            }
            
        }
    }
}


        // 更新商城显示
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
                <div class="shop-section">
                    <h3>工具</h3>
                    <div class="shop-items-grid">
                        <div class="shop-item">
                            <img src="bottle.png" />
                            <div>杀虫剂</div>
                            <div class="price">5金币</div>
                            <button onclick="window.wasmBindings.buy_pesticide()">购买</button>
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
        // 出售成功后立即保存
        let _ = save_game();
    } else {
        web_sys::console::log_1(&format!("Failed to sell {}: Not in inventory.", crop_type).into());
    }
    sold
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    // 首先尝试加载存档
    let _ = load_game();

    let win = window().ok_or_else(|| JsValue::from_str("无法获取 window"))?;
    let document = win.document().ok_or_else(|| JsValue::from_str("无法获取 document"))?;

    let canvas = document.get_element_by_id("canvas")
        .ok_or_else(|| JsValue::from_str("找不到 canvas 元素"))?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    // lib.rs 中的 canvas 点击事件处理
    {
        let size = 40;
        let canvas = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let col = (event.offset_x() / size as i32) as usize;
            let row = (event.offset_y() / size as i32) as usize;

        // 检查是否超出范围
            if row >= 10 || col >= 10 {
                return;
            }

        // 获取地块解锁状态
            let is_unlocked = FARM.with(|farm| farm.borrow().grid[row][col].is_unlocked);
            let state = get_state(row, col);

            if !is_unlocked {
            // 未解锁地块：处理解锁逻辑
                let success = unlock_tile(row, col);
                if success {
                    web_sys::console::log_1(&"地块已解锁！".into());
                // 解锁成功后保存游戏状态
                    let _ = save_game();
                } else {
                    web_sys::console::log_1(&"解锁失败：金币不足！".into());
                }
                return;
            }

        // 已解锁地块：原种植/收获逻辑
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
        // 拖动 shovel-tool 设置拖拽类型为 "shovel"
        if let Some(shovel_tool) = document.get_element_by_id("shovel-tool") {
        let shovel_icon: HtmlElement = shovel_tool.dyn_into().unwrap();

            let drag_start = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
                if let Some(data_transfer) = event.data_transfer() {
                    let _ = data_transfer.set_data("text/plain", "shovel");
                }
            }) as Box<dyn FnMut(_)>);

            shovel_icon
                .add_event_listener_with_callback("dragstart", drag_start.as_ref().unchecked_ref())?;
            drag_start.forget();
        }

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
                    
                    // 移除所有标签的active类
                    let tabs = document.get_elements_by_class_name("panel-tab");
                    for j in 0..tabs.length() {
                        if let Some(t) = tabs.get_with_index(j) {
                            let t = t.dyn_into::<HtmlElement>().unwrap();
                            let _ = t.class_list().remove_1("active");
                        }
                    }
                    
                    // 隐藏所有面板内容
                    if let Some(inventory) = document.get_element_by_id("inventory") {
                        let inventory = inventory.dyn_into::<HtmlElement>().unwrap();
                        let _ = inventory.set_attribute("style", "display: none");
                    }
                    if let Some(shop) = document.get_element_by_id("shop-items") {
                        let shop = shop.dyn_into::<HtmlElement>().unwrap();
                        let _ = shop.set_attribute("style", "display: none");
                    }
                    
                    // 添加active类到当前标签
                    let _ = tab_clone.class_list().add_1("active");
                    
                    // 显示对应的面板内容
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
        let document = window().unwrap().document().unwrap();
        if let Some(shovel_el) = document.get_element_by_id("shovel-tool") {
            let shovel = shovel_el.dyn_into::<HtmlElement>().unwrap();
            let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
                let dt = event.data_transfer().unwrap();
                let _ = dt.set_data("text/plain", "shovel");
            }) as Box<dyn FnMut(_)>);
            shovel.add_event_listener_with_callback("dragstart", closure.as_ref().unchecked_ref())?;
            closure.forget();
        }
    }

    {
        let canvas_rc = Rc::new(canvas.clone());
        let canvas_clone = canvas_rc.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            event.prevent_default();
            let canvas = canvas_clone.dyn_ref::<HtmlElement>().unwrap();
            let _ = canvas.class_list().remove_1("drag-over");
    
            let data_transfer = event.data_transfer().unwrap();
            let dragged_type = data_transfer.get_data("text/plain").unwrap_or_default();
    
            let col = (event.offset_x() / 40 as i32) as usize;
            let row = (event.offset_y() / 40 as i32) as usize;
    
            if dragged_type == "shovel" {
                // 使用铲子逻辑：清除该地块作物
                FARM.with(|farm| {
                    let mut farm = farm.borrow_mut();
                    if row < 10 && col < 10 {
                        farm.grid[row][col].state = TileState::Empty;
                    }
                });
                let _ = save_game();
                return;
            }
    
            // 否则是种子逻辑
            let crop_type = match dragged_type.as_str() {
                "wheat" => CropType::Wheat,
                "corn" => CropType::Corn,
                "carrot" => CropType::Carrot,
                _ => CropType::Wheat,
            };
    
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
        {
            let canvas_rc = Rc::new(canvas.clone());
            let canvas_clone = canvas_rc.clone();
            let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
                event.prevent_default();
                let canvas = canvas_clone.dyn_ref::<HtmlElement>().unwrap();
                let _ = canvas.class_list().remove_1("drag-over");
        
                let data_transfer = event.data_transfer().unwrap();
                let item_type = data_transfer.get_data("text/plain").unwrap_or_default();
        
                let col = (event.offset_x() / 40 as i32) as usize;
                let row = (event.offset_y() / 40 as i32) as usize;
        
                if row >= 10 || col >= 10 {
                    return;
                }
        
                // 如果是 shovel，检查是否有作物或种子，确认后清除
                if item_type == "shovel" {
                    let state_str = get_state(row, col);
                    if state_str.starts_with("planted") || state_str.starts_with("mature") || state_str.starts_with("pest") {
                        if window().unwrap().confirm_with_message("是否要铲除该作物？").unwrap_or(false) {
                            FARM.with(|farm| {
                                farm.borrow_mut().clear_tile(row, col);
                            });
                            let _ = save_game();
                        }
                    }
                    return;
                }
                if item_type == "pesticide" {
                    if window()
                        .unwrap()
                        .confirm_with_message("是否使用杀虫剂？")
                        .unwrap_or(false)
                    {
                        // ✅ 提前借用并结束作用域，避免 RefCell 冲突
                        let success = {
                            FARM.with(|farm| farm.borrow_mut().use_pesticide(row, col))
                        };
                
                        if success {
                            // ✅ 此时已释放可变借用，可以安全调用其他函数
                            draw_canvas_once();
                            let _ = save_game();
                        } else {
                            web_sys::console::log_1(
                                &"使用杀虫剂失败（可能没有库存或不是虫害）".into(),
                            );
                        }
                    }
                
                    return;
                }
                
                
                
        
                // 否则是种子拖拽种植逻辑（保持不变）
                let crop_type = match item_type.as_str() {
                    "wheat" => CropType::Wheat,
                    "corn" => CropType::Corn,
                    "carrot" => CropType::Carrot,
                    _ => CropType::Wheat,
                };
        
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
    load_image("lock.png", |img| {
        LOCK_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
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

    load_image("pest.png", |img| {
        PEST_IMAGE.with(|cell| *cell.borrow_mut() = Some(img));
    })?;
    
    
    Ok(())
}