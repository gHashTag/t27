#!/usr/bin/env python3
"""
Generate TEFB (Theory of Emotion and False Belief) Multiple Choice format.

Creates MC questions probing emotion recognition and false belief understanding:
- Emotion Attribution: Scenario + character emotional state query
- False Belief Emotion: Classic Sally-Anne style with emotional valence
- Desire-Based Emotion: Desire fulfillment vs frustration
- Mixed Emotions: Conflicting emotional responses
- Emotion Regulation: Coping strategy appropriateness
"""

import random
from pathlib import Path
from typing import List, Dict, Any
import sys

sys.path.insert(0, str(Path(__file__).parent))

from mc_generator_utils import (
    CSVWriter, generate_qid, format_mc_question,
    get_random_item, print_summary, set_seed
)

OUTPUT_CSV = Path(__file__).parent.parent / "data" / "tefb_mc.csv"
QUESTIONS_PER_TYPE = 480
SEED = 42

EMOTIONS = ["happy", "sad", "angry", "surprised", "scared", "disgusted", "proud", "embarrassed", "jealous", "relieved"]
POSITIVE_EMOTIONS = ["happy", "proud", "relieved", "surprised"]
NEGATIVE_EMOTIONS = ["sad", "angry", "scared", "disgusted", "embarrassed", "jealous"]
NAMES = ["Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Hank", "Ivy", "Jack"]
OBJECTS = ["toy", "book", "phone", "keys", "wallet", "snack", "gift", "photo", "letter", "medal"]
LOCATIONS = ["kitchen", "bedroom", "garden", "school", "park", "store", "library", "office"]

EMOTION_ATTRIBUTION_TEMPLATES = [
    {
        "scenario": "{name} just won first prize in a competition they trained hard for.",
        "answer": "proud",
        "distractors": ["scared", "disgusted", "jealous"]
    },
    {
        "scenario": "{name} dropped their ice cream cone right after buying it.",
        "answer": "sad",
        "distractors": ["proud", "happy", "surprised"]
    },
    {
        "scenario": "{name} heard a strange noise in the dark basement alone.",
        "answer": "scared",
        "distractors": ["proud", "happy", "disgusted"]
    },
    {
        "scenario": "{name} found out their best friend lied to them.",
        "answer": "angry",
        "distractors": ["happy", "proud", "relieved"]
    },
    {
        "scenario": "{name} walked into a room and everyone yelled 'Surprise!'",
        "answer": "surprised",
        "distractors": ["angry", "disgusted", "sad"]
    },
    {
        "scenario": "{name} saw someone stealing from a charity donation box.",
        "answer": "disgusted",
        "distractors": ["proud", "happy", "relieved"]
    },
    {
        "scenario": "{name} accidentally called their teacher 'Mom' in front of the class.",
        "answer": "embarrassed",
        "distractors": ["proud", "happy", "scared"]
    },
    {
        "scenario": "{name}'s sibling got a brand new bike while they got nothing.",
        "answer": "jealous",
        "distractors": ["proud", "happy", "disgusted"]
    },
    {
        "scenario": "{name} finally finished a very difficult exam they worried about for weeks.",
        "answer": "relieved",
        "distractors": ["jealous", "disgusted", "angry"]
    },
    {
        "scenario": "{name} just got a puppy they had been wishing for all year.",
        "answer": "happy",
        "distractors": ["sad", "angry", "scared"]
    },
]

FALSE_BELIEF_TEMPLATES = [
    {
        "scenario": "{name} puts their {obj} in the {loc1} and leaves. Someone moves it to the {loc2}. {name} comes back wanting their {obj}. How will {name} feel when they look in the {loc1} first?",
        "answer": "surprised or confused",
        "distractors": ["happy they found it", "angry at themselves", "proud of looking"]
    },
    {
        "scenario": "{name} thinks their friend is {loc1}, but the friend actually went to {loc2}. Where will {name} look first?",
        "answer": "The {loc1} (where they believe the friend is)",
        "distractors": ["The {loc2} (actual location)", "Nowhere", "Both places at once"]
    },
    {
        "scenario": "{name} sees a candy box but inside there are pencils. Before opening it, what does {name} think is inside?",
        "answer": "Candy",
        "distractors": ["Pencils", "Nothing", "Rocks"]
    },
    {
        "scenario": "{name} watches mom put cookies in the jar. Mom leaves and dad eats some. When mom returns, how many cookies does she think are in the jar?",
        "answer": "The original number (she doesn't know dad ate some)",
        "distractors": ["Zero", "The reduced number", "Double the original"]
    },
]

DESIRE_TEMPLATES = [
    {
        "scenario": "{name} really wants a {obj} for their birthday. They receive exactly that.",
        "answer": "happy and satisfied",
        "distractors": ["disappointed", "angry", "confused"]
    },
    {
        "scenario": "{name} wants to go to the {loc1} but it's closed. They end up going to the {loc2} instead, which they don't enjoy.",
        "answer": "frustrated and disappointed",
        "distractors": ["thrilled", "proud", "relieved"]
    },
    {
        "scenario": "{name} didn't want to go to the party. They were forced to go but ended up having a great time.",
        "answer": "pleasantly surprised",
        "distractors": ["still angry", "sad", "scared"]
    },
    {
        "scenario": "{name} wanted to win the race. They came in second place.",
        "answer": "disappointed but slightly proud",
        "distractors": ["completely devastated", "angry at others", "disgusted"]
    },
]

MIXED_EMOTION_TEMPLATES = [
    {
        "scenario": "{name} is moving to a new city. They're excited about new opportunities but will miss their old friends.",
        "answer": "a mix of happiness and sadness",
        "distractors": ["purely happy", "purely sad", "no feelings at all"]
    },
    {
        "scenario": "{name}'s team won the championship, but {name} got injured during the final game.",
        "answer": "proud of the team but worried about the injury",
        "distractors": ["only happy", "only sad", "completely angry"]
    },
    {
        "scenario": "{name} graduated top of their class, but their grandmother couldn't attend the ceremony.",
        "answer": "proud but also sad",
        "distractors": ["only happy", "only angry", "only scared"]
    },
    {
        "scenario": "{name} finally stood up to a bully (scary but empowering).",
        "answer": "brave but also frightened",
        "distractors": ["only scared", "only happy", "only disgusted"]
    },
]

REGULATION_TEMPLATES = [
    {
        "scenario": "{name} is very angry after losing a game. Which strategy is most helpful?",
        "answer": "Taking deep breaths and counting to ten",
        "distractors": ["Yelling at the opponent", "Quitting all games forever", "Breaking the game pieces"]
    },
    {
        "scenario": "{name} feels nervous before a big presentation. What should they do?",
        "answer": "Practice deep breathing and visualize success",
        "distractors": ["Skip the presentation", "Eat a lot of junk food", "Tell everyone they're scared"]
    },
    {
        "scenario": "{name} is sad because their pet ran away. What's a healthy way to cope?",
        "answer": "Talk to a trusted friend about their feelings",
        "distractors": ["Never talk about it", "Pretend it doesn't matter", "Get angry at everyone"]
    },
    {
        "scenario": "{name} feels overwhelmed with homework. What's the best approach?",
        "answer": "Break it into small tasks and take breaks",
        "distractors": ["Do nothing", "Stay up all night panicking", "Copy from a friend"]
    },
]


def generate_emotion_attribution(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(EMOTION_ATTRIBUTION_TEMPLATES)
        name = random.choice(NAMES)
        question = tmpl["scenario"].format(name=name)
        row = format_mc_question(
            generate_qid("tefb", "emotion_attr", i + 1),
            question,
            tmpl["answer"],
            tmpl["distractors"],
        )
        rows.append(row)
    return rows


def generate_false_belief(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(FALSE_BELIEF_TEMPLATES)
        name = random.choice(NAMES)
        obj = random.choice(OBJECTS)
        locs = random.sample(LOCATIONS, 2)
        question = tmpl["scenario"].format(name=name, obj=obj, loc1=locs[0], loc2=locs[1])
        row = format_mc_question(
            generate_qid("tefb", "false_belief", i + 1),
            question,
            tmpl["answer"],
            tmpl["distractors"],
        )
        rows.append(row)
    return rows


def generate_desire_based(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(DESIRE_TEMPLATES)
        name = random.choice(NAMES)
        obj = random.choice(OBJECTS)
        locs = random.sample(LOCATIONS, 2)
        question = tmpl["scenario"].format(name=name, obj=obj, loc1=locs[0], loc2=locs[1])
        row = format_mc_question(
            generate_qid("tefb", "desire", i + 1),
            question,
            tmpl["answer"],
            tmpl["distractors"],
        )
        rows.append(row)
    return rows


def generate_mixed_emotions(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(MIXED_EMOTION_TEMPLATES)
        name = random.choice(NAMES)
        question = tmpl["scenario"].format(name=name)
        row = format_mc_question(
            generate_qid("tefb", "mixed", i + 1),
            question,
            tmpl["answer"],
            tmpl["distractors"],
        )
        rows.append(row)
    return rows


def generate_emotion_regulation(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(REGULATION_TEMPLATES)
        name = random.choice(NAMES)
        question = tmpl["scenario"].format(name=name)
        row = format_mc_question(
            generate_qid("tefb", "regulation", i + 1),
            question,
            tmpl["answer"],
            tmpl["distractors"],
        )
        rows.append(row)
    return rows


def main():
    set_seed(SEED)
    stats = {"total": 0, "by_type": {}, "by_answer": {"A": 0, "B": 0, "C": 0, "D": 0}}

    generators = [
        ("emotion_attribution", generate_emotion_attribution),
        ("false_belief", generate_false_belief),
        ("desire_based", generate_desire_based),
        ("mixed_emotions", generate_mixed_emotions),
        ("emotion_regulation", generate_emotion_regulation),
    ]

    with CSVWriter(OUTPUT_CSV) as writer:
        for name, gen_fn in generators:
            rows = gen_fn(QUESTIONS_PER_TYPE)
            writer.write_rows(rows)
            stats["by_type"][name] = len(rows)
            stats["total"] += len(rows)
            for row in rows:
                stats["by_answer"][row["answer"]] += 1

    print_summary("TEFB MC Dataset Generation Complete", OUTPUT_CSV, stats)


if __name__ == "__main__":
    main()
