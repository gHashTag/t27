#!/usr/bin/env python3
# T27 Music Generator - ALL MODES
# phi^2 + 1/phi^2 = 3 | TRINITY

import sys
import os
import argparse
from pathlib import Path

# Import all generators
from music_gen import BarkGenerator, HeartMuLaGenerator, ACEStepGenerator
from lightweight import ProceduralGenerator, SimpleEffects, SimpleMixer

# Support running from trixphi-album directory
if not os.path.exists("music_gen"):
    os.symlink("../music_gen", "music_gen")
if not os.path.exists("lightweight"):
    os.symlink("../lightweight", "lightweight")

# Style prompts
STYLE_PROMPTS = {
    "trap": "140 BPM dark trap beat with heavy 808 bass, rolling hi-hats",
    "phonk": "Drift phonk with distorted bass, bells, aggressive drums, 140 BPM",
    "hiphop": "95 BPM classic hip-hop boom bap drum loop, punchy kick and snare",
    "drill": "UK drill beat with sliding 808, dark melody, 140 BPM",
    "lofi": "85 BPM lo-fi hip-hop chill beat, vinyl crackle, soft piano",
    "edm": "128 BPM EDM house drop, sidechain bass, uplifting melody",
    "ambient": "Cinematic drones and pads, slow evolving textures",
}


def generate_lightweight(style, duration, bpm, output_path):
    """Generate using NumPy lightweight method."""
    print("=== Lightweight NumPy Generation ===")
    gen = ProceduralGenerator(sample_rate=48000, seed=42)
    fx = SimpleEffects(sample_rate=48000)
    mixer = SimpleMixer(sample_rate=48000)

    if style == "trap":
        beat = gen.generate_beat(duration, bpm, "trap")
        bass = gen.generate_bassline(duration, bpm, "trap")
    elif style == "phonk":
        beat = gen.generate_beat(duration, bpm, "trap")
        bass = gen.generate_bassline(duration, bpm, "trap")
    elif style == "drill":
        beat = gen.generate_beat(duration, bpm, "trap")
        bass = gen.generate_bassline(duration, bpm, "trap")
    elif style == "lofi":
        beat = gen.generate_beat(duration, bpm, "hiphop")
        bass = gen.generate_bassline(duration, bpm, "hiphop")
    else:
        beat = gen.generate_beat(duration, bpm, "trap")
        bass = gen.generate_bassline(duration, bpm, "trap")

    # Mix
    mix = mixer.mix(beat, bass, vocal_level_db=-10)
    mix = mixer.master(mix, target_lufs=-14)

    gen.save(mix, output_path)
    return True


def generate_bark(style, duration, output_path):
    """Generate using Bark (Suno AI)."""
    print("=== Bark Generation (Suno AI) ===")
    try:
        gen = BarkGenerator(model_size="small", device="cpu")
        gen.load_model()

        prompt = STYLE_PROMPTS.get(style, STYLE_PROMPTS["trap"])
        audio = gen.generate_music(prompt=prompt, seed=42)
        gen.save(audio, output_path)

        gen.unload_model()
        return True
    except Exception as e:
        print(f"Error: {e}")
        return False


def generate_musicgen(style, duration, output_path):
    """Generate using MusicGen (Meta) via transformers."""
    print("=== MusicGen Generation (Meta) ===")

    try:
        from transformers import AutoProcessor, MusicgenForConditionalGeneration
        import scipy.io.wavfile as wavfile
    except ImportError:
        print("Error: transformers not installed. Use 'pip install transformers scipy'")
        return False

    processor = AutoProcessor.from_pretrained('facebook/musicgen-small')
    model = MusicgenForConditionalGeneration.from_pretrained('facebook/musicgen-small')

    prompt = STYLE_PROMPTS.get(style, STYLE_PROMPTS["trap"])
    print(f"Prompt: {prompt}")

    inputs = processor(
        text=[prompt],
        padding=True,
        return_tensors='pt',
    )

    print("Generating... (this takes 30-60 seconds)")
    audio_values = model.generate(**inputs, max_new_tokens=500)
    audio = audio_values[0, 0].numpy()

    wavfile.write(str(output_path), rate=32000, data=audio)
    print(f"Saved to {output_path}")
    return True


def main():
    parser = argparse.ArgumentParser(description="T27 Music Generator - All Modes")
    parser.add_argument(
        "--mode",
        type=str,
        choices=["lightweight", "bark", "musicgen", "all"],
        default="all",
        help="Generation mode",
    )
    parser.add_argument(
        "--style",
        type=str,
        choices=list(STYLE_PROMPTS.keys()),
        default="trap",
        help="Music style",
    )
    parser.add_argument(
        "--duration",
        type=int,
        default=30,
        help="Duration in seconds",
    )
    parser.add_argument(
        "--bpm",
        type=int,
        default=140,
        help="BPM",
    )
    parser.add_argument(
        "--output",
        type=str,
        help="Output prefix (will append _mode.wav)",
    )

    args = parser.parse_args()

    if args.output:
        prefix = args.output
    else:
        prefix = "music_gen"

    print("╔═════════════════════════════════════════════════════════╗")
    print("║           T27 MUSIC GENERATOR - ALL MODES                 ║")
    print("║  phi^2 + 1/phi^2 = 3 | TRINITY                           ║")
    print("╚═════════════════════════════════════════════════════════╝")
    print()

    if args.mode == "all":
        # Generate with all three methods
        modes = ["lightweight", "bark", "musicgen"]
        for mode in modes:
            output_file = Path(f"{prefix}_{mode}.wav")
            print()
            print(f"{'─'*60}")
            print(f"  MODE: {mode.upper()}")
            print(f"{'─'*60}")

            if mode == "lightweight":
                success = generate_lightweight(
                    args.style, args.duration, args.bpm, output_file
                )
            elif mode == "bark":
                success = generate_bark(
                    args.style, args.duration, output_file
                )
            elif mode == "musicgen":
                success = generate_musicgen(
                    args.style, args.duration, output_file
                )

            if success:
                size_mb = output_file.stat().st_size / (1024 * 1024)
                print(f"  ✅ SAVED: {output_file}")
                print(f"     Size: {size_mb:.2f} MB")

    else:
        # Generate with specific mode
        output_file = Path(f"{prefix}_{args.mode}.wav")
        print(f"Style: {args.style}")
        print(f"Duration: {args.duration}s")
        print(f"BPM: {args.bpm}")
        print()

        if args.mode == "lightweight":
            generate_lightweight(args.style, args.duration, args.bpm, output_file)
        elif args.mode == "bark":
            generate_bark(args.style, args.duration, output_file)
        elif args.mode == "musicgen":
            generate_musicgen(args.style, args.duration, output_file)

        print()
        print(f"✅ Saved to {output_file}")
        size_mb = output_file.stat().st_size / (1024 * 1024)
        print(f"   Size: {size_mb:.2f} MB")

    print()
    print("╔═══════════════════════════════════════════════════════════╗")
    print("║                    GENERATION COMPLETE                         ║")
    print("╚═══════════════════════════════════════════════════════╝")

    # Try to open files
    try:
        import subprocess
        if args.mode == "all":
            # Open all generated files
            for mode in ["lightweight", "bark", "musicgen"]:
                f = Path(f"{prefix}_{mode}.wav")
                if f.exists():
                    subprocess.run(["open", str(f.absolute())], check=False)
        else:
            f = Path(f"{prefix}_{args.mode}.wav")
            if f.exists():
                subprocess.run(["open", str(f.absolute())], check=False)
    except:
        pass

    return 0


if __name__ == "__main__":
    sys.exit(main())
