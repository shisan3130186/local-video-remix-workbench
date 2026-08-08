use crate::video_engine::watermark::WatermarkPosition;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct WatermarkRemovalSettings {
    pub enabled: bool,
    #[serde(default)]
    pub region_count: u32,
    #[serde(default)]
    pub manual_regions: Vec<WatermarkRemovalRegion>,
    pub mode: WatermarkRemovalMode,
    pub position: WatermarkPosition,
    pub size: WatermarkRemovalSize,
    pub margin: u32,
    pub strength: u32,
    pub cover_color: String,
    pub cover_opacity: f64,
    pub tracking_enabled: bool,
    pub tracking_region_width_ratio: f64,
    pub tracking_region_height_ratio: f64,
    pub tracking_keyframes: Vec<WatermarkTrackingKeyframe>,
}

impl Default for WatermarkRemovalSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            region_count: 1,
            manual_regions: Vec::new(),
            mode: WatermarkRemovalMode::Delogo,
            position: WatermarkPosition::TopRight,
            size: WatermarkRemovalSize::Medium,
            margin: 16,
            strength: 12,
            cover_color: "#000000".to_string(),
            cover_opacity: 0.85,
            tracking_enabled: false,
            tracking_region_width_ratio: 0.28,
            tracking_region_height_ratio: 0.12,
            tracking_keyframes: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkRemovalRegion {
    pub x_ratio: f64,
    pub y_ratio: f64,
    pub width_ratio: f64,
    pub height_ratio: f64,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WatermarkTrackingKeyframe {
    pub time_seconds: f64,
    pub x_ratio: f64,
    pub y_ratio: f64,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WatermarkRemovalMode {
    Crop,
    Delogo,
    Blur,
    Mosaic,
    Cover,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum WatermarkRemovalSize {
    Small,
    Medium,
    Large,
}

pub fn normalize_watermark_removal_settings(
    mut settings: WatermarkRemovalSettings,
) -> Result<Option<WatermarkRemovalSettings>, String> {
    if !settings.enabled {
        return Ok(None);
    }

    if settings.margin > 160 {
        return Err("原水印处理边距不能超过 160。".to_string());
    }

    if !(1..=8).contains(&settings.region_count) {
        return Err("去除水印区域数必须在 1 到 8 之间。".to_string());
    }
    if !settings.manual_regions.is_empty() {
        if settings.manual_regions.len() != settings.region_count as usize {
            return Err("去除水印区域数量与画面框数量不一致。".to_string());
        }
        for region in &settings.manual_regions {
            if !region.x_ratio.is_finite()
                || !region.y_ratio.is_finite()
                || !region.width_ratio.is_finite()
                || !region.height_ratio.is_finite()
                || region.x_ratio < 0.0
                || region.y_ratio < 0.0
                || region.width_ratio < 0.04
                || region.height_ratio < 0.04
                || region.x_ratio + region.width_ratio > 1.0001
                || region.y_ratio + region.height_ratio > 1.0001
            {
                return Err("去除水印区域不能超出视频画面。".to_string());
            }
        }
    }

    if settings.mode == WatermarkRemovalMode::Crop && settings.position == WatermarkPosition::Center
    {
        return Err("裁剪去水印只适用于画面顶部或底部，请选择四个边角之一。".to_string());
    }

    if matches!(
        settings.mode,
        WatermarkRemovalMode::Blur | WatermarkRemovalMode::Mosaic
    ) && !(4..=24).contains(&settings.strength)
    {
        return Err("模糊或马赛克强度必须在 4 到 24 之间。".to_string());
    }

    if settings.mode == WatermarkRemovalMode::Cover {
        if !is_hex_color(&settings.cover_color) {
            return Err("遮盖颜色必须是有效的六位十六进制颜色。".to_string());
        }
        if !settings.cover_opacity.is_finite()
            || settings.cover_opacity < 0.1
            || settings.cover_opacity > 1.0
        {
            return Err("遮盖透明度必须在 0.1 到 1 之间。".to_string());
        }
    }

    if settings.tracking_enabled {
        if !matches!(
            settings.mode,
            WatermarkRemovalMode::Blur | WatermarkRemovalMode::Mosaic | WatermarkRemovalMode::Cover
        ) {
            return Err("移动水印跟踪只支持区域模糊、马赛克或色块遮盖。".to_string());
        }
        if !is_valid_tracking_size(settings.tracking_region_width_ratio)
            || !is_valid_tracking_size(settings.tracking_region_height_ratio)
        {
            return Err("移动水印框的宽度和高度必须在画面的 0.04 到 0.8 之间。".to_string());
        }
        if !(2..=8).contains(&settings.tracking_keyframes.len()) {
            return Err("移动水印至少需要 2 个、最多支持 8 个关键位置。".to_string());
        }
        settings.tracking_keyframes.sort_by(|left, right| {
            left.time_seconds
                .partial_cmp(&right.time_seconds)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        for (index, keyframe) in settings.tracking_keyframes.iter().enumerate() {
            if !keyframe.time_seconds.is_finite()
                || keyframe.time_seconds < 0.0
                || !keyframe.x_ratio.is_finite()
                || keyframe.x_ratio < 0.0
                || !keyframe.y_ratio.is_finite()
                || keyframe.y_ratio < 0.0
                || keyframe.x_ratio + settings.tracking_region_width_ratio > 1.000_001
                || keyframe.y_ratio + settings.tracking_region_height_ratio > 1.000_001
            {
                return Err(format!("第 {} 个移动水印关键位置超出画面。", index + 1));
            }
            if index > 0
                && keyframe.time_seconds - settings.tracking_keyframes[index - 1].time_seconds
                    < 0.05
            {
                return Err("移动水印关键位置的时间必须依次递增，且至少间隔 0.05 秒。".to_string());
            }
        }
    }

    Ok(Some(settings))
}

pub fn validate_tracking_duration(
    settings: Option<&WatermarkRemovalSettings>,
    duration_seconds: Option<f64>,
) -> Result<(), String> {
    let Some(settings) = settings.filter(|settings| settings.tracking_enabled) else {
        return Ok(());
    };
    let Some(duration_seconds) = duration_seconds.filter(|duration| duration.is_finite()) else {
        return Ok(());
    };
    if settings
        .tracking_keyframes
        .last()
        .is_some_and(|keyframe| keyframe.time_seconds > duration_seconds + 0.05)
    {
        return Err("移动水印关键位置超过了当前视频时长，请删除超出时间的关键位置。".to_string());
    }
    Ok(())
}

pub fn build_watermark_removal_layer(
    input_label: &str,
    output_label: &str,
    settings: &WatermarkRemovalSettings,
    layer_number: usize,
    frame_dimensions: (u32, u32),
) -> String {
    if settings.tracking_enabled {
        return build_tracking_removal_layer(
            input_label,
            output_label,
            settings,
            layer_number,
            frame_dimensions,
        );
    }
    if !settings.manual_regions.is_empty() {
        let mut current_label = input_label.to_string();
        let mut filter = String::new();
        for (index, manual_region) in settings.manual_regions.iter().enumerate() {
            let region = RemovalRegion::from_manual(manual_region, frame_dimensions);
            let next_label = if index + 1 == settings.manual_regions.len() {
                output_label.to_string()
            } else {
                format!("[remmanual{layer_number}_{index}]")
            };
            filter.push_str(&format!(
                "{current_label}delogo=x={x}:y={y}:w={width}:h={height}:show=0{next_label}",
                x = region.x,
                y = region.y,
                width = region.width,
                height = region.height,
            ));
            current_label = next_label;
        }
        return filter;
    }
    let region = RemovalRegion::new(settings, frame_dimensions);
    match settings.mode {
        WatermarkRemovalMode::Crop => {
            build_crop_removal_layer(input_label, output_label, settings, &region, frame_dimensions)
        }
        WatermarkRemovalMode::Delogo => format!(
            "{input_label}delogo=x={x}:y={y}:w={width}:h={height}:show=0{output_label}",
            x = region.x,
            y = region.y,
            width = region.width,
            height = region.height,
        ),
        WatermarkRemovalMode::Blur => format!(
            "{input_label}split=2[rembase{layer_number}][remsource{layer_number}];\
             [remsource{layer_number}]crop=w={width}:h={height}:x={x}:y={y},gblur=sigma={strength:.3}[rempatch{layer_number}];\
             [rembase{layer_number}][rempatch{layer_number}]overlay=x={x}:y={y}:eof_action=pass{output_label}",
            width = region.width,
            height = region.height,
            x = region.x,
            y = region.y,
            strength = f64::from(settings.strength) / 2.0,
        ),
        WatermarkRemovalMode::Mosaic => format!(
            "{input_label}split=2[rembase{layer_number}][remsource{layer_number}];\
             [remsource{layer_number}]crop=w={width}:h={height}:x={x}:y={y},scale=w=iw/{strength}:h=ih/{strength}:flags=area,scale=w=iw*{strength}:h=ih*{strength}:flags=neighbor[rempatch{layer_number}];\
             [rembase{layer_number}][rempatch{layer_number}]overlay=x={x}:y={y}:eof_action=pass{output_label}",
            width = region.width,
            height = region.height,
            x = region.x,
            y = region.y,
            strength = settings.strength,
        ),
        WatermarkRemovalMode::Cover => {
            let color = settings
                .cover_color
                .trim_start_matches('#')
                .to_ascii_uppercase();
            format!(
                "{input_label}drawbox=x={x}:y={y}:w={width}:h={height}:color=0x{color}@{opacity:.3}:t=fill{output_label}",
                x = region.x,
                y = region.y,
                width = region.width,
                height = region.height,
                opacity = settings.cover_opacity,
            )
        }
    }
}

fn build_tracking_removal_layer(
    input_label: &str,
    output_label: &str,
    settings: &WatermarkRemovalSettings,
    layer_number: usize,
    frame_dimensions: (u32, u32),
) -> String {
    let (frame_width, frame_height) = frame_dimensions;
    let mut region_width = ratio_dimension(frame_width, settings.tracking_region_width_ratio);
    let mut region_height = ratio_dimension(frame_height, settings.tracking_region_height_ratio);
    if settings.mode == WatermarkRemovalMode::Mosaic {
        region_width = align_dimension(region_width, settings.strength, frame_width);
        region_height = align_dimension(region_height, settings.strength, frame_height);
    }
    let x_expression = build_tracking_expression(
        &settings.tracking_keyframes,
        frame_width,
        region_width,
        |keyframe| keyframe.x_ratio,
    );
    let y_expression = build_tracking_expression(
        &settings.tracking_keyframes,
        frame_height,
        region_height,
        |keyframe| keyframe.y_ratio,
    );

    match settings.mode {
        WatermarkRemovalMode::Blur => format!(
            "{input_label}split=2[rembase{layer_number}][remsource{layer_number}];\
             [remsource{layer_number}]crop=w={region_width}:h={region_height}:x='{x_expression}':y='{y_expression}',gblur=sigma={strength:.3}[rempatch{layer_number}];\
             [rembase{layer_number}][rempatch{layer_number}]overlay=x='{x_expression}':y='{y_expression}':eval=frame:eof_action=pass{output_label}",
            strength = f64::from(settings.strength) / 2.0,
        ),
        WatermarkRemovalMode::Mosaic => format!(
            "{input_label}split=2[rembase{layer_number}][remsource{layer_number}];\
             [remsource{layer_number}]crop=w={region_width}:h={region_height}:x='{x_expression}':y='{y_expression}',scale=w=iw/{strength}:h=ih/{strength}:flags=area,scale=w=iw*{strength}:h=ih*{strength}:flags=neighbor[rempatch{layer_number}];\
             [rembase{layer_number}][rempatch{layer_number}]overlay=x='{x_expression}':y='{y_expression}':eval=frame:eof_action=pass{output_label}",
            strength = settings.strength,
        ),
        WatermarkRemovalMode::Cover => {
            let color = settings
                .cover_color
                .trim_start_matches('#')
                .to_ascii_uppercase();
            format!(
                "color=c=0x{color}@{opacity:.3}:s={region_width}x{region_height},format=rgba[rempatch{layer_number}];\
                 {input_label}[rempatch{layer_number}]overlay=x='{x_expression}':y='{y_expression}':eval=frame:shortest=1{output_label}",
                opacity = settings.cover_opacity,
            )
        }
        WatermarkRemovalMode::Crop | WatermarkRemovalMode::Delogo => {
            unreachable!("移动水印设置已在参数校验阶段限制处理方式")
        }
    }
}

fn build_tracking_expression(
    keyframes: &[WatermarkTrackingKeyframe],
    frame_dimension: u32,
    region_dimension: u32,
    ratio: impl Fn(&WatermarkTrackingKeyframe) -> f64,
) -> String {
    let maximum_origin = f64::from(frame_dimension.saturating_sub(region_dimension));
    let pixel_positions = keyframes
        .iter()
        .map(|keyframe| (ratio(keyframe) * f64::from(frame_dimension)).clamp(0.0, maximum_origin))
        .collect::<Vec<_>>();
    let last_position = pixel_positions.last().copied().unwrap_or(0.0);
    let mut expression = format!("{last_position:.3}");

    for index in (0..keyframes.len().saturating_sub(1)).rev() {
        let start = &keyframes[index];
        let end = &keyframes[index + 1];
        let start_position = pixel_positions[index];
        let end_position = pixel_positions[index + 1];
        let duration = end.time_seconds - start.time_seconds;
        let interpolated = format!(
            "{start_position:.3}+({delta:.3})*(t-{start_time:.3})/{duration:.3}",
            delta = end_position - start_position,
            start_time = start.time_seconds,
        );
        expression = format!(
            "if(lt(t\\,{end_time:.3})\\,{interpolated}\\,{expression})",
            end_time = end.time_seconds,
        );
    }

    let first = &keyframes[0];
    let first_position = pixel_positions[0];
    format!(
        "if(lt(t\\,{first_time:.3})\\,{first_position:.3}\\,{expression})",
        first_time = first.time_seconds,
    )
}

fn build_crop_removal_layer(
    input_label: &str,
    output_label: &str,
    settings: &WatermarkRemovalSettings,
    region: &RemovalRegion,
    frame_dimensions: (u32, u32),
) -> String {
    let (frame_width, frame_height) = frame_dimensions;
    let maximum_crop_height = frame_height.saturating_sub(2);
    let crop_height = if maximum_crop_height < 2 {
        maximum_crop_height
    } else {
        region
            .height
            .saturating_add(settings.margin)
            .clamp(2, maximum_crop_height)
    };
    let remaining_height = frame_height.saturating_sub(crop_height);
    let crop_y = match settings.position {
        WatermarkPosition::TopLeft | WatermarkPosition::TopRight => crop_height,
        WatermarkPosition::BottomLeft | WatermarkPosition::BottomRight => 0,
        WatermarkPosition::Center => 0,
    };

    format!(
        "{input_label}crop=w={frame_width}:h={remaining_height}:x=0:y={crop_y},\
         scale=w={frame_width}:h={frame_height}:force_original_aspect_ratio=increase:force_divisible_by=2,\
         crop=w={frame_width}:h={frame_height}{output_label}"
    )
}

struct RemovalRegion {
    width: u32,
    height: u32,
    x: u32,
    y: u32,
}

impl RemovalRegion {
    fn from_manual(region: &WatermarkRemovalRegion, frame_dimensions: (u32, u32)) -> Self {
        let (frame_width, frame_height) = frame_dimensions;
        Self {
            x: (f64::from(frame_width) * region.x_ratio).round() as u32,
            y: (f64::from(frame_height) * region.y_ratio).round() as u32,
            width: ratio_dimension(frame_width, region.width_ratio),
            height: ratio_dimension(frame_height, region.height_ratio),
        }
    }

    fn new(settings: &WatermarkRemovalSettings, frame_dimensions: (u32, u32)) -> Self {
        let (width_ratio, height_ratio) = match settings.size {
            WatermarkRemovalSize::Small => (0.18, 0.10),
            WatermarkRemovalSize::Medium => (0.28, 0.16),
            WatermarkRemovalSize::Large => (0.40, 0.24),
        };
        let (frame_width, frame_height) = frame_dimensions;
        let mut width = ratio_dimension(frame_width, width_ratio);
        let mut height = ratio_dimension(frame_height, height_ratio);
        if settings.mode == WatermarkRemovalMode::Mosaic {
            width = align_dimension(width, settings.strength, frame_width);
            height = align_dimension(height, settings.strength, frame_height);
        }
        let (x, y) = region_position(
            settings.position,
            settings.margin,
            frame_dimensions,
            (width, height),
            if settings.mode == WatermarkRemovalMode::Delogo {
                1
            } else {
                0
            },
        );

        Self {
            width,
            height,
            x,
            y,
        }
    }
}

fn ratio_dimension(frame_dimension: u32, ratio: f64) -> u32 {
    let even_frame_dimension = frame_dimension.saturating_sub(frame_dimension % 2).max(2);
    let dimension = (f64::from(frame_dimension) * ratio).floor() as u32;
    dimension
        .saturating_sub(dimension % 2)
        .max(2)
        .min(even_frame_dimension)
}

fn align_dimension(dimension: u32, strength: u32, frame_dimension: u32) -> u32 {
    let aligned = (dimension / strength).max(1) * strength;
    aligned.min(frame_dimension).max(2)
}

fn region_position(
    position: WatermarkPosition,
    margin: u32,
    frame_dimensions: (u32, u32),
    region_dimensions: (u32, u32),
    minimum_inset: u32,
) -> (u32, u32) {
    let available_x = frame_dimensions.0.saturating_sub(region_dimensions.0);
    let available_y = frame_dimensions.1.saturating_sub(region_dimensions.1);
    let horizontal_margin = margin
        .max(minimum_inset)
        .min(available_x.saturating_sub(minimum_inset));
    let vertical_margin = margin
        .max(minimum_inset)
        .min(available_y.saturating_sub(minimum_inset));
    match position {
        WatermarkPosition::TopLeft => (horizontal_margin, vertical_margin),
        WatermarkPosition::TopRight => (
            available_x.saturating_sub(horizontal_margin),
            vertical_margin,
        ),
        WatermarkPosition::BottomLeft => (
            horizontal_margin,
            available_y.saturating_sub(vertical_margin),
        ),
        WatermarkPosition::BottomRight => (
            available_x.saturating_sub(horizontal_margin),
            available_y.saturating_sub(vertical_margin),
        ),
        WatermarkPosition::Center => (available_x / 2, available_y / 2),
    }
}

fn is_hex_color(value: &str) -> bool {
    value.len() == 7
        && value.starts_with('#')
        && value
            .chars()
            .skip(1)
            .all(|character| character.is_ascii_hexdigit())
}

fn is_valid_tracking_size(value: f64) -> bool {
    value.is_finite() && (0.04..=0.8).contains(&value)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn enabled_settings(mode: WatermarkRemovalMode) -> WatermarkRemovalSettings {
        WatermarkRemovalSettings {
            enabled: true,
            mode,
            ..WatermarkRemovalSettings::default()
        }
    }

    fn tracking_settings(mode: WatermarkRemovalMode) -> WatermarkRemovalSettings {
        WatermarkRemovalSettings {
            enabled: true,
            mode,
            tracking_enabled: true,
            tracking_region_width_ratio: 0.2,
            tracking_region_height_ratio: 0.1,
            tracking_keyframes: vec![
                WatermarkTrackingKeyframe {
                    time_seconds: 0.0,
                    x_ratio: 0.1,
                    y_ratio: 0.2,
                },
                WatermarkTrackingKeyframe {
                    time_seconds: 2.0,
                    x_ratio: 0.6,
                    y_ratio: 0.7,
                },
            ],
            ..WatermarkRemovalSettings::default()
        }
    }

    #[test]
    fn disabled_removal_uses_no_filter() {
        assert!(
            normalize_watermark_removal_settings(WatermarkRemovalSettings::default())
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn missing_tracking_fields_use_safe_defaults() {
        let settings: WatermarkRemovalSettings = serde_json::from_str("{}").unwrap();
        assert!(!settings.tracking_enabled);
        assert_eq!(settings.tracking_region_width_ratio, 0.28);
        assert_eq!(settings.tracking_region_height_ratio, 0.12);
        assert!(settings.tracking_keyframes.is_empty());
    }

    #[test]
    fn validates_tracking_mode_keyframes_and_duration() {
        let unsupported = tracking_settings(WatermarkRemovalMode::Delogo);
        assert_eq!(
            normalize_watermark_removal_settings(unsupported).unwrap_err(),
            "移动水印跟踪只支持区域模糊、马赛克或色块遮盖。"
        );

        let mut missing = tracking_settings(WatermarkRemovalMode::Mosaic);
        missing.tracking_keyframes.pop();
        assert_eq!(
            normalize_watermark_removal_settings(missing).unwrap_err(),
            "移动水印至少需要 2 个、最多支持 8 个关键位置。"
        );

        let normalized =
            normalize_watermark_removal_settings(tracking_settings(WatermarkRemovalMode::Blur))
                .unwrap();
        assert_eq!(
            validate_tracking_duration(normalized.as_ref(), Some(1.5)).unwrap_err(),
            "移动水印关键位置超过了当前视频时长，请删除超出时间的关键位置。"
        );
        validate_tracking_duration(normalized.as_ref(), Some(2.0)).unwrap();
    }

    #[test]
    fn builds_dynamic_tracking_filters_for_supported_modes() {
        let blur = build_watermark_removal_layer(
            "[in]",
            "[out]",
            &tracking_settings(WatermarkRemovalMode::Blur),
            4,
            (640, 360),
        );
        assert!(blur.contains("crop=w=128:h=36"));
        assert!(blur.contains("if(lt(t\\,2.000)"));
        assert!(blur.contains("overlay=x='"));
        assert!(blur.contains(":eval=frame:eof_action=pass[out]"));

        let mosaic = build_watermark_removal_layer(
            "[in]",
            "[out]",
            &tracking_settings(WatermarkRemovalMode::Mosaic),
            5,
            (640, 360),
        );
        assert!(mosaic.contains("scale=w=iw/12:h=ih/12:flags=area"));
        assert!(mosaic.contains(":eval=frame:eof_action=pass[out]"));

        let cover = build_watermark_removal_layer(
            "[in]",
            "[out]",
            &tracking_settings(WatermarkRemovalMode::Cover),
            6,
            (640, 360),
        );
        assert!(cover.contains("color=c=0x000000@0.850:s=128x36,format=rgba[rempatch6]"));
        assert!(cover.contains("overlay=x='"));
        assert!(cover.contains(":eval=frame:shortest=1[out]"));
    }

    #[test]
    fn validates_blur_strength_and_cover_color() {
        let mut blur = enabled_settings(WatermarkRemovalMode::Blur);
        blur.strength = 3;
        assert_eq!(
            normalize_watermark_removal_settings(blur).unwrap_err(),
            "模糊或马赛克强度必须在 4 到 24 之间。"
        );

        let mut cover = enabled_settings(WatermarkRemovalMode::Cover);
        cover.cover_color = "black".to_string();
        assert_eq!(
            normalize_watermark_removal_settings(cover).unwrap_err(),
            "遮盖颜色必须是有效的六位十六进制颜色。"
        );
    }

    #[test]
    fn rejects_center_position_for_crop_removal() {
        let settings = WatermarkRemovalSettings {
            enabled: true,
            mode: WatermarkRemovalMode::Crop,
            position: WatermarkPosition::Center,
            ..WatermarkRemovalSettings::default()
        };
        assert_eq!(
            normalize_watermark_removal_settings(settings).unwrap_err(),
            "裁剪去水印只适用于画面顶部或底部，请选择四个边角之一。"
        );
    }

    #[test]
    fn builds_top_and_bottom_crop_then_restores_frame_size() {
        let top = WatermarkRemovalSettings {
            enabled: true,
            mode: WatermarkRemovalMode::Crop,
            position: WatermarkPosition::TopRight,
            ..WatermarkRemovalSettings::default()
        };
        let top_filter = build_watermark_removal_layer("[in]", "[out]", &top, 1, (1920, 1080));
        assert!(top_filter.contains("crop=w=1920:h=892:x=0:y=188"));
        assert!(top_filter.contains(
            "scale=w=1920:h=1080:force_original_aspect_ratio=increase:force_divisible_by=2"
        ));
        assert!(top_filter.ends_with("crop=w=1920:h=1080[out]"));

        let bottom = WatermarkRemovalSettings {
            position: WatermarkPosition::BottomLeft,
            ..top
        };
        let bottom_filter =
            build_watermark_removal_layer("[in]", "[out]", &bottom, 1, (1920, 1080));
        assert!(bottom_filter.contains("crop=w=1920:h=892:x=0:y=0"));
    }

    #[test]
    fn builds_delogo_for_bottom_right_region() {
        let settings = WatermarkRemovalSettings {
            enabled: true,
            position: WatermarkPosition::BottomRight,
            ..WatermarkRemovalSettings::default()
        };
        let filter = build_watermark_removal_layer("[in]", "[out]", &settings, 1, (1920, 1080));
        assert!(filter.starts_with("[in]delogo="));
        assert!(filter.contains("x=1368:y=892:w=536:h=172"));
        assert!(filter.ends_with("show=0[out]"));
    }

    #[test]
    fn builds_blur_and_mosaic_patch_layers() {
        let blur = build_watermark_removal_layer(
            "[in]",
            "[out]",
            &enabled_settings(WatermarkRemovalMode::Blur),
            2,
            (1920, 1080),
        );
        assert!(blur.contains("split=2[rembase2][remsource2]"));
        assert!(blur.contains("gblur=sigma=6.000"));
        assert!(blur.contains("overlay=x=1368:y=16"));

        let mosaic = build_watermark_removal_layer(
            "[in]",
            "[out]",
            &enabled_settings(WatermarkRemovalMode::Mosaic),
            3,
            (1920, 1080),
        );
        assert!(mosaic.contains("crop=w=528:h=168"));
        assert!(mosaic.contains("scale=w=iw/12:h=ih/12:flags=area"));
        assert!(mosaic.contains("scale=w=iw*12:h=ih*12:flags=neighbor"));
    }

    #[test]
    fn builds_cover_with_color_and_opacity() {
        let settings = WatermarkRemovalSettings {
            enabled: true,
            mode: WatermarkRemovalMode::Cover,
            cover_color: "#12abef".to_string(),
            cover_opacity: 0.65,
            ..WatermarkRemovalSettings::default()
        };
        let filter = build_watermark_removal_layer("[in]", "[out]", &settings, 1, (1920, 1080));
        assert!(filter.contains("drawbox="));
        assert!(filter.contains("color=0x12ABEF@0.650:t=fill"));
    }

    #[test]
    fn keeps_region_inside_small_frames() {
        let settings = WatermarkRemovalSettings {
            enabled: true,
            margin: 160,
            position: WatermarkPosition::BottomRight,
            ..WatermarkRemovalSettings::default()
        };
        let region = RemovalRegion::new(&settings, (320, 180));
        assert!(region.x + region.width <= 320);
        assert!(region.y + region.height <= 180);
    }

    #[test]
    fn keeps_delogo_one_pixel_inside_when_margin_is_zero() {
        let settings = WatermarkRemovalSettings {
            enabled: true,
            margin: 0,
            position: WatermarkPosition::BottomRight,
            ..WatermarkRemovalSettings::default()
        };
        let region = RemovalRegion::new(&settings, (320, 180));
        assert_eq!(region.x, 231);
        assert_eq!(region.y, 151);
        assert_eq!(region.x + region.width, 319);
        assert_eq!(region.y + region.height, 179);
    }
}
