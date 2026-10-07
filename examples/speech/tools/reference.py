import argparse
import json
import pathlib
import shutil
import wave

import numpy as np
import torch
import torch.nn.functional as functional
from safetensors.torch import load_file, save_file
from transformers import WhisperConfig, WhisperForConditionalGeneration

SOT = 50258
EOT = 50257
TRANSCRIBE = 50359
NO_TIMESTAMPS = 50363
SAMPLE_RATE = 16000
N_FFT = 400
HOP = 160
CHUNK = 30 * SAMPLE_RATE
GENERATE = 64


def patch_activation(model):
    original = functional.gelu
    functional.gelu = lambda input, approximate="tanh": original(input, approximate=approximate)
    for module in (model.model.encoder, model.model.decoder):
        for layer in module.layers:
            assert hasattr(layer, "activation_fn"), "a layer of this model reads no activation"
            layer.activation_fn = torch.nn.GELU(approximate="tanh")


def hidden_to_heads(hidden, heads, width):
    batch, frames, state = hidden.shape
    return hidden.reshape(batch, frames, heads, width).permute(2, 0, 1, 3).contiguous()


def load_json(path):
    with open(path, encoding="utf-8") as handle:
        return json.load(handle)


def read_wave(path):
    with wave.open(str(path)) as handle:
        if handle.getnchannels() != 1 or handle.getframerate() != SAMPLE_RATE or handle.getsampwidth() != 2:
            raise ValueError("a wave file holds one channel of sixteen bit samples at 16000 Hz")
        frames = handle.readframes(handle.getnframes())
    return np.frombuffer(frames, dtype="<i2").astype(np.float32) / 32768.0


def synthetic(count):
    entropy = 0x9E3779B9
    values = np.empty(count, dtype=np.float32)
    for index in range(count):
        entropy = (entropy ^ (entropy << 13)) & 0xFFFFFFFF
        entropy = entropy ^ (entropy >> 17)
        entropy = (entropy ^ (entropy << 5)) & 0xFFFFFFFF
        values[index] = (entropy >> 8) / 16777216.0 - 0.5
    return values


def log_mel(audio, filters):
    samples = torch.from_numpy(np.asarray(audio, dtype=np.float32))
    if samples.shape[0] > CHUNK:
        samples = samples[:CHUNK]
    if samples.shape[0] < CHUNK:
        samples = functional.pad(samples, (0, CHUNK - samples.shape[0]))
    spectrogram = torch.stft(samples, N_FFT, HOP, window=torch.hann_window(N_FFT), return_complex=True)
    magnitudes = spectrogram[..., :-1].abs() ** 2
    mel = filters @ magnitudes
    logged = torch.clamp(mel, min=1e-10).log10()
    logged = torch.maximum(logged, logged.max() - 8.0)
    return (logged + 4.0) / 4.0


def encode(model, mel):
    return model.model.encoder(mel.unsqueeze(0)).last_hidden_state


def cross_of(model, hidden):
    heads = model.config.encoder_attention_heads
    width = model.config.d_model // heads
    keys = []
    values = []
    for layer in model.model.decoder.layers:
        keys.append(hidden_to_heads(layer.encoder_attn.k_proj(hidden), heads, width))
        values.append(hidden_to_heads(layer.encoder_attn.v_proj(hidden), heads, width))
    return keys, values


def greedy(model, hidden, prompt, cap):
    issued = []
    logits = []
    while len(issued) < cap:
        prefix = torch.tensor([prompt + issued], dtype=torch.long)
        step = model.proj_out(model.model.decoder(prefix, encoder_hidden_states=hidden).last_hidden_state)[0, -1]
        logits.append(step.detach())
        token = int(step.argmax())
        issued.append(token)
        if token == EOT:
            break
    return torch.stack(logits), issued


def golden(model, mel, prompt, out, cap, name="golden.safetensors"):
    with torch.no_grad():
        hidden = encode(model, mel)
        keys, values = cross_of(model, hidden)
        logits, issued = greedy(model, hidden, prompt, cap)
    pack = {
        "mel": mel.contiguous(),
        "logits": logits.contiguous(),
        "tokens": torch.tensor([issued], dtype=torch.float32),
    }
    for index, (key, value) in enumerate(zip(keys, values)):
        pack[f"key.{index}"] = key.contiguous()
        pack[f"value.{index}"] = value.contiguous()
    save_file(pack, str(out / name))
    return issued


def decode_tokens(model_dir, tokens):
    vocab = {index: token for token, index in load_json(model_dir / "vocab.json").items()}
    encoder = byte_encoder()
    raw = bytearray()
    for token in tokens:
        if token >= EOT:
            continue
        piece = vocab.get(token)
        if piece is None:
            continue
        for character in piece:
            byte = encoder.get(character)
            if byte is None:
                raw.extend(character.encode("utf-8"))
            else:
                raw.append(byte)
    return raw.decode("utf-8", errors="replace")


def byte_encoder():
    printable = list(range(ord("!"), ord("~") + 1)) + list(range(ord("\u00a1"), ord("\u00ac") + 1)) + list(
        range(ord("\u00ae"), ord("\u00ff") + 1)
    )
    mapped = printable[:]
    escaped = 0
    for byte in range(256):
        if byte not in printable:
            printable.append(byte)
            mapped.append(256 + escaped)
            escaped += 1
    return {chr(code): byte for byte, code in zip(printable, mapped)}


def real(model_dir, audio, language, out, wave=None):
    model = WhisperForConditionalGeneration.from_pretrained(str(model_dir), dtype=torch.float32)
    model.eval()
    patch_activation(model)
    filters = torch.tensor(load_json(model_dir / "preprocessor_config.json")["mel_filters"], dtype=torch.float32)
    mel = log_mel(audio, filters)
    prompt = [SOT, language, TRANSCRIBE, NO_TIMESTAMPS]
    out.mkdir(parents=True, exist_ok=True)
    issued = golden(model, mel, prompt, out, GENERATE)
    text = decode_tokens(model_dir, issued)
    (out / "transcript.txt").write_text(text, encoding="utf-8")
    print(f"transcript: {text}")
    print(f"steps: {len(issued)} tokens")
    if wave is not None:
        shutil.copyfile(wave, out / "speech.wav")


def fixture(out):
    config = WhisperConfig(
        d_model=8,
        encoder_layers=2,
        decoder_layers=2,
        encoder_attention_heads=2,
        decoder_attention_heads=2,
        num_mel_bins=4,
        vocab_size=64,
        max_source_positions=8,
        max_target_positions=64,
        activation_function="gelu",
        encoder_ffn_dim=16,
        decoder_ffn_dim=16,
        pad_token_id=0,
        eos_token_id=0,
        bos_token_id=0,
        decoder_start_token_id=0,
    )
    torch.manual_seed(0x5EED)
    model = WhisperForConditionalGeneration(config)
    model.eval()
    patch_activation(model)
    directory = out / "fixture"
    directory.mkdir(parents=True, exist_ok=True)
    state = {name: tensor.detach().to(torch.float32).contiguous() for name, tensor in model.state_dict().items()}
    state.pop("proj_out.weight", None)
    save_file(state, str(directory / "model.safetensors"))
    (directory / "config.json").write_text(json.dumps(config.to_dict(), indent=2), encoding="utf-8")
    mel = torch.arange(config.num_mel_bins * config.max_source_positions * 2, dtype=torch.float32).reshape(
        1, config.num_mel_bins, config.max_source_positions * 2
    ) / 64.0
    issued = golden(model, mel[0], [1, 2, 3, 4], out, 4, "fixture_golden.safetensors")
    print(f"fixture: mel {tuple(mel.shape)} tokens {issued}")


def medium(out):
    config = WhisperConfig(
        d_model=32,
        encoder_layers=2,
        decoder_layers=2,
        encoder_attention_heads=4,
        decoder_attention_heads=4,
        num_mel_bins=80,
        vocab_size=64,
        max_source_positions=1500,
        max_target_positions=64,
        activation_function="gelu",
        encoder_ffn_dim=64,
        decoder_ffn_dim=64,
        pad_token_id=0,
        eos_token_id=0,
        bos_token_id=0,
        decoder_start_token_id=0,
    )
    torch.manual_seed(0x5EED)
    model = WhisperForConditionalGeneration(config)
    model.eval()
    patch_activation(model)
    directory = out / "medium"
    directory.mkdir(parents=True, exist_ok=True)
    state = {name: tensor.detach().to(torch.float32).contiguous() for name, tensor in model.state_dict().items()}
    state.pop("proj_out.weight", None)
    save_file(state, str(directory / "model.safetensors"))
    (directory / "config.json").write_text(json.dumps(config.to_dict(), indent=2), encoding="utf-8")
    mel = torch.arange(config.num_mel_bins * config.max_source_positions * 2, dtype=torch.float32).reshape(
        1, config.num_mel_bins, config.max_source_positions * 2
    ) / 8192.0
    issued = golden(model, mel[0], [1, 2, 3, 4], out, 4, "fixture_golden.safetensors")
    print(f"medium: mel {tuple(mel.shape)} tokens {issued}")


def mel_golden(out, seconds=1.0):
    model_dir = pathlib.Path(__file__).resolve().parents[3] / "target" / "speech" / "whisper-tiny"
    preprocessor = load_json(model_dir / "preprocessor_config.json")
    filters = torch.tensor(preprocessor["mel_filters"], dtype=torch.float32)
    audio = synthetic(int(SAMPLE_RATE * seconds))
    mel = log_mel(audio, filters)[:, : int(seconds * 100)]
    directory = out / "mel"
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "preprocessor_config.json").write_text(json.dumps(preprocessor), encoding="utf-8")
    save_file({"mel": mel.contiguous()}, str(directory / "golden.safetensors"))
    print(f"mel golden {tuple(mel.shape)} taps {tuple(filters.shape)}")


def vocabulary(out):
    model_dir = pathlib.Path(__file__).resolve().parents[3] / "target" / "speech" / "whisper-tiny"
    vocab = load_json(model_dir / "vocab.json")
    pieces = {index: token for token, index in vocab.items()}
    picked = [0, 12, 220, 262, 2201, 10475, 3318, 50254, 45678, 32100, 171, 8731]
    subset = {}
    for token in picked:
        piece = pieces[token]
        subset[piece] = token
    directory = out / "vocabulary"
    directory.mkdir(parents=True, exist_ok=True)
    (directory / "vocab.json").write_text(json.dumps(subset, ensure_ascii=False, indent=2), encoding="utf-8")
    (directory / "added_tokens.json").write_text(json.dumps({"<|en|>": 1, "<|zh|>": 2}, indent=2), encoding="utf-8")
    (directory / "ids.json").write_text(json.dumps(picked), encoding="utf-8")
    (directory / "expected.txt").write_text(decode_tokens(model_dir, picked), encoding="utf-8")
    print(f"vocabulary: {len(subset)} pieces")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-dir", type=pathlib.Path)
    parser.add_argument("--audio", type=pathlib.Path)
    parser.add_argument("--language", type=int, default=50259)
    parser.add_argument("--fixture", action="store_true")
    parser.add_argument("--medium", action="store_true")
    parser.add_argument("--mel-golden", action="store_true")
    parser.add_argument("--vocabulary", action="store_true")
    parser.add_argument("--out", type=pathlib.Path, required=True)
    arguments = parser.parse_args()
    if arguments.fixture:
        fixture(arguments.out)
    elif arguments.medium:
        medium(arguments.out)
    elif arguments.mel_golden:
        mel_golden(arguments.out)
    elif arguments.vocabulary:
        vocabulary(arguments.out)
    elif arguments.audio is not None:
        real(arguments.model_dir, read_wave(arguments.audio), arguments.language, arguments.out, arguments.audio)
    else:
        raise SystemExit("an audio file, --fixture, --mel-golden or --vocabulary is required")


if __name__ == "__main__":
    main()
