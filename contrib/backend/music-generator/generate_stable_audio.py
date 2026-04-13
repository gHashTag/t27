#!/usr/bin/env python3
# Generate music using Stable Audio Open 1.0 (Stability AI)
# phi^2 + 1/phi^2 = 3 | TRINITY

import sys
sys.path.insert(0, '.')

import argparse
from pathlib import Path
from music_gen import StableAudioGenerator, enhance_prompt_for_genre, TRAP_PROMPTS

# Default prompts for different styles
STYLE_PROMPTS = {
    "trap": "140 BPM dark trap beat with heavy 808 bass, rolling hi-hats, cinematic atmosphere",
    "phonk": "140 BPM drift phonk with distorted bass, bells, and aggressive drums",
    "hiphop": "95 BPM classic hip-hop boom bap drum loop with punchy kick and snare",
    "drill": "140 BPM UK drill beat with sliding 808, dark melody, and crisp hi-hats",
    "lofi": "85 BPM lo-fi hip-hop chill beat with vinyl crackle and soft piano",
    "edm": "128 BPM EDM house drop with sidechain bass and uplifting melody",
    "ambient": "Drones and pads for cinematic atmosphere, slow evolving textures",
}


def main():
    parser = argparse.ArgumentParser(description="Generate music using Stable Audio")
    parser.add_argument(
        "--prompt",
        type=str,
        help="Text prompt for music generation",
    )
    parser.add_argument(
        "--style",
        type=str,
        choices=list(STYLE_PROMPTS.keys()),
        help="Predefined style to use",
        default="trap",
    )
    parser.add_argument(
        "--duration",
        type=float,
        default=30.0,
        help="Duration in seconds (max 47)",
    )
    parser.add_argument(
        "--output",
        type=str,
        default="stable_audio_output.wav",
        help="Output file path",
    )
    parser.add_argument(
        "--steps",
        type=int,
        default=200,
        help="Number of inference steps (higher = better quality, slower)",
    )
    parser.add_argument(
        "--cfg",
        type=float,
        default=7.0,
        help="Classifier-free guidance scale (higher = more prompt adherence)",
    )
    parser.add_argument(
        "--seed",
        type=int,
        help="Random seed for reproducibility",
    )
    parser.add_argument(
        "--device",
        type=str,
        default="cpu",
        choices=["cpu", "cuda"],
        help="Device to use for generation",
    )
    parser.add_argument(
        "--negative",
        type=str,
        default="Low quality, distorted, noisy, blurry",
        help="Negative prompt",
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
        # Enhance with genre-specific terms
        prompt = enhance_prompt_for_genre(prompt, args.style)

    print("=== Stable Audio Music Generator ===")
    print(f"Model: Stability AI Stable Audio Open 1.0")
    print(f"Device: {args.device}")
    print(f"Style: {args.style}")
    print(f"Prompt: {prompt}")
    print(f"Duration: {args.duration}s")
    print(f"Steps: {args.steps}")
    print(f"CFG Scale: {args.cfg}")
    print(f"Seed: {args.seed if args.seed else 'random'}")
    print(f"Output: {args.output}")
    print()

    # Initialize generator
    print("Loading Stable Audio model...")
    gen = StableAudioGenerator(device=args.device, max_duration=45.0)

    try:
        gen.load_model()
        print("Model loaded!")
        print()

        # Generate
        print("Generating audio... (this may take a while on CPU)")
        audio = gen.generate(
            prompt=prompt,
            duration=args.duration,
            negative_prompt=args.negative,
            num_inference_steps=args.steps,
            cfg_scale=args.cfg,
            seed=args.seed,
        )

        print("Generation complete!")
        print(f"  Audio shape: {audio.shape}")
        print(f"  Duration: {audio.shape[-1] / gen.sample_rate:.1f}s")
        print()

        # Save
        output_path = Path(args.output)
        gen.save(audio, output_path)

        print(f"=== AUDIO SAVED ===")
        print(f"Path: {output_path.absolute()}")
        print(f"Size: {output_path.stat().st_size / (1024*1024):.1f} MB")
        print(f"Sample Rate: {gen.sample_rate} Hz")
        print(f"Channels: {'stereo' if audio.shape[0] == 2 else 'mono'}")
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
        print("To install Stable Audio dependencies:")
        print("  pip install diffusers soundfile torchaudio transformers")
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
