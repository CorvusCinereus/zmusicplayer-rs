<script setup>

import { message } from '@tauri-apps/plugin-dialog';
import { open } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { onMounted, ref } from 'vue';

const volume = ref(1.0)
const path = ref("")
const emit = defineEmits(['update'])
const props = defineProps({
  dir: String
})

async function on_choose_dir() {
  const dir = await open({
    multiple: false,
    directory: true
  })
  path.value = dir
  emit('update', dir)
}

async function on_volume_input() {
  try {
    await invoke('set_volume', {volume: volume.value})
  } catch (err) {
    await message(err, {kind: "error"})
  }
}

onMounted(() => {
  path.value = props.dir
})

</script>

<template>
<div class="col">
    <div class="card">
        <label class="volume_label">音量</label>
        <div class="row">
            <svg v-if="volume != 0.0" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
              <!-- 喇叭主体：左侧矩形 + 右侧喇叭口 -->
              <path d="M11 5L6 9H3v6h3l5 4V5z" />
              <!-- 内层声波 -->
              <path d="M15.5 8.5a5 5 0 0 1 0 7" />
              <!-- 外层声波 -->
              <path d="M18.5 5.5a9 9 0 0 1 0 13" />
            </svg>
            <svg v-else xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="24" height="24"
                 fill="none" stroke="currentColor" stroke-width="2"
                 stroke-linecap="round" stroke-linejoin="round">
              <path d="M11 5L6 9H3v6h3l5 4V5z" />
              <!-- 叉号 -->
              <line x1="16" y1="9" x2="22" y2="15" />
              <line x1="22" y1="9" x2="16" y2="15" />
            </svg>
            <input class="slider" type="range" min="0.0" max="1.0" step="0.1" v-model.number="volume" @input="on_volume_input"></input>
        </div>
    </div>
    <div class="card">
        <div class="row">
            <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" width="64" height="64">
              <!-- 后壁:左上角带标签凸起 -->
              <path d="M2 7a2 2 0 0 1 2-2h4l2 2h8a2 2 0 0 1 2 2v10a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V7Z"
                    fill="#FFC93C"/>
              <!-- 前壁:略亮,压在后壁上方 -->
              <path d="M2 10h20v9a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-9Z"
                    fill="#FFD966"/>
            </svg>
            <label class="dir_label">本地音乐目录</label>
        </div>
        <div class="choose_div">
            <input class="path" type="text" v-model="path" placeholder="未选择目录" disabled></input>
            <button class="choose" @click="on_choose_dir">选择目录</button>
        </div>
    </div>
</div>
</template>

<style>
.col {
  display: flex;
  flex-direction: column;
  gap: 16px;
  padding: 16px;
  width: 100%;
}

.card {
  padding: 16px;
  border-radius: 12px;
  background: #f7f8fa;
}
.card > svg { margin-bottom: 8px; }

.volume_label,
.dir_label {
  display: block;
  margin-bottom: 8px;
  font-size: 13px;
  color: #888;
}

.row { display: flex; align-items: center; gap: 12px; }
.slider { flex: 1; accent-color: #2f6fed; }

.choose_div { display: flex; align-items: center; gap: 12px; }
.choose_col { flex: 1; display: flex; flex-direction: column; gap: 4px; min-width: 0; }

.path {
  width: 100%;
  padding: 8px 10px;
  border: 1px solid #e2e4e8;
  border-radius: 8px;
  background: #fff;
  color: #555;
}
.path:disabled { background: #fff; color: #999; }

.count { font-size: 12px; color: #aaa; }

.choose {
  flex: none;
  padding: 8px 14px;
  border: none;
  border-radius: 8px;
  background: #2f6fed;
  color: #fff;
}
.choose:hover { background: #1f5ad6; }
</style>
