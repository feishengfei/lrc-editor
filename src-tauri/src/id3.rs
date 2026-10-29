use anyhow::Result;
use crate::models::Id3Info;

/// 【mock占位实现】读取mp3 id3信息，待后续替换真实元数据库
pub fn read_id3(_path: &str) -> Result<Id3Info> {
    // 返回空标签；真实实现后续替换
    Ok(Id3Info {
        title: String::new(),
        artist: String::new(),
        album: String::new(),
        year: String::new(),
        track: String::new(),
        genre: String::new(),
        cover_base64: String::new(),
        uslt_lyric: String::new(),
    })
}

/// 【mock占位实现】写回id3标签到mp3，待后续替换真实元数据库
pub fn write_id3(_path: &str, _info: &Id3Info) -> Result<()> {
    // mock：什么都不做，返回ok
    Ok(())
}

/// 判断两份Id3Info内容是否有差异
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
