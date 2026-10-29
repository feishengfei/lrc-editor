use anyhow::Result;
use base64::Engine;
use lofty::file::AudioFile;
use lofty::id3::v2::{Id3v2Tag, UnsynchronizedTextFrame};
use lofty::picture::Picture;
use lofty::probe::Probe;
use lofty::tag::{Tag, TagType};
use std::fs::File;
use std::path::Path;

use crate::models::Id3Info;

/// 读取mp3 id3信息
pub fn read_id3(path: &str) -> Result<Id3Info> {
    let probe_file = Probe::open(path)?;
    let tagged_file = probe_file.read()?;
    let tag = tagged_file.tag(TagType::Id3v2)
        .cloned()
        .unwrap_or_else(Id3v2Tag::new);

    let title = tag.title().unwrap_or_default().to_string();
    let artist = tag.artist().unwrap_or_default().to_string();
    let album = tag.album().unwrap_or_default().to_string();
    let year = tag.year().unwrap_or_default().to_string();
    let track = tag.track().map(|t| t.to_string()).unwrap_or_default();
    let genre = tag.genre().unwrap_or_default().to_string();

    // 封面
    let mut cover_base64 = String::new();
    if let Some(pic) = tag.pictures().next() {
        cover_base64 = base64::engine::general_purpose::STANDARD.encode(pic.data());
    }

    // USLT 非同步歌词
    let mut uslt_lyric = String::new();
    for frame in tag.frames::<UnsynchronizedTextFrame>() {
        uslt_lyric = frame.text().to_string();
        break;
    }

    Ok(Id3Info {
        title,
        artist,
        album,
        year,
        track,
        genre,
        cover_base64,
        uslt_lyric,
    })
}

/// 写回id3标签到mp3
pub fn write_id3(path: &str, info: &Id3Info) -> Result<()> {
    let probe_file = Probe::open(path)?;
    let mut tagged_file = probe_file.read()?;
    // 获取或新建id3v2标签
    let mut tag = match tagged_file.tag_mut(TagType::Id3v2) {
        Some(t) => t.clone(),
        None => Id3v2Tag::new(),
    };

    // 清空旧文本
    tag.set_title("");
    tag.set_artist("");
    tag.set_album("");
    tag.set_year("");
    tag.set_track(0);
    tag.set_genre("");

    // 设置新字段
    if !info.title.is_empty() { tag.set_title(info.title.clone()); }
    if !info.artist.is_empty() { tag.set_artist(info.artist.clone()); }
    if !info.album.is_empty() { tag.set_album(info.album.clone()); }
    if !info.year.is_empty() { tag.set_year(info.year.clone()); }
    if let Ok(tr) = info.track.parse::<u32>() { tag.set_track(tr); }
    if !info.genre.is_empty() { tag.set_genre(info.genre.clone()); }

    // 清除所有旧封面
    tag.remove_pictures();
    if !info.cover_base64.is_empty() {
        let bin = base64::engine::general_purpose::STANDARD.decode(info.cover_base64.as_bytes())?;
        let pic = Picture::new_cover_front(bin);
        tag.push_picture(pic);
    }

    // 清除旧USLT
    tag.remove_frames::<UnsynchronizedTextFrame>();
    if !info.uslt_lyric.is_empty() {
        let uslt = UnsynchronizedTextFrame::new(b"eng".to_vec(), String::new(), info.uslt_lyric.clone());
        tag.push_frame(uslt);
    }

    // 写回文件
    *tagged_file.tag_mut_or_create(TagType::Id3v2) = Tag::Id3v2(tag);
    let mut f = File::create(Path::new(path))?;
    tagged_file.save_to(&mut f)?;
    Ok(())
}

/// 对比id3是否发生修改
pub fn id3_has_diff(original: &Id3Info, edited: &Id3Info) -> bool {
    original.title != edited.title
        || original.artist != edited.artist
        || original.album != edited.album
        || original.year != edited.year
        || original.track != edited.track
        || original.genre != edited.genre
        || original.cover_base64 != edited.cover_base64
        || original.uslt_lyric != edited.uslt_lyric
}

