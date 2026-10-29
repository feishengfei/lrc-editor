use anyhow::{anyhow, Result};
use rodio::{Decoder, OutputStream, Sink};
use std::sync::{Arc, Mutex};
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

/// 全局音频状态，Tauri命令多线程访问，Arc‑Mutex包裹
#[derive(Debug, Default)]
pub struct AudioState {
    /// 当前加载音频文件路径
    pub file_path: Option<String>,
    /// 是否正在播放
    pub is_playing: bool,
    /// 音频总时长，单位秒；未加载为0.0
    pub duration: f64,
    sink_opt: Option<Sink>,
    // 注意：_stream必须保留，不能drop，否则音频直接停止
    _stream_opt: Option<OutputStream>,
}

pub type SharedAudioState = Arc<Mutex<AudioState>>;

/// 创建共享音频状态实例
pub fn create_audio_state() -> SharedAudioState {
    Arc::new(Mutex::new(AudioState::default()))
}

/// 使用 symphonia 获取mp3总时长（秒）
pub fn get_mp3_duration(path: &str) -> Result<f64> {
    let file = std::fs::File::open(path)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    hint.with_extension("mp3");

    let fmt_opts = FormatOptions::default();
    let meta_opts = MetadataOptions::default();

    let probed = symphonia::default::get_probe().format(&hint, mss, &fmt_opts, &meta_opts)?;
    let mut format = probed.format;

    let track = format
        .tracks()
        .get(0)
        .ok_or_else(|| anyhow!("no audio track found"))?;

    let time_base = track.time_base.ok_or_else(|| anyhow!("no time base"))?;
    let total_ts = track.codec_params.n_frames.ok_or_else(|| anyhow!("no frame count"))?;
    let dur = time_base.calc_time(total_ts);
    Ok(dur.seconds as f64 + dur.frac as f64 / 1_000_000_000.0)
}

/// 加载音频文件，不自动播放
pub fn load_audio(state: &SharedAudioState, path: &str) -> Result<()> {
    let mut st = state.lock().unwrap();

    // 停止并释放上一个sink
    if let Some(sink) = st.sink_opt.take() {
        sink.stop();
    }

    let (stream, stream_handle) = OutputStream::try_default()?;
    let file = std::fs::File::open(path)?;
    let decoder = Decoder::new(file)?;

    let sink = Sink::try_new(&stream_handle)?;
    sink.append(decoder);
    sink.pause();

    let dur = get_mp3_duration(path)?;

    st.file_path = Some(path.to_string());
    st.is_playing = false;
    st.duration = dur;
    st.sink_opt = Some(sink);
    st._stream_opt = Some(stream);

    Ok(())
}

/// 播放
pub fn play_audio(state: &SharedAudioState) {
    let mut st = state.lock().unwrap();
    if let Some(ref sink) = st.sink_opt {
        sink.play();
        st.is_playing = true;
    }
}

/// 暂停
pub fn pause_audio(state: &SharedAudioState) {
    let mut st = state.lock().unwrap();
    if let Some(ref sink) = st.sink_opt {
        sink.pause();
        st.is_playing = false;
    }
}

/// 获取当前播放位置（秒）
pub fn current_pos(state: &SharedAudioState) -> f64 {
    let st = state.lock().unwrap();
    st.sink_opt
        .as_ref()
        .map(|s| s.pos().as_secs_f64())
        .unwrap_or(0.0)
}

/// seek跳转到指定秒
pub fn seek_audio(state: &SharedAudioState, sec: f64) {
    let mut st = state.lock().unwrap();
    if let Some(ref mut sink) = st.sink_opt {
        let _ = sink.seek(sec);
    }
}

/// 获取总时长
pub fn get_duration(state: &SharedAudioState) -> f64 {
    let st = state.lock().unwrap();
    st.duration
}

/// 获取是否播放中
pub fn is_playing(state: &SharedAudioState) -> bool {
    let st = state.lock().unwrap();
    st.is_playing
}

