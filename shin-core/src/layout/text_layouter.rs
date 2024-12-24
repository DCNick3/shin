pub trait TextLayouter {
    fn on_message_start(&mut self);
    fn on_message_end(&mut self);
    fn on_char(&mut self, codepoint: char);
    fn on_newline(&mut self);
    fn on_click_wait(&mut self);
    fn on_auto_click(&mut self);
    fn on_set_font_scale(&mut self, scale: i32);
    fn on_set_color(&mut self, color: i32);
    fn on_set_draw_speed(&mut self, speed: i32);
    fn on_set_fade(&mut self, fade: i32);
    fn on_wait(&mut self, delay: i32);
    fn on_start_parallel(&mut self);
    fn on_section(&mut self);
    fn on_sync(&mut self);
    fn on_instant_start(&mut self);
    fn on_instant_end(&mut self);
    fn on_lipsync_enabled(&mut self);
    fn on_lipsync_disabled(&mut self);
    fn on_set_voice_volume(&mut self, volume: i32);
    fn on_voice(&mut self, voice_path: String);
    fn on_voice_sync(&mut self, target_instant: i32);
    fn on_voice_wait(&mut self);
    fn on_rubi_content(&mut self, content: String);
    fn on_rubi_base_start(&mut self);
    fn on_rubi_base_end(&mut self);
    fn on_bold_start(&mut self);
    fn on_bold_end(&mut self);
}
