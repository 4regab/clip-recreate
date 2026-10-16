use clip_recreate::{ClipVideo, HEIGHT, WIDTH};
use fframes::{AudioMixOptions, CombinedMediaProvider, EncoderOptions, MediaDirectory, MediaProvider, RenderOptions, cli};
use fframes_skia_renderer::{
    SkiaFFramesRenderer, SkiaPipelineConcurrencyPolicy, SkiaPipelineConfig, vulkan::SkiaVulkanCtx,
};
use std::process::ExitCode;

fn main() -> ExitCode {
    let args = cli::parse::<cli::NoArgs>();
    // Plates, sprites and the stereo soundtrack load at runtime from assets/.
    let dir = MediaDirectory::read_folder(concat!(env!("CARGO_MANIFEST_DIR"), "/assets")).expect("assets dir");
    let assets = dir.process_media_source().expect("assets");
    let media = CombinedMediaProvider::from([&assets as &dyn MediaProvider]);
    let video = ClipVideo::new();
    let gpu = SkiaVulkanCtx::new(WIDTH, HEIGHT).expect("GPU context");

    cli::new(
        &video,
        RenderOptions {
            media: Some(&media),
            // Pass the original soundtrack through untouched: no limiter, no de-click fades.
            audio_mix: AudioMixOptions { master_gain_db: 0.0, limiter: None, declick: false },
            video_encoder_options: EncoderOptions {
                preferred_encoder: Some("libx264"),
                codec_params: Some(&[("crf", "14"), ("preset", "medium")]),
                ..Default::default()
            },
            ..Default::default()
        },
    )
    .args(args)
    .backend(
        SkiaFFramesRenderer::new_vulkan(
            &gpu,
            SkiaPipelineConfig { concurrency_policy: SkiaPipelineConcurrencyPolicy::MaxPerformance, ..Default::default() },
        )
        .expect("skia renderer"),
    )
    .preview(fframes_native_player::cli_preview)
    .run()
}
