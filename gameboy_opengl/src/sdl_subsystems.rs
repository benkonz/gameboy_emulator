pub struct SdlSubsystems {
    pub audio_subsystem: sdl2::AudioSubsystem,
    pub video_subsystem: sdl2::VideoSubsystem,
    pub event_pump: sdl2::EventPump,
}

unsafe impl Send for SdlSubsystems {}