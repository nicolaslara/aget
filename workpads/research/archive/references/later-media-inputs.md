## Later Media Inputs

| Topic | URL | Notes |
| --- | --- | --- |
| youtube-transcript-api | https://github.com/jdepoix/youtube-transcript-api | MIT. Candidate for fastest first-pass YouTube manual/auto caption retrieval without API key or browser. Need policy/reliability review. |
| yt-dlp manual | https://man.archlinux.org/man/extra/yt-dlp/yt-dlp.1.en | Candidate CLI for listing/downloading subtitles and downloading audio for local ASR fallback. Need license and platform-policy review. |
| faster-whisper | https://github.com/SYSTRAN/faster-whisper | MIT. Candidate Python local ASR backend using CTranslate2; good for product/backend iteration speed. |
| whisper.cpp | https://github.com/ggml-org/whisper.cpp | MIT. Candidate portable embedded ASR backend with C/C++ API, quantized models, CPU/GPU/Apple Silicon options. Strong Rust integration candidate. |
| WhisperX | https://github.com/m-bain/whisperX | Candidate add-on when word timestamps or diarization are needed. Need dependency/license review, especially diarization stack. |
| mlx-whisper | https://pypi.org/project/mlx-whisper/ | Candidate Apple Silicon local ASR experiment. |
| Vosk | https://alphacephei.com/vosk/ | Lightweight offline ASR option for embedded/streaming use; likely lower accuracy than Whisper-family models. |
| OpenAI Whisper | https://github.com/openai/whisper | MIT. Reference implementation; PyTorch-heavy, less ideal as embedded production backend. |
| OpenAI speech-to-text docs | https://developers.openai.com/api/docs/guides/speech-to-text | Optional explicit paid API fallback only, not local-first default. |
