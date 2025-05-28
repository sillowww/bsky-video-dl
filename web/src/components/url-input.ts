class UrlInput extends HTMLElement {
  input: HTMLInputElement;
  button: HTMLButtonElement;
  errorMsg: HTMLDivElement;

  constructor() {
    super();

    const container = document.createElement("div");
    container.className = "url-input-container";

    this.input = document.createElement("input");
    this.input.type = "text";
    this.input.placeholder = "paste a bluesky post url here";
    this.input.addEventListener("input", () => this.validateInput());
    this.input.addEventListener("keydown", (e) => {
      if (e.key === "Enter" && !this.button.disabled) {
        this.checkUrl();
      }
    });

    this.button = document.createElement("button");
    this.button.textContent = "check";
    this.button.disabled = true;
    this.button.addEventListener("click", () => this.checkUrl());

    this.errorMsg = document.createElement("div");
    this.errorMsg.className = "url-error";

    container.appendChild(this.input);
    container.appendChild(this.button);

    this.appendChild(container);
    this.appendChild(this.errorMsg);

    this.addEventListener("set-url", (e: Event) => {
      const event = e as CustomEvent;
      if (event.detail?.url) {
        this.handleUrlParam(event.detail.url);
      }
    });
  }

  handleUrlParam(url: string) {
    this.input.value = url;
    if (this.validateInput()) this.checkUrl();
  }

  validateInput() {
    const url = this.input.value.trim();
    const isValid =
      url.startsWith("https://bsky.app/profile/") && url.includes("/post/");
    this.button.disabled = !isValid;
    this.errorMsg.textContent = "";
    return isValid;
  }

  async checkUrl() {
    if (!this.validateInput()) return;

    const url = this.input.value.trim();
    this.button.disabled = true;
    this.button.textContent = "checking...";

    try {
      const hasVideo = await window.bskyVideoDownloader.check_has_video(url);

      if (hasVideo) {
        const newUrl = new URL(window.location.href);
        newUrl.searchParams.set("url", url);
        window.history.replaceState({}, "", newUrl.toString());

        const event = new CustomEvent("url-validated", {
          detail: { url },
          bubbles: true,
          composed: true,
        });
        this.dispatchEvent(event);
      } else {
        this.errorMsg.textContent = "no video found in this post.";
      }
    } catch (error) {
      let errorMessage = "failed to check url.";
      console.error("error details:", error);

      if (error instanceof Error) {
        errorMessage = error.message;
      } else if (typeof error === "string") {
        errorMessage = error;
      } else if (error && typeof error === "object") {
        const errorObj = error as {
          message?: unknown;
          toString?: () => string;
        };
        if (typeof errorObj.message === "string" && errorObj.message) {
          errorMessage = errorObj.message;
        } else if (typeof errorObj.toString === "function") {
          errorMessage = errorObj.toString();
        }
      }

      if (
        errorMessage === "failed to check url." ||
        errorMessage === "Error" ||
        errorMessage.includes("unreachable")
      ) {
        errorMessage = "failed to check url. make sure it's a valid bsky post.";
      }

      this.errorMsg.textContent = errorMessage;
    } finally {
      this.button.disabled = false;
      this.button.textContent = "check";
    }
  }
}

customElements.define("url-input", UrlInput);
