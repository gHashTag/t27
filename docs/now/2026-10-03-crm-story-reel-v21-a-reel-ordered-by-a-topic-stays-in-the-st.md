# NOW -- crm-story-reel v21: a reel ordered by a topic stays in the studio (2026-10-03)

## crm-story-reel v21: a reel ordered by a topic stays in the studio (Closes #5765)

- NEW_SETTING_EACH_SCENE false: the topic writer was told to invent a new place per scene, paying a still and a face check each
- WRITER_WRITES_SETTINGS false: the writer is asked for lines only; WRITER_SETTING_DROPPED: a place it returns anyway is dropped by code
- setting_drawn(from_writer, has_setting): a place is drawn only when a caller names it in a script of their own
