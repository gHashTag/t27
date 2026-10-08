# NOW -- plan-reel v3: a story reel's poster is drawn once by the server (2026-10-08)

## plan-reel v3: a story reel's poster is drawn once by the server (Closes #7716)

- plan-reel v3: a reel that came out gets a poster -- its frame at PREVIEW_SEEK_MS, POSTER_W 216 px (the 72 px card at density 3) -- drawn once by the server; the card shows the poster instead of fetching the video, the browser's frame stands where none is kept; the reel is out before its poster, and the sweep draws at most 3 missing posters a round, each tried once.
- spec-check 10/10, 0 vacuous; seal saved.
