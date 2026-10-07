use serde_json::Value;
use std::path::Path;

pub struct Dims {
    pub mel_bins: u32,
    pub frames: u32,
    pub positions: u32,
    pub target_positions: u32,
    pub state: u32,
    pub heads: u32,
    pub encoder_ffn: u32,
    pub decoder_ffn: u32,
    pub encoder_layers: u32,
    pub decoder_layers: u32,
    pub vocab: u32,
    pub floor: f32,
}

impl Dims {
    pub fn of(directory: &Path) -> Self {
        let path = directory.join("config.json");
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|error| panic!("the model holds no {}: {error}", path.display()));
        let config: Value = serde_json::from_slice(&bytes)
            .unwrap_or_else(|error| panic!("{} holds no json: {error}", path.display()));
        let number = |key: &str| {
            config
                .get(key)
                .and_then(Value::as_u64)
                .map(|value| value as u32)
                .unwrap_or_else(|| panic!("the config of {} holds no {key}", directory.display()))
        };
        let positions = number("max_source_positions");
        let heads = number("encoder_attention_heads");
        let state = number("d_model");
        let optional = |key: &str| {
            config
                .get(key)
                .and_then(Value::as_u64)
                .map(|value| value as u32)
                .unwrap_or(state * 4)
        };
        assert_eq!(
            number("decoder_attention_heads"),
            heads,
            "the model of {} reads queries of {heads} heads and keys of another count",
            directory.display(),
        );
        let dims = Self {
            mel_bins: number("num_mel_bins"),
            frames: positions * 2,
            positions,
            target_positions: number("max_target_positions"),
            state,
            heads,
            encoder_ffn: optional("encoder_ffn_dim"),
            decoder_ffn: optional("decoder_ffn_dim"),
            encoder_layers: number("encoder_layers"),
            decoder_layers: number("decoder_layers"),
            vocab: number("vocab_size"),
            floor: config
                .get("layer_norm_eps")
                .and_then(Value::as_f64)
                .map(|value| value as f32)
                .unwrap_or(1e-5),
        };
        dims.assert();
        dims
    }

    fn assert(&self) {
        assert!(
            self.mel_bins > 0 && self.positions > 0 && self.state > 0 && self.vocab > 0,
            "this model declares no shape",
        );
        assert!(
            self.encoder_layers > 0 && self.decoder_layers > 0,
            "this model declares no layer",
        );
        assert!(
            self.state.is_multiple_of(self.heads),
            "this model spreads {} numbers over {} heads",
            self.state,
            self.heads,
        );
        assert!(
            self.floor > 0.0,
            "this model normalizes with a floor of {}",
            self.floor,
        );
    }

    pub fn width(&self) -> u32 {
        self.state / self.heads
    }

    pub fn scale(&self) -> f32 {
        1.0 / (self.width() as f32).sqrt()
    }

    pub fn prefix(&self) -> u32 {
        self.target_positions.min(64)
    }
}
