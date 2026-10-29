use anyhow::Result;
use crate::models::LyricLine;
use crate::models::LrcParseResult;

/// 将秒数格式化为 LRC 时间标签 [mm:ss.xx]
pub fn sec_to_lrc_time(sec: f64) -> String {
    let total = sec.floor() as u32;
    let min = total / 60;
    let s = total % 60;
    let ms = ((sec - sec.floor()) * 100.0).floor() as u32;
    format!("[{:02}:{:02}.{:02}]", min, s, ms)
}

/// 把内存歌词数组导出为完整LRC文本
pub fn export_lrc(ti:&str, ar:&str, al:&str, lyrics:&[LyricLine]) -> String {
    let mut out = String::new();

    if !ti.is_empty() { out.push_str(&format!("[ti:{}]\n", ti)); }
    if !ar.is_empty() { out.push_str(&format!("[ar:{}]\n", ar)); }
    if !al.is_empty() { out.push_str(&format!("[al:{}]\n", al)); }

    for line in lyrics {
        let mut time_tags = String::new();
        for &ts in &line.timestamps {
            time_tags.push_str(&sec_to_lrc_time(ts));
        }
        out.push_str(&format!("{}{}\n", time_tags, line.text));
    }
    out
}

/// 将纯文本（无时间戳，USLT原始歌词）切分成LyricLine，timestamps为空数组，供前端后续打时间戳
pub fn plain_text_to_lyric_lines(text: &str) -> Vec<LyricLine> {
    text.lines()
        .map(|s| LyricLine {
            timestamps: Vec::new(),
            text: s.to_string(),
        })
        .collect()
}

/// 解析标准LRC字符串，生成内存LrcParseResult（元信息+歌词行数组）
/// 支持一行多时间标签，例：[00:01.23][00:05.67]这是一句歌词
pub fn parse_lrc(text: &str) -> Result<LrcParseResult> {
    let mut parse_result = LrcParseResult::default();

    for line in text.lines() {
        let trimmed = line.trim();
        // -------- 处理元标签 [ti:xxx] [ar:xxx] [al:xxx] --------
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            let inner = &trimmed[1..trimmed.len()-1];
            if let Some((key, val)) = inner.split_once(':') {
                match key {
                    "ti" => parse_result.ti = val.to_string(),
                    "ar" => parse_result.ar = val.to_string(),
                    "al" => parse_result.al = val.to_string(),
                    _ => {}
                }
                // 元标签行直接跳过，不进入歌词行解析
                continue;
            }
        }

        // -------- 普通歌词行：解析时间戳 --------
        let mut rest = line;
        let mut timestamps = Vec::new();

        // 循环提取全部 [mm:ss.xx] 时间标签
        while rest.starts_with('[') {
            let end_pos = match rest.find(']') {
                Some(p) => p,
                None => break,
            };
            let time_str = &rest[1..end_pos];
            // 解析 mm:ss.xx
            let parts: Vec<&str> = time_str.split(':').collect();
            if parts.len() != 2 {
                rest = &rest[end_pos + 1..];
                continue;
            }
            let min: f64 = parts[0].parse().unwrap_or(0.0);
            let sec_part: Vec<&str> = parts[1].split('.').collect();
            let sec: f64 = sec_part[0].parse().unwrap_or(0.0);
            let frac: f64 = if sec_part.len() >= 2 {
                let s = sec_part[1];
                // 兼容2位/3位毫秒，统一按百分秒处理
                let num = s.parse::<f64>().unwrap_or(0.0);
                num / 100.0
            } else {
                0.0
            };
            let total_sec = min * 60.0 + sec + frac;
            timestamps.push(total_sec);

            rest = &rest[end_pos + 1..];
        }
        let lyric_text = rest.to_string();
        // 时间不为空 或者 文本不为空，才保存该行
        if !timestamps.is_empty() || !lyric_text.is_empty() {
            parse_result.lines.push(LyricLine {
                timestamps,
                text: lyric_text,
            });
        }
    }

    Ok(parse_result)
}
