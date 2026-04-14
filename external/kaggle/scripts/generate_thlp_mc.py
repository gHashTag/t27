#!/usr/bin/env python3
"""
Generate THLP (Trinity Human Learning Probe) Multiple Choice format.

Creates NEW MC questions from templates for 5 learning and reasoning tasks:
- Belief Update: False belief + correction + query
- Few-Shot Learning: N examples showing rule + test case
- Error Correction: Misinformation + correction + query
- Reward Learning: Action + reward feedback + query
- Contextual Reasoning: Context + problem + query
"""

import random
from pathlib import Path
from typing import List, Dict, Tuple, Any
import sys

# Add parent directory to path for utils
sys.path.insert(0, str(Path(__file__).parent))

from mc_generator_utils import (
    CSVWriter, DistractorGenerator, generate_qid, format_mc_question,
    get_random_item, print_summary, set_seed
)

# Configuration
OUTPUT_CSV = Path(__file__).parent.parent / "data" / "thlp_mc_new.csv"
QUESTIONS_PER_TYPE = 480
SEED = 42

# Data pools for generation
COLORS = ["red", "blue", "green", "yellow", "purple", "orange", "pink", "brown", "black", "white"]
ANIMALS = ["cat", "dog", "bird", "fish", "horse", "cow", "pig", "sheep", "chicken", "rabbit"]
PROFESSIONS = ["doctor", "teacher", "engineer", "artist", "chef", "lawyer", "pilot", "nurse", "scientist", "writer"]
VEHICLES = ["car", "bike", "bus", "train", "plane", "boat", "truck", "scooter", "helicopter", "subway"]
FRUITS = ["apple", "banana", "orange", "grape", "strawberry", "watermelon", "mango", "peach", "pear", "kiwi"]
CITIES = ["Paris", "London", "Tokyo", "New York", "Sydney", "Berlin", "Rome", "Moscow", "Dubai", "Toronto"]
MUSICAL_INSTRUMENTS = ["piano", "guitar", "violin", "drums", "flute", "trumpet", "cello", "saxophone", "harp", "clarinet"]
SPORTS = ["soccer", "basketball", "tennis", "swimming", "running", "cycling", "golf", "baseball", "hockey", "volleyball"]

# Temperature facts (for belief update)
TEMPERATURE_FACTS = {
    "water boils": "100°C at sea level",
    "water freezes": "0°C",
    "body temperature": "37°C",
    "room temperature": "20-25°C",
    "fever": "38°C or higher",
}

# Physical facts (for belief update)
PHYSICAL_FACTS = {
    "Earth orbits": "the Sun",
    "Moon orbits": "the Earth",
    "gravity pulls": "downward toward Earth",
    "light travels": "faster than sound",
    "sound requires": "a medium like air",
}

# Word reversal patterns (for few-shot)
REVERSAL_PATTERNS = {
    "tac": "cat",
    "god": "dog",
    "drib": "bird",
    "hsif": "fish",
    "tse": "set",
    "nap": "pan",
    "pot": "top",
    "nwod": "down",
}

# Arithmetic patterns (for few-shot)
ARITHMETIC_PATTERNS = {
    "5": "10 (add 5)",
    "7": "14 (add 7)",
    "3": "6 (add 3)",
    "4": "8 (add 4)",
}

# Error correction scenarios
ERROR_SCENARIOS = [
    ("Water boils at 90°C", "Water boils at 100°C at sea level", "What temperature does water boil at?"),
    ("The Moon emits its own light", "The Moon reflects sunlight", "What is the source of the Moon's light?"),
    ("Heavier objects fall faster", "All objects fall at the same rate in a vacuum", "How do different weights fall?"),
    ("The Sun orbits Earth", "Earth orbits the Sun", "Which orbits which?"),
    ("We use 10% of our brains", "We use virtually all of our brain", "How much of the brain do we use?"),
    ("Goldfish have 3-second memory", "Goldfish can remember for months", "How long can goldfish remember?"),
    ("Sharks don't get cancer", "Sharks can get cancer", "Can sharks get cancer?"),
    ("Hair and nails keep growing after death", "They appear longer due to skin retraction", "Do hair/nails grow after death?"),
]

# Reward learning scenarios
REWARD_SCENARIOS = [
    ("You chose the blue door and found $100", "You received a large reward", "What should you do next?"),
    ("You pressed the red button and got shocked", "You received a negative outcome", "What should you avoid?"),
    ("You studied hard and got an A", "Your effort was rewarded", "What should you continue doing?"),
    ("You skipped practice and lost the game", "Inaction led to failure", "What should you do differently?"),
]

# Contextual reasoning scenarios
CONTEXT_SCENARIOS = [
    {
        "context": "Alice always takes the bus to work on rainy days.",
        "problem": "Today is Tuesday and it's raining heavily.",
        "query": "How is Alice most likely getting to work today?",
        "answer": "Taking the bus"
    },
    {
        "context": "The restaurant closes at 10 PM on weekdays and 11 PM on weekends.",
        "problem": "It's Saturday at 10:30 PM.",
        "query": "Can you still order food at the restaurant?",
        "answer": "Yes, it's open for 30 more minutes"
    },
    {
        "context": "Tom needs 8 hours of sleep to function well.",
        "problem": "Tom went to bed at 11 PM and needs to wake up at 6 AM.",
        "query": "How will Tom likely feel tomorrow?",
        "answer": "Tired and groggy (only 7 hours of sleep)"
    },
]


def generate_belief_question(num: int) -> Dict[str, Any]:
    """Generate a belief update question."""
    # Combine temperature and physical facts
    all_facts = list(TEMPERATURE_FACTS.items()) + list(PHYSICAL_FACTS.items())
    fact_key, fact_value = random.choice(all_facts)

    # Create false statement
    false_value = random.choice([
        fact_value.replace("100", "90"),
        fact_value.replace("0", "10"),
        fact_value.replace("Sun", "Moon"),
        fact_value.replace("Earth", "Sun"),
        fact_value.replace("faster", "slower"),
        fact_value + " (FALSE)",
    ])

    question = f"""Which best describes: {false_value}.

{fact_value}.

At what {fact_key}?"""

    correct_answer = fact_value
    distractors = [
        "Cannot determine from the information",
        "The first statement is correct",
        "Both statements could be true under different conditions",
    ]

    # Add one specific distractor based on fact type
    if "temperature" in fact_key or "°" in fact_value:
        distractors[0] = f"{random.choice(['95°C', '105°C', '98°C'])}"
    elif "orbit" in fact_key:
        distractors[0] = "They orbit each other in a binary system"
    elif "faster" in fact_value:
        distractors[0] = "Light and sound travel at the same speed in air"

    qid = generate_qid("thlp", "belief", num, 4)
    return format_mc_question(qid, question, correct_answer, distractors)


def generate_fewshot_question(num: int) -> Dict[str, Any]:
    """Generate a few-shot learning question."""
    pattern_type = random.choice(["reversal", "arithmetic", "pattern"])

    if pattern_type == "reversal":
        examples = random.sample(list(REVERSAL_PATTERNS.items()), 2)
        test_input = random.choice([k for k, _ in REVERSAL_PATTERNS.items()])
        test_output = REVERSAL_PATTERNS[test_input]

        examples_text = "\n".join([f"Input: {v} -> Output: {k}" for k, v in examples])
        # For test case, we need the original word (value) that needs to be reversed
        test_original = random.choice([v for v in REVERSAL_PATTERNS.values() if v not in [e[1] for e in examples]])
        test_correct = REVERSAL_PATTERNS.get(test_original, test_original[::-1])

        question = f"""Which best describes: Learn the rule from these examples and apply to the test case.

{examples_text}

Test: {test_original}"""

        # Distractors: wrong reversals, same word, random word
        all_words = list(REVERSAL_PATTERNS.values()) + list(REVERSAL_PATTERNS.keys())
        distractors = [
            test_original,  # Not reversed
            random.choice([w for w in all_words if w != test_correct and w != test_original]),
            random.choice([w for w in all_words if w != test_correct and w != test_original]),
        ]
        correct_answer = test_correct

        # Distractors: wrong reversals, same word, random word
        all_words = list(REVERSAL_PATTERNS.values()) + list(REVERSAL_PATTERNS.keys())
        distractors = [
            test_input,
            random.choice([w for w in all_words if w != test_output and w != test_input]),
            random.choice([w for w in all_words if w != test_output and w != test_input]),
        ]
        correct_answer = test_output

    elif pattern_type == "arithmetic":
        examples = random.sample(list(ARITHMETIC_PATTERNS.items()), 2)
        test_input = random.choice([k for k, _ in ARITHMETIC_PATTERNS.items()])
        test_output = ARITHMETIC_PATTERNS[test_input]

        examples_text = "\n".join([f"Input: {k} -> Output: {v}" for k, v in examples])
        question = f"""Which best describes: Learn the rule from these examples and apply to the test case.

{examples_text}

Test: {test_input}"""

        # Distractors: wrong arithmetic
        test_num = int(test_input)
        distractors = [
            f"{test_num + random.choice([3, 6, 9])} (add {random.choice([3, 6, 9])})",
            f"{test_num} (no change)",
            f"{test_num * 2} (multiply by 2)",
        ]
        correct_answer = test_output

    else:  # pattern matching
        # Color + animal = coloranimal
        color = random.choice(COLORS)
        animal = random.choice(ANIMALS)
        pattern_answer = f"{color}{animal}"

        question = f"""Which best describes: Learn the rule from these examples and apply to the test case.

Input: red cat -> Output: redcat
Input: blue dog -> Output: bluedog

Test: {color} {animal}"""

        distractors = [
            f"{animal}{color}",
            f"{color}-{animal}",
            f"{color} {animal}",
        ]
        correct_answer = pattern_answer

    qid = generate_qid("thlp", "fewshot", num, 4)
    return format_mc_question(qid, question, correct_answer, distractors)


def generate_error_question(num: int) -> Dict[str, Any]:
    """Generate an error correction question."""
    # Use predefined scenarios for quality
    scenario_idx = num % len(ERROR_SCENARIOS)
    false_statement, correction, query = ERROR_SCENARIOS[scenario_idx]

    question = f"""Which best describes: {false_statement}.

{correction}.

{query}"""

    correct_answer = correction

    # Generate plausible distractors
    if "temperature" in false_statement.lower():
        distractors = [
            f"{random.choice(['90°C', '95°C', '105°C'])} — at higher altitudes",
            "It depends on the altitude and pressure",
            "Both statements could be correct in different contexts",
        ]
    elif "moon" in false_statement.lower():
        distractors = [
            "The Moon absorbs and re-emits light from Earth",
            "The Moon produces light during lunar eclipses",
            "The Moon reflects light from stars",
        ]
    elif "fall" in false_statement.lower():
        distractors = [
            "Heavier objects fall significantly faster in practice",
            "Air resistance makes no difference to falling speed",
            "Only objects of the same material fall at the same rate",
        ]
    else:
        distractors = [
            "The first statement is correct",
            "Both statements have scientific merit",
            "More information is needed to determine accuracy",
        ]

    qid = generate_qid("thlp", "error", num, 4)
    return format_mc_question(qid, question, correct_answer, distractors)


def generate_reward_question(num: int) -> Dict[str, Any]:
    """Generate a reward learning question."""
    # Use predefined scenarios
    scenario_idx = num % len(REWARD_SCENARIOS)
    action, feedback, query = REWARD_SCENARIOS[scenario_idx]

    question = f"""Which best describes: {action}.

{feedback}.

{query}"""

    # Generate appropriate answer and distractors based on scenario
    if "$100" in action or "A" in action or "rewarded" in feedback.lower():
        correct_answer = "Repeat the same action"
        distractors = [
            "Try a completely different action",
            "Do the opposite of what worked before",
            "Choose randomly since outcomes are unpredictable",
        ]
    else:  # Negative feedback
        correct_answer = "Avoid that action"
        distractors = [
            "Repeat the action to see if outcome changes",
            "Increase the intensity of the action",
            "Try a similar action with minor variations",
        ]

    qid = generate_qid("thlp", "reward", num, 4)
    return format_mc_question(qid, question, correct_answer, distractors)


def generate_context_question(num: int) -> Dict[str, Any]:
    """Generate a contextual reasoning question."""
    # Use predefined scenarios, cycling through them
    scenario_idx = num % len(CONTEXT_SCENARIOS)
    scenario = CONTEXT_SCENARIOS[scenario_idx]

    question = f"""Which best describes: {scenario['context']}

{scenario['problem']}

{scenario['query']}"""

    correct_answer = scenario['answer']

    # Generate context-appropriate distractors
    if "bus" in scenario['context']:
        distractors = [
            "Driving her car",
            "Walking to work",
            "Working from home today",
        ]
    elif "restaurant" in scenario['context']:
        distractors = [
            "No, it closed 30 minutes ago",
            "No, it's closed on weekends",
            "Only takeout is available at this time",
        ]
    elif "sleep" in scenario['context'] or "Tom" in scenario['context']:
        distractors = [
            "Well-rested and energized",
            "Exactly as usual — sleep duration doesn't matter",
            "It depends on what Tom ate for dinner",
        ]
    else:
        distractors = [
            "Cannot determine from the given context",
            "The information provided is insufficient",
            "Multiple interpretations are possible",
        ]

    qid = generate_qid("thlp", "context", num, 4)
    return format_mc_question(qid, question, correct_answer, distractors)


def generate_all_questions() -> List[Dict[str, Any]]:
    """Generate all THLP MC questions."""
    questions = []
    question_type = "thlp"

    generators = {
        "belief": generate_belief_question,
        "fewshot": generate_fewshot_question,
        "error": generate_error_question,
        "reward": generate_reward_question,
        "context": generate_context_question,
    }

    stats = {"total": 0, "by_type": {}, "by_answer": {"A": 0, "B": 0, "C": 0, "D": 0}}

    for qtype, generator in generators.items():
        type_questions = []
        for i in range(QUESTIONS_PER_TYPE):
            q = generator(i + 1)
            type_questions.append(q)
            stats["by_answer"][q["answer"]] += 1

        questions.extend(type_questions)
        stats["by_type"][qtype] = len(type_questions)
        stats["total"] += len(type_questions)
        print(f"Generated {len(type_questions)} {qtype} questions")

    return questions, stats


def main():
    """Generate THLP MC dataset."""
    set_seed(SEED)

    print(f"{'='*60}")
    print("THLP MC Generation")
    print(f"{'='*60}")
    print(f"Questions per type: {QUESTIONS_PER_TYPE}")
    print(f"Total questions: {QUESTIONS_PER_TYPE * 5}")
    print(f"Output: {OUTPUT_CSV}")
    print(f"{'='*60}\n")

    questions, stats = generate_all_questions()

    # Write to CSV
    with CSVWriter(OUTPUT_CSV) as writer:
        writer.write_rows(questions)

    # Print summary
    print_summary("THLP MC Generation Summary", OUTPUT_CSV, stats)


if __name__ == "__main__":
    main()
