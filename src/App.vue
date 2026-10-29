<template>
  <div class="app">
    <div class="toolbar">
      <button @click="handleSelectMp3">打开MP3</button>
      <button @click="importLrcModalOpen = true">导入LRC</button>
      <button @click="handleExportLrc">保存到下载目录 output.lrc</button>
    </div>

    <div class="player">
      <audio ref="audioRef" @timeupdate="onTimeUpdate" @loadedmetadata="onAudioLoaded" />
      <div class="player-row">
        <button @click="togglePlay">{{ isPlaying ? '暂停' : '播放' }}</button>
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
          ✅ 打点并下一行
        </button>
      </div>
    </div>

    <div class="lyric-editor">
      <div class="lyric-tool">
        <button @click="addLyricLine">新增一行</button>
        <button @click="deleteSelected" :disabled="selectedIndex === null">删除</button>
        <button @click="moveUp" :disabled="selectedIndex === null || selectedIndex === 0">上移</button>
        <button @click="moveDown" :disabled="selectedIndex === null || selectedIndex >= lyrics.length - 1">下移</button>
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
        <h3>粘贴LRC文本</h3>
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
import { ref } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
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

// 歌词操作
function addLyricLine() {
  lyrics.value.push({ timestamps: [], text: '' })
  selectedIndex.value = lyrics.value.length - 1
}

// 打点并自动切下一行
function autoStampCurrentLine() {
  if (selectedIndex.value === null) return
  const line = lyrics.value[selectedIndex.value]
  line.timestamps.push(currentTime.value)
  line.timestamps.sort((a, b) => a - b)

  if (autoNextLine.value && selectedIndex.value < lyrics.value.length - 1) {
    selectedIndex.value += 1
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
  const list = await parseLrcText(importLrcContent.value)
  lyrics.value = list
  selectedIndex.value = 0
  importLrcModalOpen = false
  importLrcContent.value = ''
}

async function handleExportLrc() {
  try {
    // 新增调试打印，看原始invoke返回
    const rawRes = await invoke('export_lrc_text', { lyrics: lyrics.value })
    console.log('===raw invoke返回===', rawRes)

    // 原来逻辑
    const lrcStr = rawRes.data
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
