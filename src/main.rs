use clap::Parser;
use std::fs;
use std::path::Path;

mod destroyer;
mod math;

#[derive(Parser)]
#[command(name = "css-destroyer")]
#[command(about = "从审美上破坏 CSS 文件", long_about = None)]
struct Cli {
    /// 破坏等级 (1-10 或更高)
    #[arg(long)]
    level: u32,

    /// 随机种子，用于可重现的破坏
    #[arg(long)]
    seed: String,

    /// 输入的 CSS 文件路径
    filename: String,

    /// 解除 Level 5+ 的数值范围限制 (10x-0.1x)
    #[arg(long)]
    yeah_i_know_what_i_am_doing: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // 读取原始文件
    let content = fs::read_to_string(&cli.filename)?;
    
    // 备份原文件
    let path = Path::new(&cli.filename);
    let backup_name = path.with_extension("be4.css");
    fs::copy(&cli.filename, &backup_name)?;
    
    // 执行破坏
    let destroyed = destroyer::destroy_css(&content, cli.level, &cli.seed, cli.yeah_i_know_what_i_am_doing)?;
    
    // 写入破坏后的文件
    fs::write(&cli.filename, destroyed)?;
    
    println!("✓ 原文件已备份至: {}", backup_name.display());
    println!("✓ 破坏完成，level: {}, seed: {}", cli.level, cli.seed);
    
    // level > 5 时显示视力伤害警告
    if cli.level > 5 {
        println!("\n⚠️  警告：Level > 5 已启用完全随机取色模式");
        println!("⚠️  视力伤害警告：请谨慎查看生成的 CSS 效果！");
        
        if !cli.yeah_i_know_what_i_am_doing {
            println!("ℹ️  数值范围已限制在原值的 0.1x-10x 以内");
            println!("ℹ️  使用 --yeah-i-know-what-i-am-doing 解除限制");
        } else {
            println!("💀 已解除数值范围限制 - 准备好迎接混乱吧！");
        }
    }
    
    Ok(())
}
