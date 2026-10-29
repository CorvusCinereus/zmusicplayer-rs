# zmusicplayer-rs

基于[`Tauri`](https://tauri.app/)和`Rust`制作的简易音乐播放器

## 构建

```bash
pnpm install
pnpm tauri build
```

## 平台相关问题

### Debian

经过测试在Debian运行可能会出现如下报错

```bash
ALSA lib pcm_dmix.c:1000:(snd_pcm_dmix_open) unable to open slave
```

解决方法

```bash
sudo apt install pipewire-alsa
```

## LICENSE

本项目采用[Apache-2.0](LICENSE)
