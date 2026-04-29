#!/usr/bin/env python3
"""
Generate TSCP (Theory of Social Cognition Probes) Multiple Choice format.

Creates MC questions probing social cognition and theory of mind:
- Perspective Taking: Visual / informational perspective differences
- Social Norms: Norm appropriateness judgments
- Intentionality: Accidental vs intentional action classification
- Social Prediction: Predicting others' behavior from context
- Deception Detection: Identifying deceptive vs honest statements
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

OUTPUT_CSV = Path(__file__).parent.parent / "data" / "tscp_mc.csv"
QUESTIONS_PER_TYPE = 480
SEED = 42

NAMES = ["Alice", "Bob", "Carol", "Dave", "Eve", "Frank", "Grace", "Hank", "Ivy", "Jack"]
OBJECTS = ["red ball", "blue book", "green cup", "yellow pen", "purple hat"]
LOCATIONS = ["kitchen", "bedroom", "garden", "school", "park", "office", "library", "store"]

PERSPECTIVE_TEMPLATES = [
    {
        "scenario": "{name_a} is standing on one side of a car. {name_b} is standing on the other side. The car has a dent only on {name_b}'s side. Can {name_a} see the dent?",
        "answer": "No, it's on the opposite side",
        "distractors": ["Yes, it's obvious", "Only if they're tall enough", "Only with binoculars"]
    },
    {
        "scenario": "{name_a} has headphones on and can't hear {name_b} talking. {name_b} says 'The answer is 42'. What does {name_a} know about what {name_b} said?",
        "answer": "Nothing, they can't hear",
        "distractors": ["The answer is 42", "That someone is talking", "The first word only"]
    },
    {
        "scenario": "{name_a} looks into a box and sees a {obj}. {name_b} hasn't looked. What does {name_b} know about the box contents?",
        "answer": "They don't know what's inside",
        "distractors": ["They know it's a {obj}", "They know the box is empty", "They know it's something small"]
    },
    {
        "scenario": "{name_a} is looking at a picture that appears to show a vase. {name_b} sees two faces facing each other. They are looking at the same image. Why do they see different things?",
        "answer": "The image is ambiguous and can be seen both ways",
        "distractors": ["One of them needs glasses", "The picture changed", "They're looking at different images"]
    },
]

NORM_TEMPLATES = [
    {
        "scenario": "{name} starts singing loudly in a quiet library. Is this appropriate?",
        "answer": "No, it violates the social norm of being quiet in a library",
        "distractors": ["Yes, singing is always OK", "Only if the song is good", "Only if others join in"]
    },
    {
        "scenario": "{name} holds the door open for someone carrying heavy boxes. Is this socially appropriate?",
        "answer": "Yes, it's a kind and expected social gesture",
        "distractors": ["No, it's intrusive", "Only if asked first", "Only for friends"]
    },
    {
        "scenario": "{name} takes the last slice of pizza without asking anyone else at the table. Is this appropriate?",
        "answer": "No, it's polite to offer it to others first",
        "distractors": ["Yes, first come first served", "Only if they're very hungry", "Only if they paid for it"]
    },
    {
        "scenario": "{name} texts during a movie in a theater. Is this appropriate?",
        "answer": "No, the screen light disturbs others",
        "distractors": ["Yes, it's silent", "Only if the movie is boring", "Only brief messages"]
    },
    {
        "scenario": "{name} says 'thank you' when receiving a gift. Is this following social norms?",
        "answer": "Yes, expressing gratitude is expected",
        "distractors": ["No, it's unnecessary", "Only for expensive gifts", "Only from strangers"]
    },
]

INTENTIONALITY_TEMPLATES = [
    {
        "scenario": "{name} accidentally bumps into someone while walking and immediately says sorry. Was the bump intentional?",
        "answer": "No, it was accidental",
        "distractors": ["Yes, they wanted to bump them", "Partially intentional", "Cannot be determined"]
    },
    {
        "scenario": "{name} deliberately steps on someone's foot during an argument. Was this intentional?",
        "answer": "Yes, it was done on purpose during conflict",
        "distractors": ["No, it was an accident", "Only partially intentional", "Depends on the foot size"]
    },
    {
        "scenario": "{name} breaks a vase while cleaning. They didn't mean to but were being careless. Was this intentional?",
        "answer": "No, but it involved negligence",
        "distractors": ["Yes, they wanted to break it", "It was purely accidental with no fault", "It was planned"]
    },
    {
        "scenario": "{name} 'forgets' to invite their ex-friend to a party, even though they invited everyone else. Was the exclusion intentional?",
        "answer": "Yes, the selective forgetting suggests intent",
        "distractors": ["No, they genuinely forgot", "It was random chance", "The ex-friend wouldn't come anyway"]
    },
]

SOCIAL_PREDICTION_TEMPLATES = [
    {
        "scenario": "{name_a} always helps others with homework. {name_b} never shares notes. If {name_a} needs help, who is more likely to get help from others?",
        "answer": "{name_a}, because reciprocity from past helpful behavior",
        "distractors": ["{name_b}, because they keep their notes private", "Both equally", "Neither, people are selfish"]
    },
    {
        "scenario": "{name} has been practicing piano every day for a year. They have a recital tomorrow. How are they likely to perform?",
        "answer": "Well, consistent practice usually leads to good performance",
        "distractors": ["Poorly, they must be tired", "Average, practice doesn't help", "Terribly, stage fright always wins"]
    },
    {
        "scenario": "{name} ate a huge meal an hour ago. Someone offers them a big slice of cake. What is {name} most likely to do?",
        "answer": "Decline or eat only a small amount because they're full",
        "distractors": ["Eat the whole cake eagerly", "Eat exactly half", "Always say yes regardless of hunger"]
    },
    {
        "scenario": "Every time it rains, {name} carries an umbrella. Today is sunny with no clouds. Will {name} bring an umbrella?",
        "answer": "Unlikely, they respond to weather conditions",
        "distractors": ["Yes, they always carry one", "Yes, they're paranoid about rain", "It's impossible to predict"]
    },
]

DECEPTION_TEMPLATES = [
    {
        "scenario": "{name} says 'I love your haircut!' but rolls their eyes when the other person turns away. Is this statement honest?",
        "answer": "No, the eye roll contradicts the verbal statement",
        "distractors": ["Yes, they said it out loud", "Partially honest", "Cannot tell"]
    },
    {
        "scenario": "{name} finds a wallet and turns it in to the lost-and-found without keeping any money. They tell their friend 'I found a wallet and turned it in.' Is this honest?",
        "answer": "Yes, their statement matches their actions",
        "distractors": ["No, they probably kept something", "Partially honest", "Only honest if someone saw them"]
    },
    {
        "scenario": "A salesperson says 'This is the lowest price anywhere!' but {name} found it cheaper at two other stores. What should {name} conclude?",
        "answer": "The salesperson is being deceptive",
        "distractors": ["The salesperson is correct", "The other stores are lying", "All prices are the same really"]
    },
    {
        "scenario": "{name_a} tells {name_b} they're 'fine' but their voice is shaking and they're crying. Should {name_b} believe the words or the behavior?",
        "answer": "The behavior, non-verbal cues often reveal true feelings",
        "distractors": ["The words, people mean what they say", "Neither, ask a third person", "Both equally"]
    },
]


def fill_template(tmpl: Dict, name_a: str = None, name_b: str = None, obj: str = None) -> str:
    kwargs = {}
    if name_a:
        kwargs["name_a"] = name_a
    if name_b:
        kwargs["name_b"] = name_b
    if "{name}" in tmpl["scenario"]:
        kwargs["name"] = name_a or random.choice(NAMES)
    if obj:
        kwargs["obj"] = obj
    return tmpl["scenario"].format(**kwargs)


def generate_perspective(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(PERSPECTIVE_TEMPLATES)
        name_a, name_b = random.sample(NAMES, 2)
        obj = random.choice(OBJECTS)
        question = fill_template(tmpl, name_a, name_b, obj)
        rows.append(format_mc_question(
            generate_qid("tscp", "perspective", i + 1),
            question, tmpl["answer"], tmpl["distractors"],
        ))
    return rows


def generate_social_norms(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(NORM_TEMPLATES)
        name = random.choice(NAMES)
        question = tmpl["scenario"].format(name=name)
        rows.append(format_mc_question(
            generate_qid("tscp", "norms", i + 1),
            question, tmpl["answer"], tmpl["distractors"],
        ))
    return rows


def generate_intentionality(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(INTENTIONALITY_TEMPLATES)
        name = random.choice(NAMES)
        question = tmpl["scenario"].format(name=name)
        rows.append(format_mc_question(
            generate_qid("tscp", "intentionality", i + 1),
            question, tmpl["answer"], tmpl["distractors"],
        ))
    return rows


def generate_social_prediction(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(SOCIAL_PREDICTION_TEMPLATES)
        names = random.sample(NAMES, 2)
        question = tmpl["scenario"].format(name_a=names[0], name_b=names[1], name=names[0])
        answer = tmpl["answer"].format(name_a=names[0], name_b=names[1])
        distractors = [d.format(name_a=names[0], name_b=names[1]) for d in tmpl["distractors"]]
        rows.append(format_mc_question(
            generate_qid("tscp", "prediction", i + 1),
            question, answer, distractors,
        ))
    return rows


def generate_deception(num: int) -> List[Dict[str, Any]]:
    rows = []
    for i in range(num):
        tmpl = random.choice(DECEPTION_TEMPLATES)
        names = random.sample(NAMES, 2)
        question = tmpl["scenario"].format(name_a=names[0], name_b=names[1], name=names[0])
        rows.append(format_mc_question(
            generate_qid("tscp", "deception", i + 1),
            question, tmpl["answer"], tmpl["distractors"],
        ))
    return rows


def main():
    set_seed(SEED)
    stats = {"total": 0, "by_type": {}, "by_answer": {"A": 0, "B": 0, "C": 0, "D": 0}}

    generators = [
        ("perspective_taking", generate_perspective),
        ("social_norms", generate_social_norms),
        ("intentionality", generate_intentionality),
        ("social_prediction", generate_social_prediction),
        ("deception_detection", generate_deception),
    ]

    with CSVWriter(OUTPUT_CSV) as writer:
        for name, gen_fn in generators:
            rows = gen_fn(QUESTIONS_PER_TYPE)
            writer.write_rows(rows)
            stats["by_type"][name] = len(rows)
            stats["total"] += len(rows)
            for row in rows:
                stats["by_answer"][row["answer"]] += 1

    print_summary("TSCP MC Dataset Generation Complete", OUTPUT_CSV, stats)


if __name__ == "__main__":
    main()
