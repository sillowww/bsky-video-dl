export interface VideoInfo {
  cid: string;
  mimeType: string;
  size?: number;
  aspectRatio?: {
    width: number;
    height: number;
  };
}
