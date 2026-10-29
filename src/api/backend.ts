import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

export interface LrcParseResult {
  ti:string
  ar:string
  al:string
  lines:LyricLine[]
}

/** 读取mp3元信息（时长、歌词数组、id3） */
export async function readMp3Meta(path: string): Promise<AudioInfo> {
  const res = await invoke<ResultWrap<AudioInfo>>('read_mp3_meta', { path })
  if (!res.ok || !res.data) throw new Error(res.msg)
  return res.data
}

/** 解析LRC字符串 */
export async function parseLrcText(text:string):Promise<LrcParseResult>{
  const res = await invoke<ResultWrap<LrcParseResult>>("parse_lrc_text",{lrc_text:text})
  if (!res.ok || !res.data) throw new Error(res.msg)
  return res.data
}

/** 导出歌词数组 → LRC字符串 */
export async function exportLrcText(ti:string,ar:string,al:string,lyrics:LyricLine[]):Promise<string>{
  const res = await invoke<ResultWrap<string>>("export_lrc_text",{ti,ar,al,lyrics})
  if (!res.ok || !res.data) throw new Error(res.msg)
  return res.data
}

/** 设置内存编辑歌词 */
export async function setEditLyrics(lyrics: LyricLine[]): Promise<void> {
  const res = await invoke<ResultWrap<void>>('set_edit_lyrics', { lyrics })
  if (!res.ok) throw new Error(res.msg)
}

/** 获取当前编辑歌词 */
export async function getEditLyrics(): Promise<LyricLine[]> {
  const res = await invoke<ResultWrap<LyricLine[]>>('get_edit_lyrics')
  if (!res.ok || !res.data) throw new Error(res.msg)
  return res.data
}

/** 打开文件选择框，选择mp3 */
export async function selectMp3File() {
  const selected = await open({
    multiple: false,
    filters: [{ name: 'MP3 Audio', extensions: ['mp3'] }]
  })
  return selected as string | null
}
