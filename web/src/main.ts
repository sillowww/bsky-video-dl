import "./components/url-input.ts";
import "./components/video-downloader.ts";
import "./style.css";

const appMount =
  document.querySelector<HTMLDivElement>("#app") ||
  document.createElement("div");

appMount.innerHTML = `
  <header>
    <h1>bsky video downloader</h1>
  </header>

  <main>
    <url-input></url-input>
    <video-downloader></video-downloader>
  </main>

  <footer>
    <p>made with <span class="heart">♥</span>  by willow · <a href="https://github.com/sillowww/bsky-video-dl">source code</a></p>
  </footer>
`;

async function setupWasm() {
  try {
    const wasmModule = await import("../pkg/bsky_video_dl.js");
    await wasmModule.default();

    window.bskyVideoDownloader = {
      check_has_video: wasmModule.check_has_video,
      get_video_info: wasmModule.get_video_info,
      download_video: wasmModule.download_video,
    };

    document.dispatchEvent(new CustomEvent("wasm-loaded"));
    console.log("wasm module loaded successfully");

    checkUrlParam();
  } catch (error) {
    console.error("failed to load wasm module:", error);
  }
}

function checkUrlParam() {
  const urlParams = new URLSearchParams(window.location.search);
  const url = urlParams.get("url");

  if (url) {
    const urlInput = document.querySelector("url-input");
    if (urlInput) {
      const event = new CustomEvent("set-url", {
        detail: { url },
        bubbles: true,
      });
      urlInput.dispatchEvent(event);
    }
  }
}

setupWasm();
