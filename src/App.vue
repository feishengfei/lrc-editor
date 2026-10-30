<template>
  <div class="app">
    <div class="toolbar">
      <button class="btn" @click="handleSelectMp3">📂打开MP3</button>
      <button class="btn" @click="importLrcModalOpen = true">📋粘贴歌词</button>
      <button class="btn" @click="handleImportLrcFile">📁导入LRC</button>
      <button class="btn" @click="handleExportLrc">💾保存到下载目录 output.lrc</button>
    </div>
    <div class="player">
      <audio ref="audioRef" @timeupdate="onTimeUpdate" @loadedmetadata="onAudioLoaded" />
      <div class="player-row">
        <button class="btn" @click="autoStampCurrentLine" :disabled="selectedIndex === null || !isPlaying || isPreviewMode" style="min-width:50px;">
          ⏱️计时并下一行
        </button>
        <button class="btn" @click="togglePlay">{{ isPlaying ? '⏸️暂停' : '▶️播放' }}</button>
        <span>{{ formatTime(currentTime) }} / {{ formatTime(duration) }}</span>
        <input
          type="range"
          min="0"
          :max="duration"
          step="0.01"
          v-model.number="seekVal"
          @change="onSeek"
          :disabled="!duration"
          style="flex:1;min-width:320px;"
        />
        <!-- 预览切换按钮：移除外层多余边框div -->
        <button class="btn btn-primary" @click="togglePreviewMode">
          {{ isPreviewMode ? '✏️编辑' : '🔍预览' }}
        </button>
      </div>
    </div>
    <div class="meta-inputs" style="display:flex;gap:12px;flex-wrap:wrap;margin:8px 0;">
        <label>
            标题(ti)：
            <input v-model="lrcTi" placeholder="歌曲标题" />
        </label>
        <label>
            歌手(ar)：
            <input v-model="lrcAr" placeholder="歌手" />
        </label>
        <label>
            专辑(al)：
            <input v-model="lrcAl" placeholder="专辑" />
        </label>
    </div>
    <div class="lyric-editor">
      <div class="lyric-tool">
          <button class="btn" @click="clearCurrentTimestamps" :disabled="selectedIndex === null || isPreviewMode">🧽清除本行时间</button>
          <button class="btn" @click="clearAllTimestamps" :disabled="isPreviewMode">⚠️清除全部时间戳</button>
          <span>|</span>
          <button class="btn" @click="insertAbove" :disabled="selectedIndex === null || isPreviewMode">⬆️选中行上方插入</button>
          <button class="btn" @click="insertBelow" :disabled="selectedIndex === null || isPreviewMode">⬇️选中行下方插入</button>
          <button class="btn" @click="deleteSelected" :disabled="selectedIndex === null || isPreviewMode">🗑️删除当前行</button>
          <button class="btn" @click="deleteAllLines" :disabled="isPreviewMode">❗删除所有行</button>
          <button class="btn" @click="moveUp" :disabled="selectedIndex === null || selectedIndex === 0 || isPreviewMode">🔼上移</button>
          <button class="btn" @click="moveDown" :disabled="selectedIndex === null || selectedIndex >= lyrics.length - 1 || isPreviewMode">🔽下移</button>
      </div>
      <div class="lyric-list">
        <div
          v-for="(line, idx) in lyrics"
          :key="idx"
          class="lyric-item"
          :class="{
            active: selectedIndex === idx,
            'preview-hl': isPreviewMode && previewActiveIdx === idx
          }"
          @click="!isPreviewMode && (selectedIndex = idx)"
        >
          <span class="ts">
            {{ line.timestamps.map(t => formatTime(t)).join(' / ') }}
          </span>
          <input v-model="line.text" placeholder="歌词文本" :disabled="isPreviewMode" />
        </div>
      </div>
    </div>
    <!-- 导入LRC弹窗 -->
    <div v-if="importLrcModalOpen" class="modal-mask" @click.self="importLrcModalOpen = false">
      <div class="modal">
        <h3>粘贴歌词</h3>
        <textarea v-model="importLrcContent" rows="12"></textarea>
        <div class="modal-buttons">
          <button class="btn" @click="importLrcModalOpen = false">取消</button>
          <button class="btn btn-primary" @click="handleImportLrc">确认导入</button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { invoke } from '@tauri-apps/api/core'
import { ref, nextTick } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-dialog';
import { readTextFile } from '@tauri-apps/plugin-fs';
import type { LyricLine } from './types'
import { parseLrcText, exportLrcText, selectMp3File } from './api/backend'
import { confirm } from '@tauri-apps/plugin-dialog';
const audioRef = ref<HTMLAudioElement | null>(null)
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const seekVal = ref(0)
const lyrics = ref<LyricLine[]>([])
const selectedIndex = ref<number | null>(null)
const importLrcModalOpen = ref(false)
const importLrcContent = ref('')
// LRC元信息 ti标题 ar歌手 al专辑
const lrcTi = ref('')
const lrcAr = ref('')
const lrcAl = ref('')
// ======预览模式新增状态======
const isPreviewMode = ref(false)
const previewActiveIdx = ref<number | null>(null)
// =============================
// 初始保留一个可编辑行
if (lyrics.value.length === 0) {
    lyrics.value.push({ timestamps: [], text: '' })
    selectedIndex.value = 0
}
// 切换预览/编辑模式
function togglePreviewMode(){
  isPreviewMode.value = !isPreviewMode.value
  if(!isPreviewMode.value){
    previewActiveIdx.value = null
  }
}
// 仅清空当前行全部时间戳，保留文本，不删除行
function clearCurrentTimestamps() {
  if (selectedIndex.value === null) return
  const line = lyrics.value[selectedIndex.value]
  line.timestamps = []
}
// 格式化秒 → 00:02.34
function formatTime(sec: number): string {
  const s = Math.max(0, sec)
  const m = Math.floor(s / 60)
  const rem = s - m * 60
  return `${String(m).padStart(2, '0')}:${rem.toFixed(2).padStart(5, '0')}`
}
async function handleSelectMp3() {
  const path = await selectMp3File()
  if (!path) return
  if (!audioRef.value) return
  const assetUrl = convertFileSrc(path)
  audioRef.value.src = assetUrl
  currentTime.value = 0
  seekVal.value = 0
  isPlaying.value = false
}
async function handleImportLrcFile() {
  const selected = await open({
    filters: [{ name: "LRC歌词", extensions: ["lrc"] }],
    multiple: false
  })
  if (selected === null) return;
  const filePath = Array.isArray(selected) ? selected[0] : selected;
  try {
    const content = await readTextFile(filePath);
    const ret = await parseLrcText(content);
    lrcTi.value = ret.ti
    lrcAr.value = ret.ar
    lrcAl.value = ret.al
    lyrics.value = ret.lines;
    selectedIndex.value = 0;
  } catch (err) {
    console.error('读取LRC文件失败', err);
    alert('读取LRC文件失败：' + err);
  }
}
function togglePlay() {
  if (!audioRef.value || !audioRef.value.src) return
  if (isPlaying.value) {
    audioRef.value.pause()
  } else {
    audioRef.value.play()
      .catch(err => {
        console.error('播放失败：', err)
        alert('音频播放失败，' + err.message)
      })
  }
  isPlaying.value = !isPlaying.value
}
function onTimeUpdate() {
  if (!audioRef.value) return
  currentTime.value = audioRef.value.currentTime
  seekVal.value = currentTime.value
  if(isPreviewMode.value){
    const now = currentTime.value
    let bestTs = -1.0
    let bestIdx:number|null = null
    lyrics.value.forEach((line, idx)=>{
      for(const ts of line.timestamps){
        if(ts <= now && ts > bestTs){
          bestTs = ts
          bestIdx = idx
        }else if(ts <= now && ts === bestTs){
          bestIdx = idx
        }
      }
    })
    const oldIdx = previewActiveIdx.value
    previewActiveIdx.value = bestIdx
    if(bestIdx !== null && oldIdx !== bestIdx){
      nextTick(()=>{
        const items = document.querySelectorAll('.lyric-item')
        if (bestIdx >=0 && bestIdx < items.length) {
          const item = items[bestIdx] as HTMLElement
          item.scrollIntoView({ block:'center', behavior:'smooth' })
        }
      })
    }
  }
}
function onAudioLoaded() {
  if (!audioRef.value) return
  duration.value = audioRef.value.duration
}
function onSeek() {
  if (!audioRef.value) return
  audioRef.value.currentTime = seekVal.value
  currentTime.value = seekVal.value
}
function insertAbove() {
  if (selectedIndex.value === null) return
  const idx = selectedIndex.value
  lyrics.value.splice(idx, 0, { timestamps: [], text: '' })
  selectedIndex.value = idx
}
function insertBelow() {
  if (selectedIndex.value === null) return
  const idx = selectedIndex.value
  lyrics.value.splice(idx + 1, 0, { timestamps: [], text: '' })
  selectedIndex.value = idx + 1
}
async function clearAllTimestamps() {
  const ok = await confirm('确定要清空全部时间戳？', { title: '确认操作', kind: 'warning' })
  if (!ok) return
  lyrics.value.forEach(line => { line.timestamps = [] })
}
async function deleteAllLines() {
  const ok = await confirm('确定要清空全部歌词？', { title: '确认操作', kind: 'warning' })
  if (!ok) return
  lyrics.value = [{ timestamps: [], text: '' }]
  selectedIndex.value = 0
}
function autoStampCurrentLine() {
  if (selectedIndex.value === null) return
  const line = lyrics.value[selectedIndex.value]
  line.timestamps.push(currentTime.value)
  line.timestamps.sort((a, b) => a - b)
  const needScroll = selectedIndex.value < lyrics.value.length - 1
  if (needScroll) {
    selectedIndex.value += 1
    nextTick(() => {
      const listEl = document.querySelector('.lyric-list') as HTMLElement | null
      const activeItem = document.querySelector('.lyric-item.active') as HTMLElement | null
      if (!listEl || !activeItem) return
      const itemRect = activeItem.getBoundingClientRect()
      const listRect = listEl.getBoundingClientRect()
      const isNearEnd = selectedIndex.value! >= lyrics.value.length - 2
      if (itemRect.bottom > listRect.bottom) {
        if (isNearEnd) {
          activeItem.scrollIntoView({ block: 'end', behavior: 'smooth' })
        } else {
          activeItem.scrollIntoView({ block: 'center', behavior: 'smooth' })
        }
      }
    })
  }
}
function deleteSelected() {
  if (selectedIndex.value === null) return
  lyrics.value.splice(selectedIndex.value, 1)
  selectedIndex.value = null
}
function moveUp() {
  if (selectedIndex.value === null || selectedIndex.value <= 0) return
  const i = selectedIndex.value
  const tmp = lyrics.value[i]
  lyrics.value[i] = lyrics.value[i - 1]
  lyrics.value[i - 1] = tmp
  selectedIndex.value = i - 1
}
function moveDown() {
  if (selectedIndex.value === null || selectedIndex.value >= lyrics.length - 1) return
  const i = selectedIndex.value
  const tmp = lyrics.value[i]
  lyrics.value[i] = lyrics.value[i + 1]
  lyrics.value[i + 1] = tmp
  selectedIndex.value = i + 1
}
async function handleImportLrc() {
  const ret = await parseLrcText(importLrcContent.value)
  lrcTi.value = ret.ti
  lrcAr.value = ret.ar
  lrcAl.value = ret.al
  lyrics.value = ret.lines
  selectedIndex.value = 0
  importLrcModalOpen.value = false
  importLrcContent.value = ''
}
async function handleExportLrc() {
  try {
    const lrcStr = await exportLrcText(lrcTi.value,lrcAr.value,lrcAl.value,lyrics.value)
    if (!lrcStr?.trim()) {
      alert('歌词内容为空，无法导出')
      return
    }
    const blob = new Blob([lrcStr], { type: 'text/plain;charset=utf‑8' })
    const url = URL.createObjectURL(blob)
    const a = document.createElement('a')
    a.href = url
    a.download = 'output.lrc'
    document.body.appendChild(a)
    a.click()
    document.body.removeChild(a)
    URL.revokeObjectURL(url)
  } catch (err) {
    console.error('导出失败', err)
    alert('❌ 导出失败：' + err)
  }
}
</script>

<!-- 不带scoped：全局字体，作用到html/body，解决input宋体问题 -->
<style>
:root, html, body {
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
  font-size: 14px;
  color: #222;
  background-color: #ffffff;
}
input, textarea, select {
  font-family: inherit;
  font-size: inherit;
}
</style>

<style scoped>
.app { padding: 16px; color: #222; }

.btn {
  font-family: inherit;
  font-size: 13px;
  padding: 6px 12px;
  border: none;
  border-radius: 6px;
  background-color: #f1f3f4;
  color: #202124;
  cursor: pointer;
  display: inline-flex;
  align-items: center;
  gap: 5px;
  transition: background-color 0.18s ease;
  user-select: none;
}
.btn:hover {
  background-color: #e8eaed;
}
.btn:active {
  background-color: #dadce0;
}
.btn:disabled {
  background-color: #f8f9fa;
  color: #9aa0a6;
  cursor: not-allowed;
}
.btn-primary {
  background-color: #1976d2;
  color: white;
}
.btn-primary:hover {
  background-color: #1565c0;
}
.btn-primary:active {
  background-color: #0d47a1;
}
.btn-primary:disabled {
  background-color: #90caf9;
  color: #e3f2fd;
}

input[type="text"] {
  border: 1px solid #dadce0;
  border-radius: 4px;
  padding: 4px 6px;
  outline: none;
  transition: border-color 0.18s;
}
input[type="text"]:focus {
  border-color: #1976d2;
}

.player { margin:12px 0; padding:12px; border:1px solid #eee; border-radius:6px; }
.player-row { display:flex; gap:12px; align-items:center; flex-wrap:nowrap; margin:8px 0; }
.lyric-tool { margin:8px 0; display:flex; gap:6px; align-items:center; }
.lyric-list { max-height: 60vh; overflow-y:auto; border:1px solid #eee; border-radius:6px; padding:8px; }
.lyric-item { display:flex; gap:8px; padding:4px 6px; margin:4px 0; border-radius:4px; cursor:pointer; }
.lyric-item.active { background: #e8f4ff; }

/* 预览高亮：蓝色边框 */
.lyric-item.preview-hl {
  border: 2px solid #409eff !important;
  box-sizing: border-box;
  background-color: #f0f7ff;
}
.lyric-item.active.preview-hl {
  background-color: #dcebff !important;
}

.ts { min-width: 120px; font-family: monospace; color:#555; }
.lyric-item input { flex:1; border:1px solid #ccc; padding:3px; }

.modal-mask { position:fixed; inset:0; background:rgba(0,0,0,0.4); display:grid; place-items:center; z:100; }
.modal { background:white; padding:16px; border-radius:8px; width:520px; }
.modal textarea { width:100%; box-sizing:border-box; }
.modal-buttons { margin-top:10px; display:flex; gap:8px; justify-content:flex-end; }
</style>
