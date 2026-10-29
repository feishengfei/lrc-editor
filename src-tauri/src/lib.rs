mod models;
mod id3;
mod lrc;
mod utils;

use models::{AudioInfo, Id3Info, LyricLine, ResultWrap};
use std::sync::Mutex;
use utils::wrap_anyhow;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// 获取mp3总时长，单位秒（仅元数据解析，不做播放）
fn get_mp3_duration(path: &str) -> anyhow::Result<f64> {
    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());
    let mut hint = Hint::new();
    hint.with_extension("mp3");
    let fmt_opts = FormatOptions::default();
    let meta_opts = MetadataOptions::default();
    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;
    let format = probed.format;
    let track = format
        .tracks()
        .get(0)
        .ok_or_else(|| anyhow::anyhow!("no audio track found"))?;
    let time_base = track.codec_params.time_base.ok_or_else(|| anyhow::anyhow!("no time base"))?;
    let total_ts = track.codec_params.n_frames.ok_or_else(|| anyhow::anyhow!("no frame count"))?;
    let dur = time_base.calc_time(total_ts);
    Ok(dur.seconds as f64 + dur.frac as f64 / 1_000_000_000.0)
}

/// Tauri全局状态：内存中正在编辑的歌词
#[derive(Debug, Default)]
pub struct AppEditState {
    pub current_mp3_path: Option<String>,
    pub original_id3: Option<Id3Info>,
    pub edit_lyrics: Vec<LyricLine>,
}

#[tauri::command]
fn read_mp3_meta(path: &str, edit_state: tauri::State<Mutex<AppEditState>>) -> ResultWrap<AudioInfo> {
    let id3_info = match id3::read_id3(path) {
        Ok(v) => v,
        Err(e) => return ResultWrap::fail(&e.to_string()),
    };
    let duration = match get_mp3_duration(path) {
        Ok(d) => d,
        Err(_) => 0.0,
    };

    let mut st = edit_state.lock().unwrap();
    st.current_mp3_path = Some(path.to_string());
    st.original_id3 = Some(id3_info.clone());
    st.edit_lyrics = lrc::plain_text_to_lyric_lines(&id3_info.uslt_lyric);

    let audio_info = AudioInfo {
        duration,
        id3: id3_info,
        lyrics: st.edit_lyrics.clone(),
    };
    ResultWrap::success(audio_info, "read mp3 meta ok")
}

#[tauri::command]
fn write_mp3_id3(path: &str, edited_id3: Id3Info) -> ResultWrap<()> {
    wrap_anyhow(id3::write_id3(path, &edited_id3))
}

#[tauri::command]
fn id3_check_diff(original: Id3Info, edited: Id3Info) -> ResultWrap<bool> {
    let diff = id3::id3_has_diff(&original, &edited);
    ResultWrap::success(diff, "ok")
}

#[tauri::command]
fn set_edit_lyrics(lyrics: Vec<LyricLine>, edit_state: tauri::State<Mutex<AppEditState>>) -> ResultWrap<()> {
    let mut st = edit_state.lock().unwrap();
    st.edit_lyrics = lyrics;
    ResultWrap::success((), "update edit lyrics ok")
}

#[tauri::command]
fn get_edit_lyrics(edit_state: tauri::State<Mutex<AppEditState>>) -> ResultWrap<Vec<LyricLine>> {
    let st = edit_state.lock().unwrap();
    ResultWrap::success(st.edit_lyrics.clone(), "ok")
}

#[tauri::command]
fn export_lrc_text(lyrics: Vec<LyricLine>) -> ResultWrap<String> {
    let lrc_str = lrc::export_lrc(&lyrics);
    ResultWrap::success(lrc_str, "ok")
}

#[tauri::command]
fn parse_lrc_text(lrc_text: &str) -> ResultWrap<Vec<LyricLine>> {
    wrap_anyhow(lrc::parse_lrc(lrc_text))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let edit_state = Mutex::new(AppEditState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(edit_state)
        .invoke_handler(tauri::generate_handler![
            read_mp3_meta,
            write_mp3_id3,
            id3_check_diff,
            set_edit_lyrics,
            get_edit_lyrics,
            export_lrc_text,
            parse_lrc_text
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
