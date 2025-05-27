import type { VideoInfo } from "../types";

class VideoDownloader extends HTMLElement {
  container: HTMLDivElement;
  videoInfoEl: HTMLDivElement;
  videoPreviewEl: HTMLDivElement;
  downloadBtn: HTMLButtonElement;
  statusEl: HTMLDivElement;
  currentUrl = "";
  videoInfo: VideoInfo | null = null;
  videoBlob: Blob | null = null;

  constructor() {
    super();

    // create elements
    this.container = document.createElement("div");
    this.container.className = "video-downloader-container";

    // create video preview container
    this.videoPreviewEl = document.createElement("div");
    this.videoPreviewEl.className = "video-preview";

    this.videoInfoEl = document.createElement("div");
    this.videoInfoEl.className = "video-info";

    this.downloadBtn = document.createElement("button");
    this.downloadBtn.className = "download-btn";
    this.downloadBtn.textContent = "download video";
    this.downloadBtn.addEventListener("click", () => this.downloadVideo());

    this.statusEl = document.createElement("div");
    this.statusEl.className = "status";

    // append elements
    this.container.appendChild(this.videoPreviewEl);
    this.container.appendChild(this.videoInfoEl);
    this.container.appendChild(this.downloadBtn);
    this.container.appendChild(this.statusEl);

    this.appendChild(this.container);

    // listen for url validation event
    document.addEventListener("url-validated", (e: Event) => {
      const event = e as CustomEvent;
      this.handleValidatedUrl(event.detail.url);
    });

    this.videoInfoEl.style.display = "none";
    this.videoPreviewEl.style.display = "none";
    this.downloadBtn.style.display = "none";
  }

  async handleValidatedUrl(url: string) {
    this.currentUrl = url;

    this.container.className = "video-downloader-container active";
    this.videoInfoEl.style.display = "none";
    this.videoPreviewEl.style.display = "none";
    this.downloadBtn.style.display = "none";

    this.statusEl.style.display = "block";
    this.statusEl.className = "status";
    this.statusEl.textContent = "fetching video information...";

    try {
      this.videoInfo = await window.bskyVideoDownloader.get_video_info(url);
      console.log(this.videoInfo);

      this.renderVideoInfo();
      this.videoInfoEl.style.display = "block";

      // load the video bytes for preview
      this.statusEl.textContent = "loading video preview...";
      const videoBytes = await window.bskyVideoDownloader.download_video(
        this.currentUrl,
      );

      // create blob & video preview
      this.videoBlob = new Blob([videoBytes], {
        type: this.videoInfo?.mimeType,
      });
      this.renderVideoPreview();
      this.videoPreviewEl.style.display = "block";

      this.downloadBtn.disabled = false;
      this.downloadBtn.style.display = "block";

      this.statusEl.textContent = "ready to download";
    } catch (error) {
      this.statusEl.className = "status error";
      this.statusEl.textContent =
        error instanceof Error
          ? error.message
          : "failed to get video information";
    }
  }

  renderVideoInfo() {
    if (!this.videoInfo) return;

    let infoHtml = "<h3>video information</h3>";

    const formatSize = (bytes: number) => {
      if (bytes < 1024) return `${bytes} bytes`;
      if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
      return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
    };

    infoHtml += `
      <div class="info-row">
        <span class="info-label">format</span>
        <span>${this.videoInfo.mimeType.split("/")[1].toUpperCase()}</span>
      </div>
    `;

    if (this.videoInfo.size) {
      infoHtml += `
        <div class="info-row">
          <span class="info-label">file size</span>
          <span>${formatSize(this.videoInfo.size)}</span>
        </div>
      `;
    }

    if (this.videoInfo.aspectRatio) {
      infoHtml += `
        <div class="info-row">
          <span class="info-label">resolution</span>
          <span>${this.videoInfo.aspectRatio.width} × ${this.videoInfo.aspectRatio.height}</span>
        </div>
      `;
    }

    this.videoInfoEl.innerHTML = infoHtml;
  }

  renderVideoPreview() {
    if (!this.videoBlob || !this.videoInfo) return;

    const videoUrl = URL.createObjectURL(this.videoBlob);

    // create video element
    const video = document.createElement("video");
    video.controls = true;
    video.autoplay = false;
    video.muted = true;
    video.src = videoUrl;
    video.className = "preview-video";

    // set aspect ratio if available
    if (this.videoInfo.aspectRatio) {
      const ratio =
        (this.videoInfo.aspectRatio.height / this.videoInfo.aspectRatio.width) *
        100;
      const container = document.createElement("div");
      container.className = "video-aspect-container";
      container.style.paddingBottom = `${ratio}%`;
      container.appendChild(video);
      this.videoPreviewEl.innerHTML = "";
      this.videoPreviewEl.appendChild(container);
    } else {
      this.videoPreviewEl.innerHTML = "";
      this.videoPreviewEl.appendChild(video);
    }

    // add event listener to clean up the blob URL when the component is removed
    this.addEventListener("disconnectedCallback", () => {
      URL.revokeObjectURL(videoUrl);
    });
  }

  async downloadVideo() {
    if (!this.currentUrl || !this.videoBlob || !this.videoInfo) return;

    this.downloadBtn.disabled = true;
    this.downloadBtn.textContent = "downloading...";

    try {
      const fileExt = this.videoInfo.mimeType.split("/")[1] || "mp4";
      const url = URL.createObjectURL(this.videoBlob);

      const a = document.createElement("a");
      a.href = url;
      a.download = `bluesky_video.${fileExt}`;
      document.body.appendChild(a);
      a.click();
      document.body.removeChild(a);
      URL.revokeObjectURL(url);

      this.statusEl.textContent = "download complete!";
      this.statusEl.className = "status success";
      this.downloadBtn.textContent = "download again";
    } catch (error) {
      this.statusEl.className = "status error";
      this.statusEl.textContent =
        error instanceof Error ? error.message : "failed to download video";
    } finally {
      this.downloadBtn.disabled = false;
    }
  }
}

customElements.define("video-downloader", VideoDownloader);
