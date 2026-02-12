use rand::{Rng, SeedableRng};
use rand::rngs::StdRng;
use rand_distr::{Distribution, Normal};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use regex::Regex;

use crate::math::sigmoid;

pub fn destroy_css(content: &str, level: u32, seed: &str, unrestricted: bool) -> Result<String, Box<dyn std::error::Error>> {
    // 从 seed 字符串生成数值种子
    let mut hasher = DefaultHasher::new();
    seed.hash(&mut hasher);
    let numeric_seed = hasher.finish();
    
    let mut rng = StdRng::seed_from_u64(numeric_seed);
    
    // 使用 sigmoid 计算破坏强度
    let intensity = sigmoid(level);
    
    // 基于强度创建正态分布参数
    // intensity 越大，标准差越大，破坏越明显
    let color_std = intensity * 50.0;  // 颜色通道最大偏移
    let size_std = intensity * 20.0;   // 尺寸百分比偏移
    
    let mut result = content.to_string();
    
    // 破坏颜色值 - level > 5 时使用完全随机取色
    let use_random = level > 5;
    result = destroy_colors(&result, &mut rng, color_std, use_random)?;
    
    // 破坏尺寸和位置 - level > 5 时完全随机化
    result = destroy_sizes(&result, &mut rng, size_std, use_random, unrestricted)?;
    
    // 破坏布局属性
    result = destroy_layout_properties(&result, &mut rng, intensity)?;
    
    // level > 5 时强制设置 overflow 为 visible
    if use_random {
        result = force_overflow_visible(&result)?;
    }
    
    Ok(result)
}

fn destroy_colors(content: &str, rng: &mut StdRng, std_dev: f64, use_random: bool) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = content.to_string();
    
    // 匹配 hex 颜色 (#RGB 或 #RRGGBB)
    let hex_re = Regex::new(r"#([0-9a-fA-F]{6}|[0-9a-fA-F]{3})\b")?;
    result = hex_re.replace_all(&result, |caps: &regex::Captures| {
        let hex = &caps[1];
        destroy_hex_color(hex, rng, std_dev, use_random)
    }).to_string();
    
    // 匹配 rgb/rgba
    let rgb_re = Regex::new(r"rgba?\((\d+),\s*(\d+),\s*(\d+)(?:,\s*([\d.]+))?\)")?;
    result = rgb_re.replace_all(&result, |caps: &regex::Captures| {
        let r: u8 = caps[1].parse().unwrap_or(0);
        let g: u8 = caps[2].parse().unwrap_or(0);
        let b: u8 = caps[3].parse().unwrap_or(0);
        let a = caps.get(4).map(|m| m.as_str());
        
        let new_r = disturb_channel(r, rng, std_dev, use_random);
        let new_g = disturb_channel(g, rng, std_dev, use_random);
        let new_b = disturb_channel(b, rng, std_dev, use_random);
        
        if let Some(alpha) = a {
            format!("rgba({}, {}, {}, {})", new_r, new_g, new_b, alpha)
        } else {
            format!("rgb({}, {}, {})", new_r, new_g, new_b)
        }
    }).to_string();
    
    Ok(result)
}

fn destroy_hex_color(hex: &str, rng: &mut StdRng, std_dev: f64, use_random: bool) -> String {
    let (r, g, b) = if hex.len() == 3 {
        let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).unwrap_or(0);
        (r, g, b)
    } else {
        let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(0);
        (r, g, b)
    };
    
    let new_r = disturb_channel(r, rng, std_dev, use_random);
    let new_g = disturb_channel(g, rng, std_dev, use_random);
    let new_b = disturb_channel(b, rng, std_dev, use_random);
    
    format!("#{:02x}{:02x}{:02x}", new_r, new_g, new_b)
}

fn disturb_channel(value: u8, rng: &mut StdRng, std_dev: f64, use_random: bool) -> u8 {
    if use_random {
        // 完全随机取色
        rng.gen_range(0..=255)
    } else {
        // 为每个通道随机生成一个均值偏移
        let mean_range = std_dev * 1.5;
        let mean_offset = rng.gen_range(-mean_range..mean_range);
        
        // 创建以随机均值为中心的正态分布
        let normal = Normal::new(mean_offset, std_dev).unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());
        let offset = normal.sample(rng);
        
        let new_value = (value as f64 + offset).clamp(0.0, 255.0);
        new_value as u8
    }
}

fn destroy_sizes(content: &str, rng: &mut StdRng, std_dev: f64, use_random: bool, unrestricted: bool) -> Result<String, Box<dyn std::error::Error>> {
    // 匹配数值单位 (px, em, rem, %, vh, vw 等)
    let size_re = Regex::new(r"(\d+(?:\.\d+)?)(px|em|rem|%|vh|vw|pt)")?;
    let result = size_re.replace_all(content, |caps: &regex::Captures| {
        let value: f64 = caps[1].parse().unwrap_or(0.0);
        let unit = &caps[2];
        
        if use_random {
            let new_value = if unrestricted {
                // 完全随机化尺寸值（无限制）
                match unit.as_ref() {
                    "px" | "pt" => rng.gen_range(0.0..500.0),
                    "em" | "rem" => rng.gen_range(0.0..10.0),
                    "%" => rng.gen_range(0.0..200.0),
                    "vh" | "vw" => rng.gen_range(0.0..150.0),
                    _ => rng.gen_range(0.0..100.0),
                }
            } else {
                // 限制在原值的 0.1x - 10x 范围内
                let min = value * 0.1;
                let max = value * 10.0;
                rng.gen_range(min..=max).max(0.1) // 至少保持 0.1
            };
            format!("{:.2}{}", new_value, unit)
        } else {
            // 正态分布偏移
            let value: f64 = caps[1].parse().unwrap_or(0.0);
            let mean_range = std_dev * 1.5;
            let mean_offset_percent = rng.gen_range(-mean_range..mean_range);
            
            let normal = Normal::new(mean_offset_percent, std_dev).unwrap_or_else(|_| Normal::new(0.0, 1.0).unwrap());
            let offset_percent = normal.sample(rng) / 100.0;
            
            let new_value = value * (1.0 + offset_percent);
            let new_value = new_value.max(0.0);
            
            format!("{:.2}{}", new_value, unit)
        }
    }).to_string();
    
    Ok(result)
}

fn destroy_layout_properties(content: &str, rng: &mut StdRng, intensity: f64) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = content.to_string();
    
    // 计算属性被随机化的概率（基于 intensity）
    let randomize_probability = intensity * 0.6; // 最高 60% 的属性会被随机化
    
    // flex-direction 的可能值
    let flex_directions = ["row", "row-reverse", "column", "column-reverse"];
    let flex_direction_re = Regex::new(r"flex-direction:\s*(row|row-reverse|column|column-reverse)")?;
    result = flex_direction_re.replace_all(&result, |caps: &regex::Captures| {
        if rng.gen::<f64>() < randomize_probability {
            let new_value = flex_directions[rng.gen_range(0..flex_directions.len())];
            format!("flex-direction: {}", new_value)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    // align-items 的可能值
    let align_items = ["flex-start", "flex-end", "center", "baseline", "stretch"];
    let align_items_re = Regex::new(r"align-items:\s*(flex-start|flex-end|center|baseline|stretch)")?;
    result = align_items_re.replace_all(&result, |caps: &regex::Captures| {
        if rng.gen::<f64>() < randomize_probability {
            let new_value = align_items[rng.gen_range(0..align_items.len())];
            format!("align-items: {}", new_value)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    // justify-content 的可能值
    let justify_content = ["flex-start", "flex-end", "center", "space-between", "space-around", "space-evenly"];
    let justify_content_re = Regex::new(r"justify-content:\s*(flex-start|flex-end|center|space-between|space-around|space-evenly)")?;
    result = justify_content_re.replace_all(&result, |caps: &regex::Captures| {
        if rng.gen::<f64>() < randomize_probability {
            let new_value = justify_content[rng.gen_range(0..justify_content.len())];
            format!("justify-content: {}", new_value)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    // text-align 的可能值
    let text_align = ["left", "right", "center", "justify"];
    let text_align_re = Regex::new(r"text-align:\s*(left|right|center|justify)")?;
    result = text_align_re.replace_all(&result, |caps: &regex::Captures| {
        if rng.gen::<f64>() < randomize_probability {
            let new_value = text_align[rng.gen_range(0..text_align.len())];
            format!("text-align: {}", new_value)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    // position 的可能值
    let positions = ["static", "relative", "absolute", "fixed", "sticky"];
    let position_re = Regex::new(r"position:\s*(static|relative|absolute|fixed|sticky)")?;
    result = position_re.replace_all(&result, |caps: &regex::Captures| {
        if rng.gen::<f64>() < randomize_probability {
            let new_value = positions[rng.gen_range(0..positions.len())];
            format!("position: {}", new_value)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    // display 的可能值
    let displays = ["block", "inline", "inline-block", "flex", "grid", "none"];
    let display_re = Regex::new(r"display:\s*(block|inline|inline-block|flex|grid|none)")?;
    result = display_re.replace_all(&result, |caps: &regex::Captures| {
        if rng.gen::<f64>() < randomize_probability {
            let new_value = displays[rng.gen_range(0..displays.len())];
            format!("display: {}", new_value)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    // float 的可能值
    let floats = ["left", "right", "none"];
    let float_re = Regex::new(r"float:\s*(left|right|none)")?;
    result = float_re.replace_all(&result, |caps: &regex::Captures| {
        if rng.gen::<f64>() < randomize_probability {
            let new_value = floats[rng.gen_range(0..floats.len())];
            format!("float: {}", new_value)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    Ok(result)
}

fn force_overflow_visible(content: &str) -> Result<String, Box<dyn std::error::Error>> {
    let mut result = content.to_string();
    
    // 匹配现有的 overflow 属性并替换为 visible
    let overflow_re = Regex::new(r"overflow(-[xy])?:\s*[^;]+")?;
    result = overflow_re.replace_all(&result, |caps: &regex::Captures| {
        if let Some(axis) = caps.get(1) {
            format!("overflow{}: visible", axis.as_str())
        } else {
            "overflow: visible".to_string()
        }
    }).to_string();
    
    // 在每个 CSS 规则块中添加 overflow: visible（如果不存在）
    // 匹配 { ... } 块
    let block_re = Regex::new(r"\{([^}]*)\}")?;
    result = block_re.replace_all(&result, |caps: &regex::Captures| {
        let block_content = &caps[1];
        
        // 检查是否已经有 overflow 属性
        if !block_content.contains("overflow") {
            format!("{{ overflow: visible;{} }}", block_content)
        } else {
            caps[0].to_string()
        }
    }).to_string();
    
    Ok(result)
}
