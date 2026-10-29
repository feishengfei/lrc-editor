mod models;
mod id3;
mod audio;
mod lrc;
mod utils;

use audio::{create_audio_state, SharedAudioState};
use models::{AudioInfo, Id3Info, LyricLine, ResultWrap};
use std::sync::Mutex;
use utils::wrap_anyhow;

/// Tauri全局状态：内存中正在编辑的歌词，与播放器分离存储
#[derive(Debug, Default)]
pub struct AppEditState {
    pub current_mp3_path: Option<String>,
    pub original_id3: Option<Id3Info>,
    pub edit_lyrics: Vec<LyricLine>,
}

#[tauri::command]
fn read_mp3_meta(path: &str, audio_state: tauri::State<SharedAudioState>, edit_state: tauri::State<Mutex<AppEditState>>) -> ResultWrap<AudioInfo> {
    // 1.读取ID3标签
    let id3_info = match id3::read_id3(path) {
        Ok(v) => v,
        Err(e) => return ResultWrap::fail(&e.to_string()),
    };
    // 2.加载音频到播放器
    if let Err(e) = audio::load_audio(&audio_state, path) {
        return ResultWrap::fail(&e.to_string());
    }
    let duration = audio::get_duration(&audio_state);

    // 3.把原始信息存入AppEditState
    let mut st = edit_state.lock().unwrap();
    st.current_mp3_path = Some(path.to_string());
    st.original_id3 = Some(id3_info.clone());
    // USLT原始文本转成空时间戳的歌词行
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
fn export_lrc_text(edit_state: tauri::State<Mutex<AppEditState>>) -> ResultWrap<String> {
    let st = edit_state.lock().unwrap();
    let lrc_text = lrc::export_lrc(&st.edit_lyrics);
    ResultWrap::success(lrc_text, "ok")
}

#[tauri::command]
fn parse_lrc_text(lrc_text: &str) -> ResultWrap<Vec<LyricLine>> {
    wrap_anyhow(lrc::parse_lrc(lrc_text))
}

// --------播放器命令--------
#[tauri::command]
fn play(audio_state: tauri::State<SharedAudioState>) -> ResultWrap<()> {
    audio::play_audio(&audio_state);
    ResultWrap::success((), "play")
}

#[tauri::command]
fn pause(audio_state: tauri::State<SharedAudioState>) -> ResultWrap<()> {
    audio::pause_audio(&audio_state);
    ResultWrap::success((), "pause")
}

#[tauri::command]
fn current_position(audio_state: tauri::State<SharedAudioState>) -> ResultWrap<f64> {
    let pos = audio::current_pos(&audio_state);
    ResultWrap::success(pos, "ok")
}

#[tauri::command]
fn seek(audio_state: tauri::State<SharedAudioState>, sec: f64) -> ResultWrap<()> {
    audio::seek_audio(&audio_state, sec);
    ResultWrap::success((), format!("seek to {sec}").as_str())
}

#[tauri::command]
fn get_audio_duration(audio_state: tauri::State<SharedAudioState>) -> ResultWrap<f64> {
    let d = audio::get_duration(&audio_state);
    ResultWrap::success(d, "ok")
}

#[tauri::command]
fn get_is_playing(audio_state: tauri::State<SharedAudioState>) -> ResultWrap<bool> {
    let b = audio::is_playing(&audio_state);
    ResultWrap::success(b, "ok")
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let shared_audio = create_audio_state();
    let edit_state = Mutex::new(AppEditState::default());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .manage(shared_audio)
        .manage(edit_state)
        .invoke_handler(tauri::generate_handler![
            read_mp3_meta,
            write_mp3_id3,
            id3_check_diff,
            set_edit_lyrics,
            get_edit_lyrics,
            export_lrc_text,
            parse_lrc_text,
            play,
            pause,
            current_position,
            seek,
            get_audio_duration,
            get_is_playing
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
