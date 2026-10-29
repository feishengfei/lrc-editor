use serde::{Serialize, Deserialize};

/// ID3标签信息，前后端IPC传递用
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Id3Info {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub year: String,
    pub track: String,
    pub genre: String,
    /// 封面图片base64，无封面则为空字符串
    pub cover_base64: String,
    /// 原始USLT内嵌歌词（原始文本）
    pub uslt_lyric: String,
}

/// 单行歌词：支持多个时间戳（同一句歌词多行时间点）
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LyricLine {
    /// 时间戳数组，单位：秒，例如 [12.34, 15.67]
    pub timestamps: Vec<f64>,
    pub text: String,
}

/// 音频基础信息
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AudioInfo {
    /// 总时长，单位秒
    pub duration: f64,
    pub id3: Id3Info,
    /// 内存中正在编辑的歌词列表
    pub lyrics: Vec<LyricLine>,
}

/// 后端返回通用结果包装，方便前端统一处理成功/失败
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResultWrap<T> {
    pub ok: bool,
    pub msg: String,
    pub data: Option<T>,
}

impl<T> ResultWrap<T> {
    pub fn success(data: T, msg: &str) -> Self {
        Self {
            ok: true,
            msg: msg.to_string(),
            data: Some(data),
        }
    }

    pub fn fail(msg: &str) -> Self {
        Self {
            ok: false,
            msg: msg.to_string(),
            data: None,
        }
    }
}
