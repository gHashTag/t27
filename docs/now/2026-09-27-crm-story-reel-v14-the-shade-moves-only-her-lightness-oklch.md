# NOW -- crm-story-reel v14 -- the shade moves only her lightness (OKLCH), her hue stays (2026-09-27)

## crm-story-reel v14 -- the shade moves only her lightness (OKLCH), her hue stays (Closes #4957)

- crm-story-reel v14: the shade moves only lightness in OKLCH; hue and chroma are hers, chroma cut only to the sRGB gamut. The v13 sRGB mix turned a dark blue 19.8 degrees (measured).
- Law: after 8-bit rounding, a colour with chroma >= 0.05 keeps its hue within 3 degrees (hue_kept, hue_checked; measured worst 2.54). The last step is still white or black: a shade always exists.
- The ground walks the same path; her nearest colour is measured in OKLab. 31/31, 0 of 31 vacuous, 36/36 controls (tri spec-check).
