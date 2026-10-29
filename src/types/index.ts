/** 单行歌词 */
export interface LyricLine {
  timestamps: number[]
  text: string
}

/** ID3标签信息（目前后端mock，先保留类型） */
export interface Id3Info {
  title: string
  artist: string
  album: string
  year: string
  track: string
  genre: string
  coverBase64: string
  usltLyric: string
}

/** 音频整体信息 */
export interface AudioInfo {
  duration: number
  id3: Id3Info
  lyrics: LyricLine[]
}

/** 后端统一返回包装 */
export interface ResultWrap<T> {
  ok: boolean
  msg: string
  data: T | null
}
