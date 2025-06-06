use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    Wheat,
    Corn,
    Carrot,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FertilizerType {
    None,
    Basic,
    Premium,
    Super,
}

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileState {
    Empty,
    Planted {
        crop: CropType,
        timer: u32,
        fertilizer: FertilizerType,
    },
    Mature { crop: CropType },
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
    pub state: TileState,
}

impl CropType {
    pub fn sell_price(&self) -> u32 {
        match self {
            CropType::Wheat => 15,
            CropType::Corn => 25,
            CropType::Carrot => 20,
        }
    }

    pub fn base_growth_time(&self) -> u32 {
        match self {
            CropType::Wheat => 10,
            CropType::Corn => 15,
            CropType::Carrot => 12,
        }
    }

    pub fn growth_time_with_fertilizer(&self, fertilizer: FertilizerType) -> u32 {
        let base_time = self.base_growth_time();
        match fertilizer {
            FertilizerType::None => base_time,
            FertilizerType::Basic => (base_time as f32 * 0.8) as u32,
            FertilizerType::Premium => (base_time as f32 * 0.65) as u32,
            FertilizerType::Super => (base_time as f32 * 0.5) as u32,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            CropType::Wheat => "小麦",
            CropType::Corn => "玉米",
            CropType::Carrot => "胡萝卜",
        }
    }

    // 新增：作物描述信息
    pub fn description(&self) -> &'static str {
        match self {
            CropType::Wheat => "基础农作物，生长快速，用途广泛",
            CropType::Corn => "高价值作物，生长较慢但收益丰厚",
            CropType::Carrot => "营养丰富的根茎类作物，中等生长周期",
        }
    }

    // 新增：作物特性信息
    pub fn characteristics(&self) -> &'static str {
        match self {
            CropType::Wheat => "• 适应性强\n• 收获量稳定\n• 市场需求量大",
            CropType::Corn => "• 营养价值高\n• 单株产量大\n• 储存时间长",
            CropType::Carrot => "• 富含维生素\n• 抗寒性好\n• 土壤要求低",
        }
    }

    // 新增：种植建议
    pub fn planting_tips(&self) -> &'static str {
        match self {
            CropType::Wheat => "建议: 适合初学者种植，可大面积种植获得稳定收入",
            CropType::Corn => "建议: 高价值作物，建议使用肥料缩短生长时间",
            CropType::Carrot => "建议: 平衡型作物，适合搭配其他作物种植",
        }
    }
}

impl FertilizerType {
    pub fn from_string(s: &str) -> Self {
        match s {
            "basic_fertilizer" => FertilizerType::Basic,
            "premium_fertilizer" => FertilizerType::Premium,
            "super_fertilizer" => FertilizerType::Super,
            _ => FertilizerType::None,
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            FertilizerType::None => "无",
            FertilizerType::Basic => "基础肥料",
            FertilizerType::Premium => "高级肥料",
            FertilizerType::Super => "超级肥料",
        }
    }

    pub fn speed_bonus_text(&self) -> &'static str {
        match self {
            FertilizerType::None => "",
            FertilizerType::Basic => "(-20%时间)",
            FertilizerType::Premium => "(-35%时间)",
            FertilizerType::Super => "(-50%时间)",
        }
    }

    // 新增：肥料效果描述
    pub fn effect_description(&self) -> &'static str {
        match self {
            FertilizerType::None => "",
            FertilizerType::Basic => "提供基础营养，轻微加速生长",
            FertilizerType::Premium => "富含多种营养元素，显著促进生长",
            FertilizerType::Super => "顶级营养配方，极大缩短生长周期",
        }
    }
}

impl Tile {
    pub fn new() -> Self {
        Tile {
            state: TileState::Empty,
        }
    }

    pub fn can_plant(&self) -> bool {
        matches!(self.state, TileState::Empty)
    }

    pub fn can_harvest(&self) -> bool {
        matches!(self.state, TileState::Mature { .. })
    }

    pub fn can_fertilize(&self) -> bool {
        matches!(self.state, TileState::Planted { fertilizer: FertilizerType::None, .. })
    }

    pub fn get_crop_info(&self) -> String {
        match self.state {
            TileState::Empty => {
                "🌱 空地\n━━━━━━━━━━━━━━\n状态: 可以种植作物\n操作: 拖拽种子到此处进行种植\n\n💡 小贴士:\n• 不同作物有不同的生长时间和收益\n• 使用肥料可以加速作物生长\n• 右键点击可以对已种植的作物施肥".to_string()
            },
            TileState::Planted { crop, timer, fertilizer } => {
                let total_time = crop.growth_time_with_fertilizer(fertilizer);
                let remaining = total_time.saturating_sub(timer);
                let progress_percent = ((timer as f32 / total_time as f32) * 100.0) as u32;
                
                // 生成进度条
                let progress_bar_length = 20;
                let filled_length = ((timer as f32 / total_time as f32) * progress_bar_length as f32) as usize;
                let progress_bar = "█".repeat(filled_length) + "░".repeat(progress_bar_length - filled_length).as_str();
                
                let mut info = format!(
                    "🌱 {} (生长中)\n━━━━━━━━━━━━━━\n📊 生长进度: {}% [{}]\n⏰ 剩余时间: {} 秒\n⏱️ 总生长时间: {} 秒",
                    crop.display_name(), progress_percent, progress_bar, remaining, total_time
                );

                // 添加肥料信息
                if fertilizer != FertilizerType::None {
                    info.push_str(&format!(
                        "\n🧪 肥料效果: {} {}\n💬 {}", 
                        fertilizer.display_name(), 
                        fertilizer.speed_bonus_text(),
                        fertilizer.effect_description()
                    ));
                } else {
                    info.push_str("\n🧪 肥料状态: 未施肥 (右键点击可施肥加速生长)");
                }

                // 添加作物详细信息
                info.push_str(&format!(
                    "\n\n📋 作物信息:\n📝 {}\n💰 预期收益: {} 金币\n\n🌟 作物特性:\n{}\n\n💡 {}",
                    crop.description(),
                    crop.sell_price(),
                    crop.characteristics(),
                    crop.planting_tips()
                ));

                info
            },
            TileState::Mature { crop } => {
                format!(
                    "✨ {} (成熟)\n━━━━━━━━━━━━━━\n🎉 状态: 可以收获！\n💰 收获价值: {} 金币\n👆 操作: 点击收获\n\n📋 作物信息:\n📝 {}\n\n🌟 作物特性:\n{}\n\n🏆 恭喜！这株作物已经完全成熟，可以获得丰厚的收益了！",
                    crop.display_name(),
                    crop.sell_price(),
                    crop.description(),
                    crop.characteristics()
                )
            },
        }
    }

    pub fn apply_fertilizer(&mut self, fertilizer: FertilizerType) -> bool {
        if let TileState::Planted { crop, timer, fertilizer: FertilizerType::None } = self.state {
            self.state = TileState::Planted {
                crop,
                timer,
                fertilizer,
            };
            return true;
        }
        false
    }
}