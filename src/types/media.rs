//! Image generation and audio (speech / transcription) request types.
//!
//! These mirror the OpenAI-compatible `/v1/images/generations`,
//! `/v1/audio/speech`, and `/v1/audio/transcriptions` request shapes used by
//! the gateway provider's inherent media methods.

/// Parameters for an image generation request (`POST /v1/images/generations`).
///
/// Only `model` and `prompt` are required; the optional fields map directly to
/// the OpenAI-compatible image schema and are omitted from the body when unset.
#[derive(Debug, Clone, Default)]
pub struct ImageGenerationParams {
    /// Namespaced model identifier (e.g. `openai:dall-e-3`).
    pub model: String,

    /// Text prompt describing the desired image(s).
    pub prompt: String,

    /// Number of images to generate.
    pub n: Option<i32>,

    /// Rendering quality (e.g. `standard`, `hd`).
    pub quality: Option<String>,

    /// Response format (`url` or `b64_json`).
    pub response_format: Option<String>,

    /// Image size (e.g. `1024x1024`).
    pub size: Option<String>,

    /// Image style (e.g. `vivid`, `natural`).
    pub style: Option<String>,

    /// Optional end-user identifier for abuse monitoring.
    pub user: Option<String>,
}

impl ImageGenerationParams {
    /// Create a new image generation request for the given model and prompt.
    pub fn new(model: impl Into<String>, prompt: impl Into<String>) -> Self {
        Self {
            model: model.into(),
            prompt: prompt.into(),
            ..Default::default()
        }
    }

    /// Set the number of images to generate.
    #[must_use]
    pub fn with_n(mut self, n: i32) -> Self {
        self.n = Some(n);
        self
    }

    /// Set the rendering quality.
    #[must_use]
    pub fn with_quality(mut self, quality: impl Into<String>) -> Self {
        self.quality = Some(quality.into());
        self
    }

    /// Set the response format (`url` or `b64_json`).
    #[must_use]
    pub fn with_response_format(mut self, response_format: impl Into<String>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Set the image size (e.g. `1024x1024`).
    #[must_use]
    pub fn with_size(mut self, size: impl Into<String>) -> Self {
        self.size = Some(size.into());
        self
    }

    /// Set the image style.
    #[must_use]
    pub fn with_style(mut self, style: impl Into<String>) -> Self {
        self.style = Some(style.into());
        self
    }

    /// Attach an end-user identifier.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// Parameters for a text-to-speech request (`POST /v1/audio/speech`).
///
/// The gateway returns binary audio (no JSON model), so [`crate::Otari::speech`]
/// returns the raw bytes. Only `model`, `input`, and `voice` are required.
#[derive(Debug, Clone, Default)]
pub struct SpeechParams {
    /// Namespaced model identifier (e.g. `openai:tts-1`).
    pub model: String,

    /// Text to synthesize.
    pub input: String,

    /// Voice to use (e.g. `alloy`).
    pub voice: String,

    /// Output audio format (e.g. `mp3`, `wav`, `opus`).
    pub response_format: Option<String>,

    /// Playback speed multiplier.
    pub speed: Option<f64>,

    /// Voice/style instructions for models that support them.
    pub instructions: Option<String>,

    /// Optional end-user identifier for abuse monitoring.
    pub user: Option<String>,
}

impl SpeechParams {
    /// Create a new speech request for the given model, input, and voice.
    pub fn new(
        model: impl Into<String>,
        input: impl Into<String>,
        voice: impl Into<String>,
    ) -> Self {
        Self {
            model: model.into(),
            input: input.into(),
            voice: voice.into(),
            ..Default::default()
        }
    }

    /// Set the output audio format.
    #[must_use]
    pub fn with_response_format(mut self, response_format: impl Into<String>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Set the playback speed multiplier.
    #[must_use]
    pub fn with_speed(mut self, speed: f64) -> Self {
        self.speed = Some(speed);
        self
    }

    /// Set voice/style instructions.
    #[must_use]
    pub fn with_instructions(mut self, instructions: impl Into<String>) -> Self {
        self.instructions = Some(instructions.into());
        self
    }

    /// Attach an end-user identifier.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}

/// Parameters for an audio transcription request
/// (`POST /v1/audio/transcriptions`).
///
/// `file` holds the raw audio bytes, uploaded as the multipart `file` part;
/// `filename` is sent with that part so providers can infer the format from its
/// extension. The remaining fields map to the OpenAI-compatible transcription
/// schema and are sent as form fields when set.
#[derive(Debug, Clone)]
pub struct TranscriptionParams {
    /// Namespaced model identifier (e.g. `openai:whisper-1`).
    pub model: String,

    /// Raw audio bytes to transcribe.
    pub file: Vec<u8>,

    /// Filename for the multipart upload (providers may infer the audio format
    /// from its extension).
    pub filename: String,

    /// Source language hint (ISO-639-1).
    pub language: Option<String>,

    /// Optional prompt to guide the transcription style.
    pub prompt: Option<String>,

    /// Response format (`json`, `text`, `srt`, `verbose_json`, `vtt`).
    pub response_format: Option<String>,

    /// Sampling temperature.
    pub temperature: Option<f64>,

    /// Optional end-user identifier for abuse monitoring.
    pub user: Option<String>,
}

impl TranscriptionParams {
    /// Create a new transcription request for the given model and audio bytes.
    ///
    /// The multipart filename defaults to `audio`; override it with
    /// [`Self::with_filename`] so providers can infer the format from its
    /// extension (e.g. `clip.mp3`).
    pub fn new(model: impl Into<String>, file: impl Into<Vec<u8>>) -> Self {
        Self {
            model: model.into(),
            file: file.into(),
            filename: "audio".to_string(),
            language: None,
            prompt: None,
            response_format: None,
            temperature: None,
            user: None,
        }
    }

    /// Set the multipart upload filename.
    #[must_use]
    pub fn with_filename(mut self, filename: impl Into<String>) -> Self {
        self.filename = filename.into();
        self
    }

    /// Set the source language hint.
    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    /// Set the guiding prompt.
    #[must_use]
    pub fn with_prompt(mut self, prompt: impl Into<String>) -> Self {
        self.prompt = Some(prompt.into());
        self
    }

    /// Set the response format (`json`, `text`, `srt`, `verbose_json`, `vtt`).
    #[must_use]
    pub fn with_response_format(mut self, response_format: impl Into<String>) -> Self {
        self.response_format = Some(response_format.into());
        self
    }

    /// Set the sampling temperature.
    #[must_use]
    pub fn with_temperature(mut self, temperature: f64) -> Self {
        self.temperature = Some(temperature);
        self
    }

    /// Attach an end-user identifier.
    #[must_use]
    pub fn with_user(mut self, user: impl Into<String>) -> Self {
        self.user = Some(user.into());
        self
    }
}
