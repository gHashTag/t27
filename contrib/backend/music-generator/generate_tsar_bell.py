#!/usr/bin/env python3
# Generate Tsar Bell Church test track
# phi^2 + 1/phi^2 = 3 | TRINITY

import sys
sys.path.insert(0, '.')

import numpy as np
from pathlib import Path
from lightweight.generative import ProceduralGenerator
from lightweight.effects import SimpleEffects
from lightweight.mixer import SimpleMixer

# Style parameters
BPM = 140
DURATION = 60  # seconds
STYLE = "trap"  # Use trap base, modify for church bells
SAMPLE_RATE = 48000

print(f"=== Generating Tsar Bell Church Track ===")
print(f"BPM: {BPM}")
print(f"Duration: {DURATION}s")
print(f"Sample Rate: {SAMPLE_RATE} Hz")
print()

# Initialize
gen = ProceduralGenerator(sample_rate=SAMPLE_RATE, seed=42)
fx = SimpleEffects(sample_rate=SAMPLE_RATE)
mixer = SimpleMixer(sample_rate=SAMPLE_RATE)

# Custom patterns for the style
# Fast rolling hi-hats (trap style with more activity)
HAT_PATTERN_FAST = [1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1]

# Kick pattern with double-time feel
KICK_PATTERN_DOUBLE = [1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0]

# Snare on 2 and 4 with extra hits
SNARE_PATTERN_TRAP = [0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 1]

print("Step 1/7: Generating drum beat with fast hi-hats...")
beat = gen.generate_beat(
    duration=DURATION,
    bpm=BPM,
    style="trap",
    kick_pattern=KICK_PATTERN_DOUBLE,
    snare_pattern=SNARE_PATTERN_TRAP,
    hihat_pattern=HAT_PATTERN_FAST,
)

print("Step 2/7: Generating heavy 808 bass with slides...")
# Modify bass for more 808 characteristics
bass = gen.generate_bassline(
    duration=DURATION,
    bpm=BPM,
    root_note=36.0,  # Lower root for heavy 808
    style="trap",
)
print(f"  Bass shape: {bass.shape}")

print("Step 3/7: Generating atmospheric synth pads (cinematic, minor)...")
# Generate cinematic pad with minor key
pad = gen.generate_melody(
    duration=DURATION,
    bpm=BPM,
    scale="minor",
    style="eerie",
)
# Make pad more atmospheric
pad_reverb = fx.reverb(pad, room_size=0.8, decay=1.0, wet_level=0.6, dry_level=0.4)
print(f"  Pad shape: {pad_reverb.shape}")

print("Step 4/7: Generating dark church bells (background)...")
# Generate bell sounds using higher frequencies with long decay
bell_audio = np.zeros((2, int(DURATION * SAMPLE_RATE)))
t = np.linspace(0, DURATION, int(DURATION * SAMPLE_RATE))

# Dark church bell frequencies (minor key)
bell_freqs = [130.81, 155.56, 196.00, 233.08, 261.63]  # C3, Eb3, G3, Bb3, C4

for i, freq in enumerate(bell_freqs):
    # Bell timbre: multiple harmonics with long decay
    bell = np.zeros(int(DURATION * SAMPLE_RATE))
    for harmonic in [1, 2.5, 3, 4.5, 6]:  # Inharmonic harmonics for bell sound
        harmonic_freq = freq * harmonic
        harmonic_wave = np.sin(2 * np.pi * harmonic_freq * t)
        decay = np.exp(-2 * t)  # Long decay for bells
        bell += harmonic_wave * decay * (1 / harmonic)

    # Place bells periodically (every 8 beats = ~3.4 seconds)
    bell_interval = int(8 * (60 / BPM) * SAMPLE_RATE)
    bell_position = (i * bell_interval * 3) % bell_audio.shape[1]
    bell_duration = int(bell_interval * 1.5)

    end = min(bell_position + bell_duration, bell_audio.shape[1])
    if bell_position < bell_audio.shape[1]:
        bell_len = min(len(bell), end - bell_position)
        bell_audio[:, bell_position:bell_position + bell_len] += bell[:bell_len] * 0.15

print(f"  Bell audio shape: {bell_audio.shape}")

print("Step 5/7: Applying atmospheric effects...")
# Dark atmospheric processing
# Add depth to the beat
beat_reverb = fx.reverb(beat, room_size=0.7, decay=0.6, wet_level=0.4, dry_level=0.6)

# Process bass for 808 character
bass_distorted = fx.distort(bass, drive_db=20, tone=0.3)
bass_low = fx.eq(bass_distorted, low_db=6, mid_db=-2, high_db=-8)  # Boost low end
bass_final = fx.compress(bass_low, threshold_db=-16, ratio=6)

# Combine bells with pad for atmospheric background
atmosphere = pad_reverb + bell_audio * 0.3
atmosphere = fx.compress(atmosphere, threshold_db=-12, ratio=4)

print("Step 6/7: Mixing all layers...")
# Mix down the layers
mix = mixer.mix(beat_reverb * 0.7, bass_final * 0.8, vocal_level_db=-8)
mix = mixer.mix(mix, atmosphere * 0.4, vocal_level_db=-12)

print("Step 7/7: Mastering with cinematic processing...")
# Cinematic mastering
# Add stereo width for atmosphere
mastered = mixer.master(mix, target_lufs=-12, stereo_width=1.3)

# Final cinematic touches
# Subtle distortion for grit
final = fx.distort(mastered, drive_db=5, tone=0.7)
# Reverb for cathedral atmosphere
final = fx.reverb(final, room_size=0.9, decay=0.8, wet_level=0.3, dry_level=0.7)
# Final limiter
final = fx.limit(final, threshold_db=-0.5)

# Save
output_path = Path("tsar_bell_church_test.wav")
try:
    gen.save(final, output_path)
    print()
    print(f"=== TRACK SAVED ===")
    print(f"Path: {output_path.absolute()}")
    print(f"Size: {output_path.stat().st_size / (1024*1024):.1f} MB")
    print()
    print(f"=== AUDIO INFO ===")
    info = mixer.analyze(final)
    print(f"Peak: {info['peak_db']:.2f} dB")
    print(f"RMS:  {info['rms_db']:.2f} dB")
    print(f"LUFS: ~{info['lufs_approx']:.1f}")
    print(f"Dynamic Range: {info['dynamic_range']:.2f} dB")
    print()
    print("=== TRACK DESCRIPTION ===")
    print("Style: Tsar Bell Church")
    print("Genre: Dark Trap / Cinematic Hip-Hop")
    print(f"BPM: {BPM}")
    print(f"Duration: {DURATION}s")
    print("Elements:")
    print("  - Dark church bells (background)")
    print("  - Heavy 808 bass with slides")
    print("  - Fast rolling hi-hats (machine gun flow)")
    print("  - Atmospheric synth pads (minor key, cinematic)")
    print("  - Double-time / triplet flows")
    print("  - Grime/trap influence")
    print()
except Exception as e:
    print(f"Error saving (soundfile not installed): {e}")
    print("Audio generated successfully, but not saved.")

print("=== GENERATION COMPLETE ===")
