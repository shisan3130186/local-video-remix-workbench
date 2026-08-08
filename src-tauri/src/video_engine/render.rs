use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RenderVideoResult {
    pub(crate) output_path: String,
    pub(crate) message: String,
    pub(crate) output_resolution: String,
    pub(crate) output_encoder: String,
    pub(crate) output_frame_rate: String,
    pub(crate) output_quality: String,
    pub(crate) output_video_bitrate_kbps: u32,
}
