use serde::Deserialize;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CanvasAspectRatio {
    Original,
    Portrait916,
    Square11,
    Landscape169,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CanvasBackgroundMode {
    Black,
    Blur,
}

pub struct CanvasFilter {
    pub filter: String,
    pub is_complex: bool,
}

impl CanvasAspectRatio {
    pub fn dimensions(self) -> Option<(u32, u32)> {
        match self {
            Self::Original => None,
            Self::Portrait916 => Some((1080, 1920)),
            Self::Square11 => Some((1080, 1080)),
            Self::Landscape169 => Some((1920, 1080)),
        }
    }
}

pub fn build_canvas_filter(
    aspect_ratio: CanvasAspectRatio,
    background_mode: CanvasBackgroundMode,
    video_filters: &[String],
) -> Option<CanvasFilter> {
    let (width, height) = aspect_ratio.dimensions()?;
    let foreground_filters = build_foreground_filters(video_filters, width, height);

    if background_mode == CanvasBackgroundMode::Blur {
        return Some(CanvasFilter {
            filter: format!(
                "[0:v]split=2[bgsrc][fgsrc];\
                 [bgsrc]scale={width}:{height}:force_original_aspect_ratio=increase,\
                 crop={width}:{height},gblur=sigma=24[bg];\
                 [fgsrc]{foreground_filters}[fg];\
                 [bg][fg]overlay=(W-w)/2:(H-h)/2,format=yuv420p[v]"
            ),
            is_complex: true,
        });
    }

    Some(CanvasFilter {
        filter: foreground_filters,
        is_complex: false,
    })
}

pub fn build_plain_video_filter(video_filters: &[String]) -> Option<CanvasFilter> {
    if video_filters.is_empty() {
        return None;
    }

    Some(CanvasFilter {
        filter: video_filters.join(","),
        is_complex: false,
    })
}

fn build_foreground_filters(video_filters: &[String], width: u32, height: u32) -> String {
    let mut filters = video_filters.to_vec();

    filters.push(format!(
        "scale={width}:{height}:force_original_aspect_ratio=decrease"
    ));
    filters.push(format!(
        "pad={width}:{height}:(ow-iw)/2:(oh-ih)/2:black"
    ));
    filters.push("format=yuv420p".to_string());

    filters.join(",")
}
