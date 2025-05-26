# bsky video downloader

download videos from bluesky posts with rust + wasm.

## development setup

```bash
git clone https://github.com/sillowww/bsky-video-dl
cd bsky-video-dl-rs
nix develop # if you use nix.
```

## building

```bash
wasm-pack build --target web --out-dir pkg
```

## usage

```js
import init, {
  check_has_video,
  get_video_info,
  download_video,
  resolve_handle,
  parse_url
} from './pkg/bsky_video_dl.js';

await init();

// check if a post has a video
const hasVideo = await check_has_video('https://bsky.app/profile/handle.bsky.social/post/abc123');

// get video metadata
const info = await get_video_info('https://bsky.app/profile/handle.bsky.social/post/abc123');
console.log(info.cid, info.mimeType, info.size, info.aspectRatio);

// download video as bytes
const videoBytes = await download_video('https://bsky.app/profile/handle.bsky.social/post/abc123');

// create download link
const blob = new Blob([videoBytes], { type: info.mimeType });
const url = URL.createObjectURL(blob);
const a = document.createElement('a');
a.href = url;
a.download = `video.${info.mimeType.split('/')[1]}`;
a.click();

// utility functions
const did = await resolve_handle('handle.bsky.social');
const [handle, rkey] = parse_url('https://bsky.app/profile/handle.bsky.social/post/abc123');
```

## copying

this project is licensed under the gnu agplv3, you can find a copy of the
license in [`COPYING`](./COPYING)
