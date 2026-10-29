<script setup>

import { open, message } from '@tauri-apps/plugin-dialog';
import { invoke } from '@tauri-apps/api/core';
import { onMounted, ref } from 'vue';

const volume = ref(1.0)
const path = ref("")
const emit = defineEmits(['update', 'volume_change'])
const props = defineProps({
  dir: String,
  v: Number
})

async function on_choose_dir() {
  const dir = await open({
    multiple: false,
    directory: true
  })
  if (!dir) return
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

function on_volume_change() {
  emit('volume_change', volume.value)
}

onMounted(async () => {
  path.value = props.dir
  volume.value = props.v
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
            <input class="slider" type="range" min="0.0" max="1.0" step="0.1" v-model.number="volume" @input="on_volume_input" @change="on_volume_change"  @pointerup="on_volume_change" :style="{ '--fill': (volume * 100) + '%' }"></input>
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

<style scoped>
/* ============ MD3 Tokens ============ */
*,
*::before,
*::after {
  box-sizing: border-box;
}

.col {
  --md-primary: #6750A4;
  --md-on-primary: #FFFFFF;
  --md-primary-container: #EADDFF;
  --md-surface-container-low: #F7F2FA;
  --md-surface-container: #F3EDF7;
  --md-surface-container-high: #ECE6F0;
  --md-on-surface: #1D1B20;
  --md-on-surface-variant: #49454F;
  --md-outline: #79747E;
  --md-outline-variant: #CAC4D0;

  flex: 1 1 auto;
  min-height: 0;
  display: flex;
  flex-direction: column;
  gap: 12px;
  width: 100%;
  padding: 8px 16px 16px;
  overflow-y: auto;
}

.col::-webkit-scrollbar {
  width: 8px;
}

.col::-webkit-scrollbar-thumb {
  background: var(--md-outline-variant);
  border-radius: 4px;
}

.col::-webkit-scrollbar-track {
  background: transparent;
}

/* ============ 卡片 ============ */
.card {
  flex: 0 0 auto;
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 16px;
  border-radius: 16px;
  background: var(--md-surface-container-low);
}

/* ============ 音量 ============ */
.volume_label {
  font-size: .875rem;
  font-weight: 500;
  letter-spacing: .1px;
  color: var(--md-on-surface-variant);
}

.row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.row svg {
  flex: 0 0 auto;
  color: var(--md-primary);
}

.slider {
  flex: 1 1 auto;
  min-width: 0;
  height: 4px;
  margin: 0;
  border-radius: 2px;
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

.slider::-webkit-slider-runnable-track {
  height: 4px;
  border-radius: 2px;
  background: transparent;
}

.slider::-moz-range-track {
  height: 4px;
  border-radius: 2px;
  background: transparent;
}

.slider::-webkit-slider-thumb {
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

.slider::-webkit-slider-thumb:hover {
  box-shadow: 0 0 0 8px color-mix(in srgb, var(--md-primary) 16%, transparent);
}

.slider::-moz-range-thumb {
  width: 14px;
  height: 14px;
  border: none;
  border-radius: 50%;
  background: var(--md-primary);
  cursor: pointer;
}

/* ============ 目录选择 ============ */
.dir_label {
  font-size: 1rem;
  font-weight: 500;
  color: var(--md-on-surface);
}

.choose_div {
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  gap: 8px;
}

.path {
  flex: 1 1 160px;
  min-width: 0;
  height: 40px;
  padding: 0 12px;
  border: 1px solid var(--md-outline-variant);
  border-radius: 8px;
  background: transparent;
  color: var(--md-on-surface-variant);
  font-family: inherit;
  font-size: .875rem;
  text-overflow: ellipsis;
  outline: none;
}

.path:disabled {
  opacity: 1;
  cursor: default;
}

.choose {
  flex: 0 0 auto;
  height: 40px;
  padding: 0 24px;
  border: none;
  border-radius: 20px;
  background: var(--md-primary);
  color: var(--md-on-primary);
  font-family: inherit;
  font-size: .875rem;
  font-weight: 500;
  letter-spacing: .1px;
  white-space: nowrap;
  cursor: pointer;
  transition: background-color .2s, box-shadow .2s;
}

.choose:hover {
  background: color-mix(in srgb, var(--md-primary) 92%, var(--md-on-primary));
  box-shadow: 0 1px 2px rgba(0, 0, 0, .3), 0 1px 3px 1px rgba(0, 0, 0, .15);
}

.choose:active {
  background: color-mix(in srgb, var(--md-primary) 84%, var(--md-on-primary));
}
</style>
