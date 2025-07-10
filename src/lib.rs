use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
pub mod utils;
use crate::tile::FertilizerType;
use crate::utils::play_sound;
use crate::utils::play_background_music;
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
use wasm_bindgen_futures::spawn_local;

// Import modules and types

mod shop;
mod tile;
mod inventory;
mod farm;
use crate::tile::{CropType, TileState,Tile};
use crate::inventory::Inventory;
use crate::shop::Shop;
use crate::farm::Farm;

#[derive(Serialize, Deserialize)]
struct GameState {
    farm_grid: Vec<Vec<TileState>>,
    inventory_seeds: std::collections::HashMap<String, u32>,
    inventory_crops: std::collections::HashMap<String, u32>,
    inventory_fertilizers: std::collections::HashMap<String, u32>,
    balance: u32,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum TaskType {
    PlantCrop { crop: String, count: u32 },
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Task {
    pub id: u32,
    pub description: String,
    pub task_type: TaskType,
    pub progress: u32,
    pub target: u32,
    pub reward: u32,
    pub completed: bool,
    pub claimed: bool,
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
    static SELECTED_FERTILIZER: RefCell<String> = RefCell::new("basic_fertilizer".to_string());
    static LOADED_COUNT: RefCell<u32> = RefCell::new(0);
    static TOOLTIP_UPDATE_TIMER: RefCell<Option<i32>> = RefCell::new(None);
    static CURRENT_HOVERED_POSITION: RefCell<Option<(usize, usize, i32, i32)>> = RefCell::new(None);
    static BUG_PROTECTION_ENABLED: RefCell<bool> = RefCell::new(false);

    static TASKS: RefCell<Vec<Task>> = RefCell::new(vec![
        Task {
            id: 1,
            description: "种植小麦10个".to_string(),
            task_type: TaskType::PlantCrop { crop: "wheat".to_string(), count: 10 },
            progress: 0,
            target: 10,
            reward: 30,
            completed: false,
            claimed: false,
        },
        Task {
            id: 2,
            description: "种植玉米5个".to_string(),
            task_type: TaskType::PlantCrop { crop: "corn".to_string(), count: 5 },
            progress: 0,
            target: 5,
            reward: 20,
            completed: false,
            claimed: false,
        },
        Task {
            id: 3,
            description: "种植胡萝卜3个".to_string(),
            task_type: TaskType::PlantCrop { crop: "carrot".to_string(), count: 3 },
            progress: 0,
            target: 3,
            reward: 15,
            completed: false,
            claimed: false,
        },
    ]);
}


#[wasm_bindgen]
pub fn try_play_music() {
    crate::utils::play_background_music();
}


#[wasm_bindgen]
pub fn tick() {
    BUG_PROTECTION_ENABLED.with(|flag| {
        FARM.with(|farm| {
            if *flag.borrow() {
                farm.borrow_mut().tick_without_infestation();
            } else {
                farm.borrow_mut().tick();
            }
        });
    });
}
#[wasm_bindgen]
pub fn apply_bug_protection() {
    BUG_PROTECTION_ENABLED.with(|flag| *flag.borrow_mut() = true);

    // 清除现有害虫
    FARM.with(|farm| {
        let mut farm = farm.borrow_mut();
        for row in farm.grid.iter_mut() {
            for tile in row.iter_mut() {
                if let TileState::Infested { crop } = tile.state {
                    tile.state = TileState::Planted {
                        crop,
                        timer: 0,
                        fertilizer: FertilizerType::None,
                    };
                }
            }
        }
    });

    play_sound("click.wav");
    crate::utils::show_message("🕸️ 捕虫网部署完成！");
    let _ = save_game();
}


#[wasm_bindgen]
pub fn get_crop_info(row: usize, col: usize) -> String {
    FARM.with(|farm| farm.borrow().get_crop_info(row, col))
}

#[wasm_bindgen]
pub fn spray_tile(row: usize, col: usize) {
    FARM.with(|farm| {
        let mut farm = farm.borrow_mut();
        if row < farm.grid.len() && col < farm.grid[0].len() {
            if let TileState::Infested { crop, .. } = farm.grid[row][col].state {
                // 只有遭到虫害时才清除害虫
                farm.grid[row][col].state = TileState::Planted {
                    crop,
                    timer: 0,
                    fertilizer: FertilizerType::None,
                };
                crate::utils::play_sound("click.wav");
                crate::utils::show_message("🐛 害虫已清除！");
            } else {
                // 没有害虫的情况
                crate::utils::show_message("🚫 这里没有害虫需要清除");
            }
        }
    });
    
    let _ = save_game();
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
    if success {
        play_sound("plant_seed.mp3");
        TASKS.with(|tasks| {
            let mut tasks = tasks.borrow_mut();
            for task in tasks.iter_mut() {
                let TaskType::PlantCrop { crop: ref task_crop, count: _ } = task.task_type;
                if !task.completed && crop == *task_crop {
                    task.progress += 1;
                    if task.progress >= task.target {
                        task.completed = true;
                    }
                }
            }
        });
        let _ = save_game();
    } else {
        web_sys::console::log_1(&"种植失败：没有足够的种子或地块不为空".into());
    }
}

#[wasm_bindgen]
pub fn harvest(row: usize, col: usize) {
    FARM.with(|farm| farm.borrow_mut().harvest(row, col));
    play_sound("sell_crop.wav"); 
    let _ = save_game();
}

#[wasm_bindgen]
pub fn get_state(row: usize, col: usize) -> String {
    FARM.with(|farm| {
        let tile = &farm.borrow().grid[row][col];
        match tile.state {
            TileState::Empty => "empty".into(),
            TileState::Planted { crop, .. } => match crop {
                CropType::Wheat => "planted_wheat".into(),
                CropType::Corn => "planted_corn".into(),
                CropType::Carrot => "planted_carrot".into(),
            },
            TileState::Mature { crop } => match crop {
                CropType::Wheat => "mature_wheat".into(),
                CropType::Corn => "mature_corn".into(),
                CropType::Carrot => "mature_carrot".into(),
            },
            TileState::Infested { crop } => match crop {
                CropType::Wheat => "infested_wheat".into(),
                CropType::Corn => "infested_corn".into(),
                CropType::Carrot => "infested_carrot".into(),
            },
        }
    })
}


#[wasm_bindgen]
pub fn fertilize(row: usize, col: usize) -> bool {
    let fertilizer_type = SELECTED_FERTILIZER.with(|f| f.borrow().clone());
    let result = FARM.with(|farm| {
        farm.borrow_mut().fertilize(row, col, &fertilizer_type)
    });
    if result {
        let _ = save_game();
    }
    result
}

#[wasm_bindgen]
pub fn select_fertilizer(fertilizer_type: String) {
    SELECTED_FERTILIZER.with(|f| *f.borrow_mut() = fertilizer_type);
}

#[wasm_bindgen]
pub fn buy_fertilizer(fertilizer_type: String) -> bool {
    let result = SHOP.with(|shop| {
        let mut shop = shop.borrow_mut();
        if shop.buy_fertilizer(&fertilizer_type) {
            FARM.with(|farm| {
                farm.borrow_mut().inventory.add_fertilizer(&fertilizer_type);
            });
            true
        } else {
            false
        }
    });
    if result {
        play_sound("sell_crop.wav"); 
        let _ = save_game();
    }else{
        play_sound("buy_fail.wav"); 
    }
    result
}

#[wasm_bindgen]
pub fn get_full_inventory() -> JsValue {
    FARM.with(|farm| {
        let inventory = farm.borrow().get_full_inventory();
        serde_wasm_bindgen::to_value(&inventory).unwrap()
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
        play_sound("sell_crop.wav"); 
        let _ = save_game();
    }else{
        play_sound("buy_fail.wav");
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
        
        let (seeds, crops, fertilizers) = farm.get_full_inventory();
        let balance = SHOP.with(|shop| shop.borrow().get_balance());
        
        GameState {
            farm_grid: grid,
            inventory_seeds: seeds,
            inventory_crops: crops,
            inventory_fertilizers: fertilizers,
            balance,
        }
    });

    let storage = window().unwrap().local_storage()?.unwrap();
    let json = serde_json::to_string(&game_state).map_err(|e| JsValue::from_str(&e.to_string()))?;
    storage.set_item("farm_game_state", &json)?;
    Ok(())
}

#[wasm_bindgen]
pub fn clear_tile(row: usize, col: usize) {
    FARM.with(|farm| {
        let mut farm = farm.borrow_mut();
        if row < farm.grid.len() && col < farm.grid[0].len() {
            if !farm.grid[row][col].can_plant() {
                farm.grid[row][col].state = TileState::Empty;
                crate::utils::show_message("🌿 作物已被清除！");
                crate::utils::play_sound("audio/plant_seed.wav"); // 有这个音效才加
            }
        }
    });
    let _ = save_game();
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
            farm.inventory.fertilizers = game_state.inventory_fertilizers;
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
        farm.grid = vec![vec![Tile { state: TileState::Empty }; 10]; 10];
        farm.inventory = Inventory::new();
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
        
                // ✅ 判断虫害状态，设置背景色
                let is_infested = state.starts_with("infested_");
                let bg_color = if is_infested { "#444" } else { "#ddd" };
        
                closure_ctx.set_fill_style(&JsValue::from_str(bg_color));
                closure_ctx.fill_rect(
                    (col * size) as f64,
                    (row * size) as f64,
                    (size - 2) as f64,
                    (size - 2) as f64,
                );
        
                // ✅ 为虫害或正常状态统一提取图像名
                let image = match state.as_str() {
                    "planted_wheat" | "infested_wheat" => SEED_IMAGE.with(|img| img.borrow().clone()),
                    "planted_corn"  | "infested_corn"  => SEED_IMAGE.with(|img| img.borrow().clone()),
                    "planted_carrot"| "infested_carrot"=> SEED_IMAGE.with(|img| img.borrow().clone()),
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
                let (seeds, crops, fertilizers) = FARM.with(|farm| farm.borrow().get_full_inventory());
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
                    <div class="inventory-section">
                        <h3>肥料</h3>
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
                    }).collect::<Vec<_>>().join(""),
                    fertilizers.iter().map(|(item, count)| {
                        let display_name = match item.as_str() {
                            "basic_fertilizer" => "基础肥料",
                            "premium_fertilizer" => "高级肥料",
                            "super_fertilizer" => "超级肥料",
                            _ => item,
                        };
                        let select_fn_call = format!("window.wasmBindings.select_fertilizer('{}')", item);
                        format!(
                            r#"<div class="inventory-item">
                                <img src="fertilizer.png" />
                                <div>{}</div>
                                <div>x{}</div>
                                <button onclick="{}">选择</button>
                            </div>"#,
                            display_name, count, select_fn_call
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
                            <button onclick="window.wasmBindings.buy_seed('premium_wheat')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="corn.png" />
                            <div>优质玉米种子</div>
                            <div class="price">35金币</div>
                            <button onclick="window.wasmBindings.buy_seed('premium_corn')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="carrot.png" />
                            <div>优质胡萝卜种子</div>
                            <div class="price">30金币</div>
                            <button onclick="window.wasmBindings.buy_seed('premium_carrot')">购买</button>
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
                            <button onclick="window.wasmBindings.buy_seed('golden_wheat')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="corn.png" />
                            <div>金色玉米种子</div>
                            <div class="price">60金币</div>
                            <button onclick="window.wasmBindings.buy_seed('golden_corn')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="carrot.png" />
                            <div>金色胡萝卜种子</div>
                            <div class="price">55金币</div>
                            <button onclick="window.wasmBindings.buy_seed('golden_carrot')">购买</button>
                        </div>
                    </div>
                </div>
                <div class="shop-section">
                    <h3>肥料</h3>
                    <div class="shop-items-grid">
                        <div class="shop-item">
                            <img src="fertilizer.png" />
                            <div>基础肥料</div>
                            <div class="price">25金币</div>
                            <div class="description">减少20%成长时间</div>
                            <button onclick="window.wasmBindings.buy_fertilizer('basic_fertilizer')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="fertilizer.png" />
                            <div>高级肥料</div>
                            <div class="price">50金币</div>
                            <div class="description">减少35%成长时间</div>
                            <button onclick="window.wasmBindings.buy_fertilizer('premium_fertilizer')">购买</button>
                        </div>
                        <div class="shop-item">
                            <img src="fertilizer.png" />
                            <div>超级肥料</div>
                            <div class="price">80金币</div>
                            <div class="description">减少50%成长时间</div>
                            <button onclick="window.wasmBindings.buy_fertilizer('super_fertilizer')">购买</button>
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
        play_sound("sell_crop.wav"); 
    } else {
        web_sys::console::log_1(&format!("Failed to sell {}: Not in inventory.", crop_type).into());
    }
    sold
}

#[wasm_bindgen]
pub fn get_tasks() -> JsValue {
    TASKS.with(|tasks| {
        serde_wasm_bindgen::to_value(&*tasks.borrow()).unwrap()
    })
}

#[wasm_bindgen]
pub fn claim_task_reward(task_id: u32) -> bool {
    let mut claimed = false;
    TASKS.with(|tasks| {
        let mut tasks = tasks.borrow_mut();
        if let Some(task) = tasks.iter_mut().find(|t| t.id == task_id) {
            if task.completed && !task.claimed {
                SHOP.with(|shop| shop.borrow_mut().balance += task.reward);
                task.claimed = true;
                claimed = true;
            }
        }
    });
    if claimed {
        let _ = save_game();
        play_sound("sell_crop.wav");
    }
    claimed
}

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    let _ = load_game();

    play_background_music();

    let win = window().ok_or_else(|| JsValue::from_str("无法获取 window"))?;
    let document = win.document().ok_or_else(|| JsValue::from_str("无法获取 document"))?;

    let canvas = document.get_element_by_id("canvas")
        .ok_or_else(|| JsValue::from_str("找不到 canvas 元素"))?;
    let canvas: HtmlCanvasElement = canvas.dyn_into()?;

    let tooltip = document.create_element("div")?.dyn_into::<HtmlElement>()?;
    tooltip.set_id("crop-tooltip");
    tooltip.set_attribute("style", r#"
        position: fixed;
        background: linear-gradient(145deg, rgba(20, 20, 40, 0.95), rgba(40, 40, 80, 0.95));
        color: white;
        padding: 12px 16px;
        border-radius: 8px;
        font-size: 13px;
        font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
        pointer-events: none;
        z-index: 1000;
        display: none;
        box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
        border: 1px solid rgba(255, 255, 255, 0.1);
        max-width: 280px;
        line-height: 1.6;
        white-space: pre-line !important;
        word-wrap: break-word;
        overflow-wrap: break-word;
    "#)?;

    document.body().unwrap().append_child(&tooltip)?;
    
    // 修改 update_tooltip_content 函数
    fn update_tooltip_content(tooltip: &HtmlElement, row: usize, col: usize, x: i32, y: i32) {
        if row >= 10 || col >= 10 {
            let _ = tooltip.set_attribute("style", r#"
                position: fixed;
                background: linear-gradient(145deg, rgba(20, 20, 40, 0.95), rgba(40, 40, 80, 0.95));
                color: white;
                padding: 12px 16px;
                border-radius: 8px;
                font-size: 13px;
                font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                pointer-events: none;
                z-index: 1000;
                display: none;
                box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
                border: 1px solid rgba(255, 255, 255, 0.1);
                max-width: 320px;
                line-height: 1.6;
                white-space: pre-line;
                word-wrap: break-word;
                overflow-wrap: break-word;
            "#);
            return;
        }

        let crop_info = get_crop_info(row, col);
        let tooltip_text = if crop_info.is_empty() {
            format!("位置: ({}, {})\n状态: 空地\n点击种植作物", row, col)
        } else {
            crop_info  // 直接使用 crop_info，不再添加位置信息，因为 get_crop_info 已经包含了完整信息
        };

        // 使用 textContent 设置文本内容
        tooltip.set_text_content(Some(&tooltip_text));
        
        // 确保样式中包含正确的 white-space 属性
        let _ = tooltip.set_attribute("style", &format!(r#"
            position: fixed;
            background: linear-gradient(145deg, rgba(20, 20, 40, 0.95), rgba(40, 40, 80, 0.95));
            color: white;
            padding: 16px 20px;
            border-radius: 12px;
            font-size: 13px;
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            font-weight: 400;
            pointer-events: none;
            z-index: 1000;
            display: block;
            box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
            border: 1px solid rgba(255, 255, 255, 0.15);
            backdrop-filter: blur(10px);
            max-width: 350px;
            line-height: 1.7;
            white-space: pre-line;
            word-wrap: break-word;
            overflow-wrap: break-word;
            left: {}px;
            top: {}px;
            transform: translateY(-10px);
            animation: tooltipFadeIn 0.2s ease-out;
        "#, x + 15, y - 10));
    }

    // 修改mousemove事件处理器
    {
        let size = 40;
        let canvas = canvas.clone();
        let tooltip = tooltip.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let col = (event.offset_x() / size as i32) as usize;
            let row = (event.offset_y() / size as i32) as usize;
            let x = event.client_x() + 10;
            let y = event.client_y() + 10;

            // 保存当前悬停位置
            CURRENT_HOVERED_POSITION.with(|pos| {
                *pos.borrow_mut() = Some((row, col, x, y));
            });

            // 立即更新tooltip
            update_tooltip_content(&tooltip, row, col, x, y);

            // 启动定时器进行实时更新
            start_tooltip_update_timer();
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

        // 添加mouseleave事件停止定时器
    {
        let canvas = canvas.clone();
        let tooltip = tooltip.clone();
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            // 清除悬停位置
            CURRENT_HOVERED_POSITION.with(|pos| {
                *pos.borrow_mut() = None;
            });
            
            // 停止定时器
            stop_tooltip_update_timer();
            
            // 隐藏tooltip
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
        canvas.add_event_listener_with_callback("mouseleave", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
        // 启动tooltip更新定时器的函数
    fn start_tooltip_update_timer() {
        // 先停止现有定时器
        stop_tooltip_update_timer();
        
        let callback = Closure::wrap(Box::new(|| {
            CURRENT_HOVERED_POSITION.with(|pos| {
                if let Some((row, col, x, y)) = *pos.borrow() {
                    if let Some(tooltip_el) = window().unwrap().document().unwrap().get_element_by_id("crop-tooltip") {
                        let tooltip_el = tooltip_el.dyn_into::<HtmlElement>().unwrap();
                        update_tooltip_content(&tooltip_el, row, col, x, y);
                    }
                }
            });
        }) as Box<dyn FnMut()>);
        
        let timer_id = window().unwrap()
            .set_interval_with_callback_and_timeout_and_arguments_0(
                callback.as_ref().unchecked_ref(),
                1000, // 每秒更新一次
            ).unwrap();
        
        TOOLTIP_UPDATE_TIMER.with(|timer| {
            *timer.borrow_mut() = Some(timer_id);
        });
        
        callback.forget();
    }

    // 停止tooltip更新定时器的函数
    fn stop_tooltip_update_timer() {
        TOOLTIP_UPDATE_TIMER.with(|timer| {
            if let Some(timer_id) = timer.borrow_mut().take() {
                window().unwrap().clear_interval_with_handle(timer_id);
            }
        });
    }

    {
        let canvas = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            event.prevent_default();
            let size = 40;
            let col = (event.offset_x() / size as i32) as usize;
            let row = (event.offset_y() / size as i32) as usize;
            
            let result = fertilize(row, col);
            
            if result {
                let _ = save_game();
                web_sys::console::log_1(&format!("成功施肥位置 ({}, {})", row, col).into());
            } else {
                web_sys::console::log_1(&format!("施肥失败位置 ({}, {})", row, col).into());
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("contextmenu", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let bag_icon = document.get_element_by_id("bag-icon")
            .ok_or_else(|| JsValue::from_str("找不到 bag-icon 元素"))?;
        let bag_icon: Element = bag_icon.dyn_into()?;
        let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
            play_sound("click.wav"); 
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

    {
        let tabs = document.get_elements_by_class_name("panel-tab");
        for i in 0..tabs.length() {
            if let Some(tab) = tabs.get_with_index(i) {
                let tab = tab.dyn_into::<HtmlElement>().unwrap();
                let tab_clone = tab.clone();
                let closure = Closure::wrap(Box::new(move |_event: MouseEvent| {
                    let document = window().unwrap().document().unwrap();
                    crate::utils::play_sound("click.wav");
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
            let is_tooltip = click_target.closest("#crop-tooltip").unwrap().is_some() 
                            || click_target.id() == "crop-tooltip";
            
            if !is_inside_panel && !is_bag_icon && !is_canvas && !is_tooltip {
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

    {
        let canvas = canvas.clone();
        let closure = Closure::wrap(Box::new(move |event: MouseEvent| {
            let size = 40;
            let col = (event.offset_x() / size as i32) as usize;
            let row = (event.offset_y() / size as i32) as usize;
            
            let state = get_state(row, col);
            if state.starts_with("mature_") {
                harvest(row, col);
                web_sys::console::log_1(&format!("收获了位置 ({}, {})", row, col).into());
            }
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let canvas_rc = Rc::new(canvas.clone());
        let canvas_clone = canvas_rc.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            event.prevent_default();
            let canvas_html_el = canvas_clone.dyn_ref::<HtmlElement>().unwrap();
            let _ = canvas_html_el.class_list().add_1("drag-over");
        }) as Box<dyn FnMut(_)>);
        canvas.add_event_listener_with_callback("dragover", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    {
        let canvas_rc = Rc::new(canvas.clone());
        let canvas_clone = canvas_rc.clone();
        let closure = Closure::wrap(Box::new(move |event: web_sys::DragEvent| {
            event.prevent_default();
            let canvas_html_el = canvas_clone.dyn_ref::<HtmlElement>().unwrap();
            let _ = canvas_html_el.class_list().remove_1("drag-over");
        
            let data_transfer = event.data_transfer().unwrap();
            let seed_type_string = data_transfer.get_data("text/plain").unwrap();
            
            let col = (event.offset_x() / 40 as i32) as usize;
            let row = (event.offset_y() / 40 as i32) as usize;
        
            // ✅ 新增：拖的是铲子 shovel，就清除作物
            if seed_type_string == "shovel" {
                wasm_bindgen_futures::spawn_local(async move {
                    clear_tile(row, col);
                });
                return;
            }
        
            // 否则是种子，就种植
            plant(row, col, seed_type_string);
        }) as Box<dyn FnMut(_)>);
        
        canvas.add_event_listener_with_callback("drop", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }
    
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