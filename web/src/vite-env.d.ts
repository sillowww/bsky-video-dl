// bsky-video-dl/web/src/vite-env.d.ts
/// <reference types="vite/client" />

interface VideoInfo {
  cid: string;
  mimeType: string;
  size?: number;
  aspectRatio?: {
    width: number;
    height: number;
  };
}

interface Window {
  bskyVideoDownloader: {
    check_has_video: (url: string) => Promise<boolean>;
    get_video_info: (url: string) => Promise<VideoInfo>;
    download_video: (url: string) => Promise<Uint8Array>;
  };
}
