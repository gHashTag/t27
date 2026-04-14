#!/usr/bin/env python3
# Generate music using MusicGen (Meta AudioCraft)
# phi^2 + 1/phi^2 = 3 | TRINITY

import sys
sys.path.insert(0, '.')

import argparse
from pathlib import Path
from music_gen import MusicGenWrapper

# Genre-specific prompts
GENRE_PROMPTS = {
    "trap": "140 BPM dark trap beat with heavy 808 bass, rolling hi-hats, hard hitting snare",
    "phonk": "Drift phonk with distorted bass, bells, and aggressive drums, 140 BPM",
    "hiphop": "95 BPM classic hip-hop boom bap drum loop with punchy kick and snare",
    "drill": "UK drill beat with sliding 808 and dark melody, 140 BPM",
    "lofi": "85 BPM lo-fi hip-hop chill beat with vinyl crackle and soft piano",
    "edm": "128 BPM EDM house drop with sidechain bass and uplifting melody",
    "ambient": "Cinematic drones and pads, slow evolving textures for atmosphere",
}


def main():
    parser = argparse.ArgumentParser(description="Generate music using MusicGen (Meta)")
    parser.add_argument(
        "--prompt",
        type=str,
        help="Text prompt for music generation",
    )
    parser.add_argument(
        "--genre",
        type=str,
        choices=list(GENRE_PROMPTS.keys()),
        help="Predefined genre/style",
        default="trap",
    )
    parser.add_argument(
        "--model-size",
        type=str,
        choices=["small", "medium", "large"],
        default="small",
        help="Model size (small=fast, medium=balanced, large=best quality)",
    )
    parser.add_argument(
        "--duration",
        type=int,
        default=30,
        help="Duration in seconds (max 30 for small, can extend with chunks)",
    )
    parser.add_argument(
        "--bpm",
        type=int,
        help="BPM for tempo guidance",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="musicgen_output.wav",
        help="Output file path",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="cpu",
        choices=["cpu", "cuda"],
        help="Device to use (CPU recommended for no GPU)",
    )
    parser.add_argument(
        "--sample-rate",
        type=int,
        default=48000,
        help="Sample rate in Hz",
    )
    parser.add_argument(
        "--list-genres",
        action="store_true",
        help="List available genres and exit",
    )

    args = parser.parse_args()

    # List genres
    if args.list_genres:
        print("=== Available Genres ===")
        for genre, prompt in GENRE_PROMPTS.items():
            print(f"{genre:12} : {prompt}")
        return

    # Build prompt
    if args.prompt:
        prompt = args.prompt
    else:
        prompt = GENRE_PROMPTS.get(args.genre, GENRE_PROMPTS["trap"])

    print("=== MusicGen Music Generator ===")
    print(f"Model: Meta AudioCraft MusicGen")
    print(f"Size: {args.model_size}")
    print(f"Device: {args.device}")
    print(f"Genre: {args.genre}")
    print(f"Prompt: {prompt}")
    print(f"Duration: {args.duration}s")
    print(f"BPM: {args.bpm if args.bpm else 'auto'}")
    print(f"Sample Rate: {args.sample_rate} Hz")
    print(f"Output: {args.output}")
    print()

    # Initialize generator
    print(f"Loading MusicGen {args.model_size} model...")
    gen = MusicGenWrapper(
        model_size=args.model_size,
        device=args.device,
        sample_rate=args.sample_rate,
    )

    try:
        gen.load_model()
        print("Model loaded!")
        print()

        # Generate
        print("Generating music... (MusicGen is optimized for music!)")
        audio = gen.generate(
            prompt=prompt,
            duration=args.duration,
            bpm=args.bpm,
            genre=args.genre,
            output_path=Path(args.output),
        )

        print()
        print("=== GENERATION COMPLETE ===")
        output_path = Path(args.output)

        if output_path.exists():
            size_mb = output_path.stat().st_size / (1024 * 1024)
            print(f"Saved: {output_path}")
            print(f"Size: {size_mb:.2f} MB")
            print(f"Channels: {'stereo' if audio.shape[0] == 2 else 'mono'}")
            print(f"Sample Rate: {args.sample_rate} Hz")
            print(f"Duration: {audio.shape[-1] / args.sample_rate:.1f}s")

            # Open in player
            try:
                import subprocess
                print()
                print(f"Opening {output_path} in default player...")
                subprocess.run(["open", str(output_path.absolute())])
            except:
                pass
        else:
            print("Warning: Output file not found")

    except ImportError as e:
        print(f"Error: {e}")
        print()
        print("To install MusicGen dependencies:")
        print("  pip install audiocraft")
        print()
        print("Or install all music-generator dependencies:")
        print("  pip install -r requirements.txt")
        return 1
    except Exception as e:
        print(f"Error during generation: {e}")
        import traceback
        traceback.print_exc()
        return 1
    finally:
        # Clean up
        gen.unload_model()

    return 0


if __name__ == "__main__":
    sys.exit(main())
