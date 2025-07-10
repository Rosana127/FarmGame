use serde::{Serialize, Deserialize};
use crate::utils::show_message;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CropType {
    Wheat,
    PremiumWheat,
    GoldenWheat,
    Corn,
    PremiumCorn,
    GoldenCorn,
    Carrot,
    PremiumCarrot,
    GoldenCarrot,
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
    Infested { crop: CropType }, // 🐛 新增虫害状态
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Tile {
    pub state: TileState,
}

impl CropType {
    pub fn sell_price(&self) -> u32 {
        match self {
            CropType::Wheat => 15,
            CropType::PremiumWheat => 25,
            CropType::GoldenWheat => 50,
            CropType::Corn => 25,
            CropType::PremiumCorn => 40,
            CropType::GoldenCorn => 70,
            CropType::Carrot => 20,
            CropType::PremiumCarrot => 32,
            CropType::GoldenCarrot => 60,
        }
    }

    pub fn base_growth_time(&self) -> u32 {
        match self {
            CropType::Wheat => 10,
            CropType::PremiumWheat => 14,
            CropType::GoldenWheat => 20,
            CropType::Corn => 15,
            CropType::PremiumCorn => 20,
            CropType::GoldenCorn => 28,
            CropType::Carrot => 12,
            CropType::PremiumCarrot => 16,
            CropType::GoldenCarrot => 24,
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
            CropType::PremiumWheat => "优质小麦",
            CropType::GoldenWheat => "金色小麦",
            CropType::Corn => "玉米",
            CropType::PremiumCorn => "优质玉米",
            CropType::GoldenCorn => "金色玉米",
            CropType::Carrot => "胡萝卜",
            CropType::PremiumCarrot => "优质胡萝卜",
            CropType::GoldenCarrot => "金色胡萝卜",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            CropType::Wheat => "基础农作物，生长快速，用途广泛",
            CropType::PremiumWheat => "优质小麦，产量更高，生长略慢",
            CropType::GoldenWheat => "金色小麦，极高价值，生长周期长",
            CropType::Corn => "高价值作物，生长较慢但收益丰厚",
            CropType::PremiumCorn => "优质玉米，产量更高，生长更久",
            CropType::GoldenCorn => "金色玉米，极高价值，生长周期最长",
            CropType::Carrot => "营养丰富的根茎类作物，中等生长周期",
            CropType::PremiumCarrot => "优质胡萝卜，产量更高，生长略慢",
            CropType::GoldenCarrot => "金色胡萝卜，极高价值，生长周期长",
        }
    }

    pub fn characteristics(&self) -> &'static str {
        match self {
            CropType::Wheat => "• 适应性强\n• 收获量稳定\n• 市场需求量大",
            CropType::PremiumWheat => "• 更高产量\n• 稳定收益\n• 适合大面积种植",
            CropType::GoldenWheat => "• 极高售价\n• 稀有作物\n• 需要耐心等待",
            CropType::Corn => "• 营养价值高\n• 单株产量大\n• 储存时间长",
            CropType::PremiumCorn => "• 更高产量\n• 高营养\n• 适合搭配肥料",
            CropType::GoldenCorn => "• 极高售价\n• 稀有作物\n• 需要耐心等待",
            CropType::Carrot => "• 富含维生素\n• 抗寒性好\n• 土壤要求低",
            CropType::PremiumCarrot => "• 更高产量\n• 健康营养\n• 适合多地块轮作",
            CropType::GoldenCarrot => "• 极高售价\n• 稀有作物\n• 需要耐心等待",
        }
    }

    pub fn planting_tips(&self) -> &'static str {
        match self {
            CropType::Wheat => "建议: 适合初学者种植，可大面积种植获得稳定收入",
            CropType::PremiumWheat => "建议: 适合追求高产的玩家，注意生长周期",
            CropType::GoldenWheat => "建议: 适合后期冲刺高收益，需耐心等待成熟",
            CropType::Corn => "建议: 高价值作物，建议使用肥料缩短生长时间",
            CropType::PremiumCorn => "建议: 适合搭配高级肥料，追求极致产出",
            CropType::GoldenCorn => "建议: 适合后期冲刺高收益，需耐心等待成熟",
            CropType::Carrot => "建议: 平衡型作物，适合搭配其他作物种植",
            CropType::PremiumCarrot => "建议: 适合多样化种植，搭配轮作提升收益",
            CropType::GoldenCarrot => "建议: 适合后期冲刺高收益，需耐心等待成熟",
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

                let progress_bar_length = 20;
                let filled_length = ((timer as f32 / total_time as f32) * progress_bar_length as f32) as usize;
                let progress_bar = "█".repeat(filled_length) + "░".repeat(progress_bar_length - filled_length).as_str();

                let mut info = format!(
                    "🌱 {} (生长中)\n━━━━━━━━━━━━━━\n📊 生长进度: {}% [{}]\n⏰ 剩余时间: {} 秒\n⏱️ 总生长时间: {} 秒",
                    crop.display_name(), progress_percent, progress_bar, remaining, total_time
                );

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
            TileState::Infested { crop } => {
                format!(
                    "🐛 {} (已被虫害感染)\n━━━━━━━━━━━━━━\n⚠️ 状态: 无法生长\n💀 需要喷雾驱虫恢复\n\n📋 作物信息:\n📝 {}\n💡 建议尽快清理虫害后继续生长",
                    crop.display_name(),
                    crop.description()
                )
            }
        }
    }

    pub fn apply_fertilizer(&mut self, fertilizer: FertilizerType) -> bool {
        match self.state {
            TileState::Planted { crop, timer, fertilizer: FertilizerType::None } => {
                self.state = TileState::Planted {
                    crop,
                    timer,
                    fertilizer,
                };
                show_message(&format!(
                    "施肥成功！使用了{}，生长速度加快。",
                    fertilizer.display_name()
                ));
                true
            },
            TileState::Infested { .. } => {
                show_message("无法施肥：作物已被虫害感染！");
                false
            },
            _ => {
                show_message("无法施肥：该地块未种植或已施肥！");
                false
            }
        }
    }
}
