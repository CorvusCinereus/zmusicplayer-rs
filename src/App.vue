<script setup>

import { onMounted, onUnmounted, ref } from 'vue';
import SettingPage from './SettingPage.vue';
import { message } from '@tauri-apps/plugin-dialog';
import { readDir } from '@tauri-apps/plugin-fs';
import { load } from '@tauri-apps/plugin-store';
import { invoke } from '@tauri-apps/api/core';

const tab = ref("music_list")
const search = ref("")
const music_list = ref([])
const current_index = ref(-1)
const playing = ref(false)
const path = ref("")
const loop = ref(false)
let store = undefined

async function on_delete_item(index) {
  if (current_index.value === index) {
    current_index.value = -1
    await stop()
  } else if (index < current_index.value && current_index.value != 0) {
    current_index.value -= 1
  }
  music_list.value.splice(index, 1)
}

async function on_play_button() {
  try {
    await invoke('switch')
  } catch (err) {
    await message(err, {kind: "error"})
  }
  playing.value = !playing.value
}

async function on_update(val) {
  path.value = val
  await store.set('path', val)

  music_list.value = []
  scan_dir(val)
}

async function scan_dir(dir) {
  try {
    const entries = await readDir(dir)
    for (const entry of entries) {
      if (entry.isDirectory) {
        scan_dir(`${path.value}/${entry.name}`)
      } else {
        music_list.value.push(entry.name)
      }
    }
  } catch (err) {
    await message(err, {kind: "error"})
  }
}

async function play_music(index) {
  try {
    await invoke('play_music', { path: `${path.value}/${music_list.value[index]}` })
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
  } catch (err) {
    await message(err, {kind: 'error'})
  }
}

async function next() {
  current_index.value += 1
  if (current_index.value === music_list.value.length) {
    current_index.value = 0
  }
  await play_music(current_index.value)
}

async function previous() {
  current_index.value -= 1
  if (current_index.value < 0) {
    current_index.value = music_list.value.length - 1
  }
  await play_music(current_index.value)
}

async function change_loop() {
  loop.value = !loop.value
  await store.set('loop', loop.value)
}

onMounted(async () => {
  store = await load('store.json', { autoSave: true })
  loop.value = await store.get('loop')
  path.value = await store.get("path")
  if (path.value) {
    scan_dir(path.value)
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
                <li class="music_item" :class="{active: current_index === index}" v-for="(item, index) in music_list" @click="() => play_music(index)">
                    <div class="music_item_title">{{item}}</div>
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
        <SettingPage class="setting_page" @update="on_update" :dir="path" v-else></SettingPage>
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

<style>
* { box-sizing: border-box; }
html, body { margin: 0; height: 100%; }
body {
  font-family: -apple-system, "Segoe UI", "PingFang SC", "Microsoft YaHei", sans-serif;
  color: #222;
  background: #fff;
}
button { font: inherit; cursor: pointer; }
input { font: inherit; outline: none; }
svg { display: block; }

.container {
  display: flex;
  flex-direction: column;
  height: 100vh;
  max-width: 420px;
  margin: 0 auto;
  border-left: 1px solid #eee;
  border-right: 1px solid #eee;
}

/* ---- 顶栏 ---- */
.appbar { flex: none; padding: 12px 16px; border-bottom: 1px solid #eee; }
.appbar_row { display: flex; align-items: center; gap: 8px; }
.appbar_row .icon { flex: none; color: #888; }
.search {
  flex: 1;
  padding: 8px 12px;
  border: none;
  border-radius: 8px;
  background: #f2f3f5;
}
.title { font-size: 18px; font-weight: 600; }

/* ---- 列表 ---- */
.center.fill { flex: 1; overflow-y: auto; }
.music_list_div ul { list-style: none; margin: 0; padding: 0; }
.music_item {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  padding: 14px 16px;
  border-bottom: 1px solid #f0f0f0;
  cursor: pointer;
}
.music_item:hover { background: #fafafa; }
.music_item.active { background: #eef4ff; color: #2f6fed; font-weight: 600; }
.music_item_title { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
.music_item_button {
  flex: none;
  display: flex;
  padding: 4px;
  border: none;
  border-radius: 50%;
  background: none;
  color: #bbb;
}
.music_item_button:hover { color: #e5484d; background: #fdecec; }

/* ---- 播放控制 ---- */
.control_row {
  flex: none;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 28px;
  padding: 12px 16px;
  border-top: 1px solid #eee;
}
.control_row button {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 44px;
  height: 44px;
  border: none;
  border-radius: 50%;
  background: none;
  color: #333;
}
.control_row button:hover { background: #f2f3f5; }
.control_row .play_button {
  width: 56px;
  height: 56px;
  background: #2f6fed;
  color: #fff;
}
.control_row .play_button:hover { background: #1f5ad6; }

/* ---- 底部 tab ---- */
.bottom_row_tab { flex: none; display: flex; border-top: 1px solid #eee; }
.bottom_col {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
  padding: 8px 0;
  color: #999;
  font-size: 12px;
  cursor: pointer;
}
.bottom_col.active { color: #2f6fed; }
</style>
