/// Sigmoid 函数，将 level 映射到 [0, 1] 区间
/// 使用调整后的参数使得 level=1 到 level=10 有良好的响应曲线
pub fn sigmoid(level: u32) -> f64 {
    let x = level as f64;
    // 调整参数：k 控制陡峭度，x0 控制中心点
    let k = 0.8;
    let x0 = 5.5;
    1.0 / (1.0 + (-k * (x - x0)).exp())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sigmoid_range() {
        assert!(sigmoid(1) < 0.5);
        assert!(sigmoid(5) > 0.4 && sigmoid(5) < 0.6);
        assert!(sigmoid(10) > 0.5);
    }
}
