pub fn video_size(video: &web_sys::HtmlVideoElement) -> (u32, u32) {
    (video.video_width(), video.video_height())
}
