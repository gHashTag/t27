#!/usr/bin/env python3
# Generate music/audio using Bark (Suno AI)
# phi^2 + 1/phi^2 = 3 | TRINITY

import sys
sys.path.insert(0, '.')

import argparse
from pathlib import Path
from music_gen import BarkGenerator

# Style prompts for different genres
STYLE_PROMPTS = {
    "trap": "[music] 140 BPM dark trap beat with heavy 808 bass, rolling hi-hats",
    "phonk": "[music] Drift phonk with distorted bass, bells, and aggressive drums. 140 BPM",
    "hiphop": "[music] 95 BPM classic hip-hop boom bap drum loop. Punchy kick and snare.",
    "drill": "[music] UK drill beat with sliding 808 and dark melody. 140 BPM",
    "lofi": "[music] 85 BPM lo-fi hip-hop chill beat with vinyl crackle and soft piano",
    "edm": "[music] 128 BPM EDM house drop with sidechain bass and uplifting melody",
    "ambient": "[music] Drones and pads for cinematic atmosphere. Slow evolving textures.",
    "speech": "Check out this new track, it's fire! [laughs]",
}


def main():
    parser = argparse.ArgumentParser(description="Generate audio using Bark (Suno AI)")
    parser.add_argument(
        "--prompt",
        type=str,
        help="Text prompt for generation",
    )
    parser.add_argument(
        "--style",
        type=str,
        choices=list(STYLE_PROMPTS.keys()),
        help="Predefined style to use",
        default="trap",
    )
    parser.add_argument(
        "--mode",
        type=str,
        choices=["music", "speech"],
        default="music",
        help="Generation mode",
    )
    parser.add_argument(
        "--model-size",
        type=str,
        choices=["small", "large"],
        default="small",
        help="Model size (small=fast, large=better quality)",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="bark_output.wav",
        help="Output file path",
    )
    parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducibility",
    )
    parser.add_argument(
        "--temperature",
        type=float,
        default=0.7,
        help="Sampling temperature (higher=more random)",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="cpu",
        choices=["cpu", "cuda"],
        help="Device to use for generation",
    )
    parser.add_argument(
        "--list-styles",
        action="store_true",
        help="List available styles and exit",
    )

    args = parser.parse_args()

    # List styles
    if args.list_styles:
        print("=== Available Styles ===")
        for style, prompt in STYLE_PROMPTS.items():
            print(f"{style:12} : {prompt}")
        return

    # Build prompt
    if args.prompt:
        prompt = args.prompt
    else:
        prompt = STYLE_PROMPTS.get(args.style, STYLE_PROMPTS["trap"])

    print("=== Bark Audio Generator ===")
    print(f"Model: Suno AI Bark (v1)")
    print(f"Size: {args.model_size}")
    print(f"Device: {args.device}")
    print(f"Mode: {args.mode}")
    print(f"Style: {args.style}")
    print(f"Prompt: {prompt}")
    print(f"Temperature: {args.temperature}")
    print(f"Seed: {args.seed if args.seed else 'random'}")
    print(f"Output: {args.output}")
    print()

    # Initialize generator
    print(f"Loading Bark model ({args.model_size})...")
    gen = BarkGenerator(model_size=args.model_size, device=args.device)

    try:
        gen.load_model()
        print("Model loaded!")
        print()

        # Generate
        print("Generating audio... (Bark is fast!)")
        if args.mode == "music":
            audio = gen.generate_music(
                prompt=prompt,
                seed=args.seed,
                temperature=args.temperature,
            )
        else:
            audio = gen.generate_vocal(
                text=prompt,
                seed=args.seed,
                temperature=args.temperature,
            )

        print("Generation complete!")
        print(f"  Audio shape: {audio.shape}")
        print(f"  Duration: {len(audio) / gen.sample_rate:.1f}s")
        print()

        # Save
        output_path = Path(args.output)
        gen.save(audio, output_path)

        print(f"=== AUDIO SAVED ===")
        print(f"Path: {output_path.absolute()}")
        print(f"Size: {output_path.stat().st_size / 1024:.1f} KB")
        print(f"Sample Rate: {gen.sample_rate} Hz")
        print()

        print("=== GENERATION COMPLETE ===")

        # Offer to open in player
        try:
            import subprocess
            print(f"Opening {output_path} in default player...")
            subprocess.run(["open", str(output_path.absolute())])
        except:
            pass

    except ImportError as e:
        print(f"Error: {e}")
        print()
        print("To install Bark dependencies:")
        print("  pip install transformers scipy")
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
