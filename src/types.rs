use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    pub value: PostRecord,
}

#[derive(Deserialize, Debug)]
pub struct PostRecord {
    pub embed: Option<Embed>,
}

#[derive(Deserialize, Debug)]
pub struct Embed {
    #[serde(rename = "$type")]
    pub embed_type: String,
    pub video: Option<VideoBlob>,
    #[serde(rename = "aspectRatio")]
    pub aspect_ratio: Option<AspectRatio>,
}

#[derive(Deserialize, Debug)]
pub struct AspectRatio {
    pub width: u32,
    pub height: u32,
}

#[derive(Deserialize, Debug)]
pub struct VideoBlob {
    #[serde(rename = "ref")]
    pub blob_ref: BlobRef,
    #[serde(rename = "mimeType")]
    pub mime_type: String,
    pub size: Option<u64>,
}

#[derive(Deserialize, Debug)]
pub struct BlobRef {
    #[serde(rename = "$link")]
    pub cid: String,
}

#[derive(Deserialize, Debug)]
pub struct ResolveResponse {
    pub did: String,
}
