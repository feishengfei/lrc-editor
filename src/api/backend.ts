import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog'

/** 读取mp3元信息（时长、歌词数组、id3） */
export async function readMp3Meta(path: string): Promise<AudioInfo> {
  const res = await invoke<ResultWrap<AudioInfo>>('read_mp3_meta', { path })
  if (!res.ok || !res.data) throw new Error(res.msg)
  return res.data
}

/** 解析LRC字符串 → LyricLine[] */
export async function parseLrcText(lrcText: string): Promise<LyricLine[]> {
  const res = await invoke<ResultWrap<LyricLine[]>>('parse_lrc_text', { lrcText })
  if (!res.ok || !res.data) throw new Error(res.msg)
  return res.data
}

/** 导出歌词数组 → LRC字符串 */
export async function exportLrcText(lyrics: LyricLine[]): Promise<string> {
  const res = await invoke<ResultWrap<string>>('export_lrc_text', { lyrics })
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
