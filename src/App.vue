<template>
  <div class="app">
    <div class="toolbar">
      <button @click="handleSelectMp3">📂打开MP3</button>
      <button @click="importLrcModalOpen = true">📋粘贴歌词</button>
      <button @click="handleImportLrcFile">📁导入LRC</button>
      <button @click="handleExportLrc">💾保存到下载目录 output.lrc</button>
    </div>

    <div class="player">
      <audio ref="audioRef" @timeupdate="onTimeUpdate" @loadedmetadata="onAudioLoaded" />
      <div class="player-row">
        <button @click="togglePlay">{{ isPlaying ? '⏸️暂停' : '▶️播放' }}</button>
        <span>{{ formatTime(currentTime) }} / {{ formatTime(duration) }}</span>
        <input
          type="range"
          min="0"
          :max="duration"
          step="0.01"
          v-model.number="seekVal"
          @change="onSeek"
          :disabled="!duration"
        />
      </div>
      <div class="player-row">
        <label>
          <input type="checkbox" v-model="autoNextLine">
          开启自动下一行（打点后自动跳到下一行）
        </label>
        <button @click="autoStampCurrentLine" :disabled="selectedIndex === null || !isPlaying">
          📌打点并下一行
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
          <button @click="insertAbove" :disabled="selectedIndex === null">⬆️选中行上方插入</button>
          <button @click="insertBelow" :disabled="selectedIndex === null">⬇️选中行下方插入</button>
          <button @click="clearCurrentTimestamps" :disabled="selectedIndex === null">⏱️清除本行时间</button>
          <button @click="clearAllTimestamps">⚠️清除全部时间戳</button>
          <button @click="deleteSelected" :disabled="selectedIndex === null">🗑️删除当前行</button>
          <button @click="deleteAllLines">❗删除所有行</button>
          <button @click="moveUp" :disabled="selectedIndex === null || selectedIndex === 0">🔼上移</button>
          <button @click="moveDown" :disabled="selectedIndex === null || selectedIndex >= lyrics.length - 1">🔽下移</button>
      </div>

      <div class="lyric-list">
        <div
          v-for="(line, idx) in lyrics"
          :key="idx"
          class="lyric-item"
          :class="{active: selectedIndex === idx}"
          @click="selectedIndex = idx"
        >
          <span class="ts">
            {{ line.timestamps.map(t => formatTime(t)).join(' / ') }}
          </span>
          <input v-model="line.text" placeholder="歌词文本" />
        </div>
      </div>
    </div>

    <!-- 导入LRC弹窗 -->
    <div v-if="importLrcModalOpen" class="modal-mask" @click.self="importLrcModalOpen = false">
      <div class="modal">
        <h3>粘贴歌词</h3>
        <textarea v-model="importLrcContent" rows="12"></textarea>
        <div class="modal-buttons">
          <button @click="importLrcModalOpen = false">取消</button>
          <button @click="handleImportLrc">确认导入</button>
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


const audioRef = ref<HTMLAudioElement | null>(null)
const isPlaying = ref(false)
const currentTime = ref(0)
const duration = ref(0)
const seekVal = ref(0)

const lyrics = ref<LyricLine[]>([])
const selectedIndex = ref<number | null>(null)
const autoNextLine = ref(true)

const importLrcModalOpen = ref(false)
const importLrcContent = ref('')

// LRC元信息 ti标题 ar歌手 al专辑
const lrcTi = ref('')
const lrcAr = ref('')
const lrcAl = ref('')

// 初始保留一个可编辑行
if (lyrics.value.length === 0) {
    lyrics.value.push({ timestamps: [], text: '' })
    selectedIndex.value = 0
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

  // ✅ Tauri2 正确方式：convertFileSrc
  const assetUrl = convertFileSrc(path)
  audioRef.value.src = assetUrl
  currentTime.value = 0
  seekVal.value = 0
  isPlaying.value = false
}


// 通过系统文件对话框选择本地 .lrc 文件导入
async function handleImportLrcFile() {
  const selected = await open({
    filters: [{
      name: "LRC歌词",
      extensions: ["lrc"]
    }],
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

// 在选中行上方插入空行
function insertAbove() {
  if (selectedIndex.value === null) return
  const idx = selectedIndex.value
  lyrics.value.splice(idx, 0, { timestamps: [], text: '' })
  selectedIndex.value = idx
}

// 在选中行下方插入空行
function insertBelow() {
  if (selectedIndex.value === null) return
  const idx = selectedIndex.value
  lyrics.value.splice(idx + 1, 0, { timestamps: [], text: '' })
  selectedIndex.value = idx + 1
}

// 清除全部行的时间戳，保留文本
function clearAllTimestamps() {
  lyrics.value.forEach(line => {
    line.timestamps = []
  })
}

// 删除所有歌词行
function deleteAllLines() {
  lyrics.value = [{ timestamps: [], text: '' }]
  selectedIndex.value = 0
}


// 打点并自动切下一行
function autoStampCurrentLine() {
  if (selectedIndex.value === null) return
  const line = lyrics.value[selectedIndex.value]
  line.timestamps.push(currentTime.value)
  line.timestamps.sort((a, b) => a - b)

  const needScroll = autoNextLine.value && selectedIndex.value < lyrics.value.length - 1
  if (needScroll) {
    selectedIndex.value += 1
    // 仅自动跳转场景执行滚动
    nextTick(() => {
      const listEl = document.querySelector('.lyric-list') as HTMLElement | null
      const activeItem = document.querySelector('.lyric-item.active') as HTMLElement | null
      if (!listEl || !activeItem) return

      const listRect = listEl.getBoundingClientRect()
      const itemRect = activeItem.getBoundingClientRect()
      // 判断是否靠近列表末尾：最后2行则不做居中偏移
      const isNearEnd = selectedIndex.value! >= lyrics.value.length - 2

      // 只有元素底部跑出容器可视区域才滚动
      if (itemRect.bottom > listRect.bottom) {
        if (isNearEnd) {
          // 末尾：直接滚动到让元素底部贴容器底部
          activeItem.scrollIntoView({ block: 'end', behavior: 'smooth' })
        } else {
          // 中间区域：尽量放到视口偏中下位置
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
  if (selectedIndex.value === null || selectedIndex.value >= lyrics.value.length - 1) return
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

<style scoped>
.app { padding: 16px; color: #222; }
.toolbar button { margin-right: 8px; padding: 4px 8px; }
.player { margin:12px 0; padding:12px; border:1px solid #eee; border-radius:6px; }
.player-row { display:flex; gap:10px; align-items:center; flex-wrap:wrap; margin:6px 0; }
.lyric-tool { margin:8px 0; }
.lyric-tool button { margin-right:6px; padding:4px 8px; }
.lyric-list { max-height: 60vh; overflow-y:auto; border:1px solid #eee; border-radius:6px; padding:8px; }
.lyric-item { display:flex; gap:8px; padding:4px 6px; margin:4px 0; border-radius:4px; cursor:pointer; }
.lyric-item.active { background: #e8f4ff; }
.ts { min-width: 120px; font-family: monospace; color:#555; }
.lyric-item input { flex:1; border:1px solid #ccc; padding:3px; }
.modal-mask { position:fixed; inset:0; background:rgba(0,0,0,0.4); display:grid; place-items:center; z:100; }
.modal { background:white; padding:16px; border-radius:8px; width:520px; }
.modal textarea { width:100%; box-sizing:border-box; }
.modal-buttons { margin-top:10px; display:flex; gap:8px; justify-content:flex-end; }
</style>
