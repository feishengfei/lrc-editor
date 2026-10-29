use anyhow::Result;
use crate::models::LyricLine;

/// 将秒数格式化为 LRC 时间标签 [mm:ss.xx]
pub fn sec_to_lrc_time(sec: f64) -> String {
    let total = sec.floor() as u32;
    let min = total / 60;
    let s = total % 60;
    let ms = ((sec - sec.floor()) * 100.0).floor() as u32;
    format!("[{:02}:{:02}.{:02}]", min, s, ms)
}

/// 把内存歌词数组导出为完整LRC文本
pub fn export_lrc(lyrics: &[LyricLine]) -> String {
    let mut out = String::new();
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

/// 解析标准LRC字符串，生成内存LyricLine数组
/// 支持一行多时间标签，例：[00:01.23][00:05.67]这是一句歌词
pub fn parse_lrc(lrc_text: &str) -> Result<Vec<LyricLine>> {
    let mut result = Vec::new();

    for line in lrc_text.lines() {
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
        if !timestamps.is_empty() || !lyric_text.is_empty() {
            result.push(LyricLine {
                timestamps,
                text: lyric_text,
            });
        }
    }
    Ok(result)
}
