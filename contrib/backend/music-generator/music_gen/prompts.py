# contrib/backend/music-generator/music_gen/prompts.py
# Genre-specific music generation prompts
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Genre-specific prompts for MusicGen generation.

Provides prompt templates and enhancement functions for different
musical genres including phonk, trap, hip-hop, drill, and lofi.
"""

from typing import List, Dict


# Genre-specific prompt templates
PHONK_PROMPTS: List[str] = [
    "dark drift phonk, aggressive double-time cowbell, heavy 808 bass, hypnotic melody",
    "cowbell melody, drift phonk, distorted bass, Memphis rap influence, dark atmosphere",
    "aggressive phonk, bells and 808, driving beat, ominous synthesizer",
    "phonk drift, heavy distortion, fast tempo, bells, ominous atmosphere",
]

TRAP_PROMPTS: List[str] = [
    "hard trap, rolling hi-hats, 808 slides, dark atmosphere, minimal melody",
    "trap beat, 808 bass, fast hi-hats, dark synth plucks, minor key",
    "Atlanta trap, rolling 808s, triplet hi-hats, dark piano, atmospheric",
    "trap instrumental, heavy 808, rapid hi-hats, sparse melody, moody",
]

HIP_HOP_PROMPTS: List[str] = [
    "boom bap hip-hop, dusty drums, soul sample, classic feel, head-nodding beat",
    "golden era hip-hop, sampled drums, jazz piano loop, gritty texture",
    "90s hip-hop, boom bap drums, soul sample, vinyl crackle, classic",
    "lo-fi hip-hop beat, dusty sample, chill drums, nostalgic atmosphere",
]

DRILL_PROMPTS: List[str] = [
    "UK drill, sliding 808s, dark melody, fast hi-hats, ominous atmosphere",
    "Brooklyn drill, heavy 808 slides, dark synthesizer, fast-paced hi-hats",
    "drill beat, sliding bass, dark piano, rapid hi-hats, menacing",
    "NY drill, 808 slides, dark melody, trap hi-hats, aggressive",
]

LOFI_PROMPTS: List[str] = [
    "lofi hip-hop, chill beats, vinyl crackle, nostalgic, rainy day vibe",
    "chill lofi, soft piano, vinyl hiss, cozy atmosphere, slow tempo",
    "lofi beats, cassette tape quality, warm pad, relaxing, study music",
    "downtempo lofi, vinyl noise, soft melody, chill drums, peaceful",
]


# Genre enhancement templates
GENRE_ENHANCEMENTS: Dict[str, str] = {
    "phonk": ", dark drift phonk style, aggressive double-time cowbell, heavy distorted 808 bass",
    "trap": ", hard trap style, rolling hi-hats, 808 slides, dark atmospheric production",
    "hip_hop": ", boom bap hip-hop style, dusty drums, soul sample influence, classic feel",
    "drill": ", UK drill style, sliding 808s, dark minor melody, fast triplet hi-hats",
    "lofi": ", lofi hip-hop style, vinyl crackle, warm atmosphere, nostalgic quality",
}


def enhance_prompt_for_genre(prompt: str, genre: str) -> str:
    """Enhance a user prompt with genre-specific characteristics.

    Args:
        prompt: Base prompt from user
        genre: Target genre (phonk, trap, hip_hop, drill, lofi)

    Returns:
        Enhanced prompt with genre-specific descriptors

    Complexity: O(1)

    Example:
        >>> enhance_prompt_for_genre("melancholic", "phonk")
        "melancholic, dark drift phonk style, aggressive double-time cowbell, heavy distorted 808 bass"
    """
    genre_key = genre.lower().replace("-", "_")
    enhancement = GENRE_ENHANCEMENTS.get(genre_key, "")

    if not enhancement:
        return prompt

    return f"{prompt}{enhancement}"


def get_random_prompt_for_genre(genre: str) -> str:
    """Get a random prompt template for the specified genre.

    Args:
        genre: Target genre (phonk, trap, hip_hop, drill, lofi)

    Returns:
        Random prompt string for the genre

    Complexity: O(1)

    Example:
        >>> get_random_prompt_for_genre("phonk")
        "dark drift phonk, aggressive double-time cowbell, heavy 808 bass, hypnotic melody"
    """
    import random

    genre_prompts_map: Dict[str, List[str]] = {
        "phonk": PHONK_PROMPTS,
        "trap": TRAP_PROMPTS,
        "hip_hop": HIP_HOP_PROMPTS,
        "drill": DRILL_PROMPTS,
        "lofi": LOFI_PROMPTS,
    }

    prompts = genre_prompts_map.get(genre.lower(), PHONK_PROMPTS)
    return random.choice(prompts)
