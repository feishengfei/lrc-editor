use base64::Engine;
use id3::{Tag, TagLike, Frame, Version, frame};
use std::fs::File;
use std::path::Path;
use crate::models::Id3Info;

/// 从mp3文件路径读取ID3标签信息
pub fn read_id3(path: &str) -> anyhow::Result<Id3Info> {
    let tag = Tag::read_from_path(path)?;

    let mut cover_base64 = String::new();
    // 读取APIC封面图片，取第一张图片
    if let Some(pic_frame) = tag.pictures().next() {
        cover_base64 = base64::engine::general_purpose::STANDARD.encode(&pic_frame.data);
    }

    // 读取USLT同步歌词帧，取第一个
    let mut uslt_lyric = String::new();
    for frame in tag.frames() {
        if let Frame::UserText(_) = frame {
        } else if let Frame::Other(ref inner) = frame {
            if let Ok(uslt) = frame::UnsynchronizedText::try_from(inner) {
                uslt_lyric = uslt.text;
                break;
            }
        }
    }

    let info = Id3Info {
        title: tag.title().unwrap_or_default().to_string(),
        artist: tag.artist().unwrap_or_default().to_string(),
        album: tag.album().unwrap_or_default().to_string(),
        year: tag.year().map(|v| v.to_string()).unwrap_or_default(),
        track: tag.track().map(|v| v.to_string()).unwrap_or_default(),
        genre: tag.genre().unwrap_or_default().to_string(),
        cover_base64,
        uslt_lyric,
    };

    Ok(info)
}

/// 将Id3Info写入MP3文件，版本使用ID3v2.4
pub fn write_id3(path: &str, info: &Id3Info) -> anyhow::Result<()> {
    let mut tag = match Tag::read_from_path(path) {
        Ok(t) => t,
        Err(_) => Tag::new(),
    };

    // 清空原有文本标签
    tag.set_title("");
    tag.set_artist("");
    tag.set_album("");
    tag.set_year(0i32);
    tag.set_track(0u32);
    tag.set_genre("");

    // 设置基础文本字段
    if !info.title.is_empty() {
        tag.set_title(info.title.as_str());
    }
    if !info.artist.is_empty() {
        tag.set_artist(info.artist.as_str());
    }
    if !info.album.is_empty() {
        tag.set_album(info.album.as_str());
    }
    if let Ok(y) = info.year.parse::<i32>() {
        tag.set_year(y);
    }
    if let Ok(track_num) = info.track.parse::<u32>() {
        tag.set_track(track_num);
    }
    if !info.genre.is_empty() {
        tag.set_genre(info.genre.as_str());
    }

    // 清除旧封面
    tag.remove_all_pictures();
    if !info.cover_base64.is_empty() {
        let img_bin = base64::engine::general_purpose::STANDARD.decode(info.cover_base64.as_bytes())?;
        let pic = frame::Picture {
            mime_type: "image/jpeg".to_string(),
            picture_type: frame::PictureType::CoverFront,
            description: "front cover".to_string(),
            data: img_bin,
        };
        tag.add_frame(Frame::Picture(pic));
    }

    // 删除全部旧USLT帧
    let mut keep_frames = Vec::new();
    for f in tag.frames() {
        if let Frame::Other(ref inner) = f {
            if frame::UnsynchronizedText::try_from(inner).is_ok() {
                continue;
            }
        }
        keep_frames.push(f.clone());
    }
    tag.set_frames(keep_frames);

    // 写入新USLT
    if !info.uslt_lyric.is_empty() {
        let uslt = frame::UnsynchronizedText {
            lang: b"eng".to_vec(),
            description: String::new(),
            text: info.uslt_lyric.clone(),
        };
        tag.add_frame(Frame::Other(frame::Content::UnsynchronizedText(uslt)));
    }

    // 写回原MP3文件，ID3v2.4
    let mut file = File::create(Path::new(path))?;
    tag.write_to(&mut file, Version::Id3v24)?;
    Ok(())
}

/// 判断两份Id3Info内容是否有差异，用于前端：不一致则激活保存按钮
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
