<script setup>

import { onMounted, onUnmounted, ref } from 'vue';
import SettingPage from './SettingPage.vue';
import { message } from '@tauri-apps/plugin-dialog';
import { readDir } from '@tauri-apps/plugin-fs';
import { load } from '@tauri-apps/plugin-store';
import { invoke } from '@tauri-apps/api/core';

const AUDIO_EXT = new Set(['mp3','flac','wav','ogg','m4a','aac','opus','wma','aiff','alac'])

const tab = ref("music_list")
const search = ref("")
const music_list = ref([])
const current_index = ref(-1)
const playing = ref(false)
const path = ref("")
const loop = ref(false)
const length = ref(0)
const now = ref(0)
const dragging = ref(false)
let store = null
let process_listener = null
let volume = 1.0

async function on_delete_item(index) {
  if (current_index.value === index) {
    current_index.value = -1
    await stop()
  } else if (index < current_index.value) {
    current_index.value -= 1
  }
  music_list.value.splice(index, 1)
}

async function on_play_button() {
  if (current_index.value === -1) return
  try {
    await invoke('switch')
    playing.value = !playing.value
  } catch (err) {
    await message(err, {kind: "error"})
  }
}

async function on_update(val) {
  if (playing.value) await stop()
  path.value = val
  await store.set('path', val)
  await store.save()

  music_list.value = []
  await scan_dir(val)
}

async function scan_dir(dir) {
  const result = []
  async function scan(dir) {
    try {
      const entries = await readDir(dir)
      for (const entry of entries) {
        if (entry.isDirectory) {
          await scan(`${dir}/${entry.name}`)
        } else {
          const ext = entry.name.split('.').pop()?.toLowerCase()
          if (AUDIO_EXT.has(ext)) result.push({name: entry.name, path: `${dir}/${entry.name}`})
        }
      }
    } catch (err) {
      await message(err, {kind: "error"})
    }
  }

  await scan(dir)
  music_list.value = result
}

function on_pointer_down(e) {
  dragging.value = true
  e.currentTarget.setPointerCapture(e.pointerId)
}

async function play_music(index) {
  now.value = 0
  try {
    length.value = await invoke('play_music', { path: music_list.value[index].path})
    current_index.value = index
    playing.value = true
  } catch (err) {
    await message(err, {kind: 'error'})
  }
}

async function stop() {
  try {
    await invoke('stop')
    current_index.value = -1
    playing.value = false
    now.value = 0
    length.value = 0
  } catch (err) {
    await message(err, {kind: 'error'})
  }
}

async function next() {
  const n = music_list.value.length
  if (!n) return
  const cur = current_index.value < 0 ? -1 : current_index.value
  await play_music((cur + 1) % n)
}

async function previous() {
  const n = music_list.value.length
  if (!n) return
  const cur = current_index.value < 0 ? 0 : current_index.value
  await play_music((cur - 1 + n) % n)
}

async function change_loop() {
  loop.value = !loop.value
  await store.set('loop', loop.value)
  await store.save()
}

async function on_seek() {
  try {
    if (length.value > 0 && now.value >= length.value) {
      if (loop.value) {
        await play_music(current_index.value)
      } else {
        await next()
      }
      return
    }
    await invoke('seek', {pos: now.value})
  } catch (err) {
    playing.value = false
    await stop()
    await message(err, {kind: 'error'})
  }
}

async function update_pos() {
  if (!playing.value || dragging.value) return
  let pos, finished
  try {
    [pos, finished] = await invoke('get_pos')
  } catch {
    return
  }
  now.value = pos
  if (finished && playing.value) {
    playing.value = false
    if (loop.value) {
      await play_music(current_index.value)
    } else {
      await next()
    }
  }
}

function format_duration(d) {
  return new Date(d * 1000).toISOString().slice(11, 19)
}

async function on_volume_change(val) {
  volume = val
  await store.set('volume', val)
  await store.save()
}

onMounted(async () => {
  store = await load('store.json', { autoSave: false })
  loop.value = (await store.get('loop')) ?? false
  path.value = (await store.get('path')) ?? ""
  volume = (await store.get('volume')) ?? 1.0
  if (path.value) {
    scan_dir(path.value)
  }

  try {
    await invoke('set_volume', {volume: volume})
  } catch (err) {
    await message(err, {kind: "error"})
  }

  process_listener = setInterval(update_pos, 500)
})

onUnmounted(() => {
  if (process_listener) {
    clearInterval(process_listener)
  }
})

</script>

<template>
  <main class="container">
    <div class="appbar">
        <div class="appbar_row" v-if="tab === 'music_list'">
            <svg class="icon" viewBox="0 0 24 24" width="16" height="16">
              <circle cx="11" cy="11" r="7" fill="none" stroke="currentColor" stroke-width="2" />
              <line x1="16.5" y1="16.5" x2="21" y2="21" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
            </svg>
            <input class="search" type="text" v-model="search"></input>
        </div>
        <label class="title" v-else>设置</label>
    </div>
    <div class="center fill">
        <div class="music_list_div" v-if="tab === 'music_list'">
            <ul>
                <li class="music_item" :class="{active: current_index === index}" v-for="(item, index) in music_list" v-show="item.name.includes(search)" @click="() => play_music(index)">
                    <div class="music_item_title">{{item.name}}</div>
                    <button class="music_item_button" @click.stop="on_delete_item(index)">
                        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                          <!-- 垃圾桶盖上的提手 -->
                          <path d="M9 4a3 3 0 0 1 6 0" />

                          <!-- 垃圾桶盖 -->
                          <path d="M3 5h18" />

                          <!-- 垃圾桶身 -->
                          <path d="M6 7l1.5 14h9L18 7" />

                          <!-- 桶身左侧竖线 -->
                          <line x1="10" y1="10" x2="10" y2="18" />

                          <!-- 桶身右侧竖线 -->
                          <line x1="14" y1="10" x2="14" y2="18" />
                        </svg>
                    </button>
                </li>
            </ul>
        </div>
        <SettingPage class="setting_page" @update="on_update" @volume_change="on_volume_change" :dir="path" :v="volume" v-else></SettingPage>
    </div>
    <div class="process_row" v-if="tab === 'music_list'">
        <label class="now">{{format_duration(now)}}</label>
        <input class="process_slider" type="range" min="0" :max="length" v-model.number="now" @change="on_seek" @pointerdown="on_pointer_down" @pointerup="dragging = false" @pointercancel="dragging = false" :style="{ '--fill': (length > 0 ? (now / length) * 100 : 0) + '%' }"></input>
        <label class="length">{{format_duration(length)}}</label>
    </div>
    <div class="control_row" v-if="tab === 'music_list'">
        <button class="previous_button" @click="previous">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
              <path d="M6 6h2v12H6zm12 0v12l-10-6z"/>
            </svg>
        </button>
        <button class="play_button" @click="on_play_button">
            <svg v-if="playing" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
              <rect x="6" y="5" width="4" height="14"/>
              <rect x="14" y="5" width="4" height="14"/>
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
              <polygon points="8,5 19,12 8,19"/>
            </svg>
        </button>
        <button class="next_button" @click="next">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24" fill="currentColor">
              <path d="M18 6h2v12h-2zM6 6v12l10-6z"/>
            </svg>
        </button>
        <button class="loop_button" @click="change_loop">
            <svg v-if="loop" xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 6H7a4 4 0 0 0-4 4v2"/>
              <path d="M3 18h14a4 4 0 0 0 4-4v-2"/>
              <path d="M17 2l4 4-4 4"/>
              <path d="M7 22l-4-4 4-4"/>
              <text x="12" y="12" dy=".35em" text-anchor="middle" font-family="system-ui, -apple-system, sans-serif" font-weight="bold" font-size="13" fill="currentColor" stroke="none">1</text>
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
              <path d="M21 6H7a4 4 0 0 0-4 4v2"/>
              <path d="M3 18h14a4 4 0 0 0 4-4v-2"/>
              <path d="M17 2l4 4-4 4"/>
              <path d="M7 22l-4-4 4-4"/>
            </svg>
        </button>
    </div>
    <div class="bottom_row_tab">
        <div class="bottom_col" :class="{active: tab === 'music_list'}" @click="tab = 'music_list'">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16">
              <circle cx="12" cy="12" r="10" fill="none" stroke="currentColor" stroke-width="1.5"/>
              <path d="M10 8.5v7l6-3.5z" fill="currentColor"/>
            </svg>
            <label class="music_list_label">播放</label>
        </div>
        <div class="bottom_col" :class="{active: tab === 'setting'}" @click="tab = 'setting'">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="16" height="16" fill="currentColor">
              <path d="M19.14 12.94c.04-.3.06-.61.06-.94 0-.32-.02-.64-.07-.94l2.03-1.58a.49.49 0 0 0 .12-.61l-1.92-3.32a.488.488 0 0 0-.59-.22l-2.39.96c-.5-.38-1.03-.7-1.62-.94l-.36-2.54a.484.484 0 0 0-.48-.41h-3.84c-.24 0-.43.17-.47.41l-.36 2.54c-.59.24-1.13.57-1.62.94l-2.39-.96c-.22-.08-.47 0-.59.22L2.74 8.87c-.12.21-.08.47.12.61l2.03 1.58c-.05.3-.09.63-.09.94s.02.64.07.94l-2.03 1.58a.49.49 0 0 0-.12.61l1.92 3.32c.12.22.37.29.59.22l2.39-.96c.5.38 1.03.7 1.62.94l.36 2.54c.05.24.24.41.48.41h3.84c.24 0 .44-.17.47-.41l.36-2.54c.59-.24 1.13-.56 1.62-.94l2.39.96c.22.08.47 0 .59-.22l1.92-3.32c.12-.22.07-.47-.12-.61l-2.01-1.58zM12 15.6c-1.98 0-3.6-1.62-3.6-3.6s1.62-3.6 3.6-3.6 3.6 1.62 3.6 3.6-1.62 3.6-3.6 3.6z"/>
            </svg>
            <label class="setting_label">设置</label>
        </div>
    </div>
  </main>
</template>

<style scoped>
/* ============ MD3 Tokens & 全局重置 ============ */
:global(html),
:global(body),
:global(#app) {
  height: 100%;
  margin: 0;
  padding: 0;
}

:global(body) {
  overflow: hidden;
}

*,
*::before,
*::after {
  box-sizing: border-box;
}

.container {
  --md-primary: #6750A4;
  --md-on-primary: #FFFFFF;
  --md-primary-container: #EADDFF;
  --md-on-primary-container: #21005D;
  --md-secondary-container: #E8DEF8;
  --md-on-secondary-container: #1D192B;
  --md-surface: #FEF7FF;
  --md-surface-container-low: #F7F2FA;
  --md-surface-container: #F3EDF7;
  --md-surface-container-high: #ECE6F0;
  --md-surface-container-highest: #E6E0E9;
  --md-on-surface: #1D1B20;
  --md-on-surface-variant: #49454F;
  --md-outline: #79747E;
  --md-outline-variant: #CAC4D0;

  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100vh;
  height: 100dvh;
  overflow: hidden;
  background: var(--md-surface);
  color: var(--md-on-surface);
  font-family: 'Roboto', system-ui, -apple-system, 'Segoe UI', sans-serif;
  -webkit-font-smoothing: antialiased;
}

/* ============ 顶部搜索栏 / 标题 ============ */
.appbar {
  flex: 0 0 auto;
  padding: 12px 16px 8px;
}

.appbar_row {
  display: flex;
  align-items: center;
  gap: 8px;
  height: 48px;
  padding: 0 16px 0 14px;
  border-radius: 28px;
  background: var(--md-surface-container-high);
  transition: background-color .2s, box-shadow .2s;
}

.appbar_row:focus-within {
  background: var(--md-surface-container-highest);
  box-shadow: inset 0 0 0 2px var(--md-primary);
}

.appbar_row .icon {
  flex: 0 0 auto;
  width: 20px;
  height: 20px;
  color: var(--md-on-surface-variant);
}

.search {
  flex: 1 1 auto;
  min-width: 0;
  height: 100%;
  border: none;
  outline: none;
  background: transparent;
  color: var(--md-on-surface);
  font-family: inherit;
  font-size: .95rem;
}

.search::placeholder {
  color: var(--md-on-surface-variant);
}

.title {
  display: block;
  padding: 0 4px;
  font-size: 1.375rem;
  font-weight: 400;
  line-height: 48px;
  color: var(--md-on-surface);
}

/* ============ 内容区（自适应填充剩余空间） ============ */
.center {
  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.fill {
  flex: 1 1 auto;
  min-height: 0;
}

.setting_page {
  flex: 1 1 auto;
  min-height: 0;
  width: 100%;
}

/* ============ 歌曲列表 ============ */
.music_list_div {
  flex: 1 1 auto;
  min-height: 0;
  padding: 4px 8px;
  overflow-y: auto;
  overflow-x: hidden;
}

.music_list_div::-webkit-scrollbar {
  width: 8px;
}

.music_list_div::-webkit-scrollbar-thumb {
  background: var(--md-outline-variant);
  border-radius: 4px;
}

.music_list_div::-webkit-scrollbar-track {
  background: transparent;
}

.music_list_div ul {
  display: flex;
  flex-direction: column;
  gap: 2px;
  margin: 0;
  padding: 0;
  list-style: none;
}

.music_item {
  display: flex;
  align-items: center;
  gap: 8px;
  min-height: 56px;
  padding: 0 8px 0 16px;
  border-radius: 12px;
  cursor: pointer;
  user-select: none;
  transition: background-color .2s, color .2s;
}

.music_item:hover {
  background: color-mix(in srgb, var(--md-on-surface) 8%, transparent);
}

.music_item.active {
  background: var(--md-secondary-container);
  color: var(--md-on-secondary-container);
}

.music_item.active:hover {
  background: color-mix(in srgb, var(--md-secondary-container) 92%, var(--md-on-secondary-container));
}

.music_item_title {
  flex: 1 1 auto;
  min-width: 0;
  font-size: .875rem;
  font-weight: 500;
  letter-spacing: .1px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.music_item_button {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--md-on-surface-variant);
  cursor: pointer;
  opacity: 0;
  transition: opacity .2s, background-color .2s, color .2s;
}

.music_item_button svg {
  width: 20px;
  height: 20px;
}

.music_item:hover .music_item_button,
.music_item.active .music_item_button {
  opacity: 1;
}

.music_item_button:hover {
  background: color-mix(in srgb, var(--md-on-surface) 12%, transparent);
  color: var(--md-on-surface);
}

/* ============ 进度条 ============ */
.process_row {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  gap: 12px;
  height: 40px;
  padding: 0 16px;
}

.now,
.length {
  flex: 0 0 auto;
  min-width: 44px;
  font-size: .75rem;
  font-variant-numeric: tabular-nums;
  color: var(--md-on-surface-variant);
  text-align: center;
}

.process_slider {
  flex: 1 1 auto;
  min-width: 0;
  height: 4px;
  margin: 0;
  border-radius: 2px;
  /* 已播放 / 未播放两段填充 */
  background: linear-gradient(
    to right,
    var(--md-primary) 0%,
    var(--md-primary) var(--fill, 0%),
    var(--md-outline-variant) var(--fill, 0%),
    var(--md-outline-variant) 100%
  );
  -webkit-appearance: none;
  appearance: none;
  outline: none;
  cursor: pointer;
}

/* 让原生 track 透明，露出 input 的渐变背景 */
.process_slider::-webkit-slider-runnable-track {
  height: 4px;
  border-radius: 2px;
  background: transparent;
}

.process_slider::-moz-range-track {
  height: 4px;
  border-radius: 2px;
  background: transparent;
}

.process_slider::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 14px;
  height: 14px;
  border: none;
  border-radius: 50%;
  background: var(--md-primary);
  cursor: pointer;
  transition: box-shadow .15s;
  margin-top: -5px;
}

.process_slider::-webkit-slider-thumb:hover {
  box-shadow: 0 0 0 8px color-mix(in srgb, var(--md-primary) 16%, transparent);
}

.process_slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border: none;
  border-radius: 50%;
  background: var(--md-primary);
  cursor: pointer;
}

/* ============ 播放控制 ============ */
.control_row {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 4px 16px 8px;
}

.control_row button {
  flex: 0 0 auto;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: 0;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--md-on-surface-variant);
  cursor: pointer;
  transition: background-color .2s, color .2s, transform .1s;
}

.control_row button:hover {
  background: color-mix(in srgb, var(--md-on-surface) 8%, transparent);
}

.control_row button:active {
  transform: scale(.94);
}

.previous_button,
.next_button,
.loop_button {
  width: 48px;
  height: 48px;
}

.loop_button {
  color: var(--md-on-surface-variant);
}

.play_button {
  width: 56px;
  height: 56px;
  margin: 0 8px;
  background: var(--md-primary);
  color: var(--md-on-primary);
}

.play_button:hover {
  background: color-mix(in srgb, var(--md-primary) 92%, var(--md-on-primary));
}

.play_button:active {
  background: color-mix(in srgb, var(--md-primary) 84%, var(--md-on-primary));
}

/* ============ 底部导航栏 ============ */
.bottom_row_tab {
  flex: 0 0 auto;
  display: flex;
  gap: 8px;
  height: 80px;
  padding: 8px 12px;
  background: var(--md-surface-container);
  border-top: 1px solid var(--md-outline-variant);
}

.bottom_col {
  flex: 1 1 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  border-radius: 16px;
  cursor: pointer;
  user-select: none;
  color: var(--md-on-surface-variant);
  background: transparent;
  transition: color .2s, background-color .2s;
}

.bottom_col:hover {
  background: color-mix(in srgb, var(--md-on-surface) 8%, transparent);
}

.bottom_col.active {
  color: var(--md-on-secondary-container);
  background: var(--md-secondary-container);
}

.bottom_col.active:hover {
  background: var(--md-secondary-container);
}

.bottom_col svg {
  width: 24px;
  height: 24px;
}

.bottom_col label {
  font-size: .75rem;
  font-weight: 500;
  line-height: 1;
  letter-spacing: .5px;
  cursor: pointer;
}

/* ============ 小屏适配 ============ */
@media (max-width: 420px) {
  .appbar {
    padding: 8px 12px 4px;
  }

  .process_row {
    gap: 8px;
    padding: 0 12px;
  }

  .control_row {
    gap: 4px;
    padding: 4px 12px 8px;
  }
}
</style>
