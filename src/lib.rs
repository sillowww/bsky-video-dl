use js_sys::*;
use serde_json::Value as JsonValue;
use wasm_bindgen::prelude::*;

mod types;
use types::*;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);

    #[wasm_bindgen(js_namespace = console)]
    fn error(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

macro_rules! console_error {
    ($($t:tt)*) => (error(&format_args!($($t)*).to_string()))
}

async fn resolve_handle_to_did(handle: &str) -> Result<String, JsValue> {
    if handle.is_empty() {
        return Err(JsValue::from_str("handle cannot be empty"));
    }
    let client = create_http_client()?;
    let url = format!(
        "https://public.api.bsky.app/xrpc/app.bsky.actor.getProfile?actor={}",
        handle.trim()
    );
    console_log!("resolving handle via public.api.bsky.app: {}", handle);

    let response = client.get(&url).send().await;

    match response {
        Ok(resp) if resp.status().is_success() => {
            let json: serde_json::Value = resp.json().await.map_err(|e| {
                console_error!("failed to parse profile response: {}", e);
                JsValue::from_str(&format!(
                    "invalid response when resolving handle '{}'.",
                    handle
                ))
            })?;
            if let Some(did) = json.get("did").and_then(|v| v.as_str()) {
                console_log!("resolved DID: {}", did);
                Ok(did.to_string())
            } else {
                let msg = format!("profile response missing 'did' field for '{}'", handle);
                console_error!("{}", msg);
                Err(JsValue::from_str(&msg))
            }
        }
        Ok(resp) => {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            let msg = if let Ok(json) = serde_json::from_str::<serde_json::Value>(&text) {
                if let Some(message) = json.get("message").and_then(|m| m.as_str()) {
                    format!("unable to resolve handle (status {}): {}", status, message)
                } else {
                    format!("unable to resolve handle (status {}): {}", status, text)
                }
            } else {
                format!("unable to resolve handle (status {}): {}", status, text)
            };
            console_error!("{}", msg);
            Err(JsValue::from_str(&msg))
        }
        Err(e) => {
            let msg = format!("network error while resolving handle: {}", e);
            console_error!("{}", msg);
            Err(JsValue::from_str(&msg))
        }
    }
}

async fn resolve_did_to_pds(did: &str) -> Result<String, JsValue> {
    let url = if did.starts_with("did:web:") {
        let domain = &did[8..];
        let path = domain.replace(':', "/");
        format!("https://{}/.well-known/did.json", path)
    } else if did.starts_with("did:plc:") {
        format!("https://plc.directory/{}", did)
    } else {
        return Err(JsValue::from_str("unsupported DID method"));
    };

    let client = create_http_client()?;
    let response = client.get(&url).send().await.map_err(|e| {
        console_error!("failed to fetch DID doc: {}", e);
        JsValue::from_str(&format!("failed to fetch DID doc: {}", e))
    })?;
    if !response.status().is_success() {
        return Err(JsValue::from_str(&format!(
            "failed to fetch DID doc (status: {})",
            response.status().as_u16()
        )));
    }
    let doc: JsonValue = response.json().await.map_err(|e| {
        console_error!("invalid DID doc: {}", e);
        JsValue::from_str(&format!("invalid DID doc: {}", e))
    })?;
    let endpoint = doc["service"]
        .as_array()
        .and_then(|arr| arr.iter().find(|svc| svc["id"] == "#atproto_pds"))
        .and_then(|svc| svc["serviceEndpoint"].as_str())
        .ok_or_else(|| JsValue::from_str("no PDS endpoint found in DID doc"))?;
    Ok(endpoint.to_string())
}

fn create_http_client() -> Result<reqwest::Client, JsValue> {
    reqwest::Client::builder()
        .user_agent("bsky-video-dl/0.1.0")
        .build()
        .map_err(|e| JsValue::from_str(&format!("failed to create HTTP client: {}", e)))
}

#[wasm_bindgen]
pub async fn get_video_info(post_url: &str) -> Result<JsValue, JsValue> {
    console_log!("getting video info for URL: {}", post_url);

    let (did, rkey, pds_base) = resolve_post_url(post_url).await?;
    let record_url = format!(
        "{}/xrpc/com.atproto.repo.getRecord?repo={}&collection=app.bsky.feed.post&rkey={}",
        pds_base, did, rkey
    );

    let client = create_http_client()?;
    let response = client
        .get(&record_url)
        .send()
        .await
        .map_err(|e| JsValue::from_str(&format!("failed to fetch post: {}", e)))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let error_msg = match status {
            404 => "post not found. check that the URL is correct.".to_string(),
            400 => "invalid post URL.".to_string(),
            _ => format!("failed to fetch post (status: {}).", status),
        };
        return Err(JsValue::from_str(&error_msg));
    }

    let api_response: ApiResponse = response
        .json()
        .await
        .map_err(|_| JsValue::from_str("invalid post data received."))?;

    let embed = api_response
        .value
        .embed
        .ok_or_else(|| JsValue::from_str("no embed found"))?;

    if embed.embed_type != "app.bsky.embed.video" {
        return Err(JsValue::from_str("not a video post"));
    }

    let video = embed
        .video
        .ok_or_else(|| JsValue::from_str("no video data"))?;

    // create js object with video info
    let info = js_sys::Object::new();
    js_sys::Reflect::set(&info, &"cid".into(), &video.blob_ref.cid.into())?;
    js_sys::Reflect::set(&info, &"mimeType".into(), &video.mime_type.into())?;

    if let Some(size) = video.size {
        js_sys::Reflect::set(&info, &"size".into(), &(size as f64).into())?;
    }

    if let Some(aspect) = embed.aspect_ratio {
        let aspect_obj = js_sys::Object::new();
        js_sys::Reflect::set(&aspect_obj, &"width".into(), &(aspect.width as f64).into())?;
        js_sys::Reflect::set(
            &aspect_obj,
            &"height".into(),
            &(aspect.height as f64).into(),
        )?;
        js_sys::Reflect::set(&info, &"aspectRatio".into(), &aspect_obj)?;
    }

    Ok(info.into())
}

#[wasm_bindgen]
pub async fn check_has_video(post_url: &str) -> Result<bool, JsValue> {
    match get_video_info(post_url).await {
        Ok(_) => Ok(true),
        Err(e) => {
            let error_str = e.as_string().unwrap_or_default();
            if error_str.contains("no embed found")
                || error_str.contains("not a video post")
                || error_str.contains("no video data")
            {
                Ok(false)
            } else {
                Err(e)
            }
        }
    }
}

#[wasm_bindgen]
pub async fn download_video(post_url: &str) -> Result<Vec<u8>, JsValue> {
    console_log!("starting video download for URL: {}", post_url);

    let (did, rkey, pds_base) = resolve_post_url(post_url).await?;
    console_log!("parsed DID: {}, RKey: {}, PDS: {}", did, rkey, pds_base);

    let record_url = format!(
        "{}/xrpc/com.atproto.repo.getRecord?repo={}&collection=app.bsky.feed.post&rkey={}",
        pds_base, did, rkey
    );

    let client = create_http_client()?;

    console_log!("fetching post record...");
    let response = client.get(&record_url).send().await.map_err(|e| {
        console_error!("failed to fetch post record: {}", e);
        JsValue::from_str(&format!("failed to fetch post record: {}", e))
    })?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        console_error!("post record request failed with status: {}", status);
        return Err(JsValue::from_str(&format!(
            "failed to fetch post record. Status: {}",
            status
        )));
    }

    let api_response: ApiResponse = response.json().await.map_err(|e| {
        console_error!("failed to parse post record: {}", e);
        JsValue::from_str(&format!("failed to parse post record: {}", e))
    })?;

    console_log!("post record fetched successfully");

    // extract video information
    let embed = api_response
        .value
        .embed
        .ok_or_else(|| JsValue::from_str("post has no embed content"))?;

    if embed.embed_type != "app.bsky.embed.video" {
        return Err(JsValue::from_str(&format!(
            "post embed is not a video (type: {})",
            embed.embed_type
        )));
    }

    let video = embed
        .video
        .ok_or_else(|| JsValue::from_str("video embed has no video data"))?;

    console_log!("found video with CID: {}", video.blob_ref.cid);
    console_log!("video MIME type: {}", video.mime_type);

    if let Some(size) = video.size {
        console_log!("video size: {} bytes", size);
    }

    // download the blob
    let blob_url = format!(
        "{}/xrpc/com.atproto.sync.getBlob?did={}&cid={}",
        pds_base, did, video.blob_ref.cid
    );

    console_log!("downloading video blob...");
    let video_response = client.get(&blob_url).send().await.map_err(|e| {
        console_error!("failed to download video: {}", e);
        JsValue::from_str(&format!("failed to download video: {}", e))
    })?;

    if !video_response.status().is_success() {
        let status = video_response.status().as_u16();
        console_error!("video download failed with status: {}", status);
        return Err(JsValue::from_str(&format!(
            "failed to download video. Status: {}",
            status
        )));
    }

    let bytes = video_response.bytes().await.map_err(|e| {
        console_error!("failed to read video bytes: {}", e);
        JsValue::from_str(&format!("failed to read video bytes: {}", e))
    })?;

    console_log!("video downloaded successfully. size: {} bytes", bytes.len());
    Ok(bytes.to_vec())
}

fn parse_bsky_url(url: &str) -> Result<(String, String), JsValue> {
    let url = url.trim();

    if url.is_empty() {
        return Err(JsValue::from_str("URL cannot be empty"));
    }

    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(JsValue::from_str("URL must start with http:// or https://"));
    }

    if !url.contains("/profile/") || !url.contains("/post/") {
        return Err(JsValue::from_str(
            "invalid URL format. expected format: https://{domain}/profile/{handle}/post/{rkey}",
        ));
    }

    let parts: Vec<&str> = url.split('/').collect();

    let profile_idx = parts
        .iter()
        .position(|&x| x == "profile")
        .ok_or_else(|| JsValue::from_str("no 'profile' segment found in URL"))?;

    let post_idx = parts
        .iter()
        .position(|&x| x == "post")
        .ok_or_else(|| JsValue::from_str("no 'post' segment found in URL"))?;

    if profile_idx + 1 >= parts.len() {
        return Err(JsValue::from_str(
            "no handle/DID found after 'profile' in URL",
        ));
    }

    if post_idx + 1 >= parts.len() {
        return Err(JsValue::from_str("no post ID found after 'post' in URL"));
    }

    if post_idx <= profile_idx {
        return Err(JsValue::from_str(
            "invalid URL structure: 'post' must come after 'profile'",
        ));
    }

    let handle_or_did = parts[profile_idx + 1];
    let rkey = parts[post_idx + 1];

    if handle_or_did.is_empty() {
        return Err(JsValue::from_str("handle/DID cannot be empty"));
    }

    if rkey.is_empty() {
        return Err(JsValue::from_str("post ID cannot be empty"));
    }

    Ok((handle_or_did.to_string(), rkey.to_string()))
}

async fn resolve_post_url(url: &str) -> Result<(String, String, String), JsValue> {
    let (handle_or_did, rkey) = parse_bsky_url(url)?;
    let did = if handle_or_did.starts_with("did:") {
        console_log!("using provided DID: {}", handle_or_did);
        handle_or_did
    } else {
        console_log!("resolving handle to DID: {}", handle_or_did);
        resolve_handle_to_did(&handle_or_did).await?
    };
    let pds = resolve_did_to_pds(&did).await?;
    Ok((did, rkey, pds))
}

#[wasm_bindgen]
pub fn parse_url(url: &str) -> Result<js_sys::Array, JsValue> {
    let (handle_or_did, rkey) = parse_bsky_url(url)?;
    let result = js_sys::Array::new();
    result.push(&JsValue::from_str(&handle_or_did));
    result.push(&JsValue::from_str(&rkey));
    Ok(result)
}
