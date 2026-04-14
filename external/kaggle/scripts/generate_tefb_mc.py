#!/usr/bin/env python3
"""
Generate TEFB (Trinity Executive Function & Behavior) Multiple Choice format.

Creates NEW MC questions from templates for executive function tasks:
- Multi-step planning and task execution
- Working memory operations and manipulation
- Inhibitory control (Stroop-like tasks)
- Cognitive flexibility (Wisconsin Card Sorting)
- Conflict resolution and rule switching
"""

import random
from pathlib import Path
from typing import List, Dict, Any, Tuple
import sys

# Add parent directory to path for utils
sys.path.insert(0, str(Path(__file__).parent))

from mc_generator_utils import (
    CSVWriter, DistractorGenerator, generate_qid, format_mc_question,
    print_summary, set_seed
)

# Configuration
OUTPUT_CSV = Path(__file__).parent.parent / "data" / "tefb_mc_new.csv"
QUESTIONS_PER_TYPE = 480
SEED = 42

# Data pools
COLORS = ["red", "blue", "green", "yellow", "purple", "orange", "pink", "brown", "black", "white"]
SHAPES = ["circle", "square", "triangle", "diamond", "star", "heart", "cross", "oval"]
NUMBERS = list(range(1, 11))
ANIMALS = ["cat", "dog", "bird", "fish", "horse", "cow", "pig", "sheep", "chicken", "rabbit"]
OBJECTS = ["key", "book", "pen", "phone", "wallet", "bag", "cup", "plate", "lamp", "chair"]
LOCATIONS = ["kitchen", "bedroom", "bathroom", "living room", "garage", "office", "park", "store"]
ACTIONS = ["running", "walking", "jumping", "sleeping", "eating", "reading", "swimming", "flying"]

# Planning templates
PLANNING_TEMPLATES = [
    {
        "task": "You need to bake a cake for a party at 5 PM. It takes 30 minutes to prepare, 45 minutes to bake, and 20 minutes to cool. You also need 15 minutes to frost it. What time should you start preparing?",
        "correct": "2:50 PM — 30 + 45 + 20 + 15 = 110 minutes, and 5:00 PM minus 110 minutes is 2:50 PM",
        "distractors": [
            "3:10 PM — you forgot the cooling time",
            "3:30 PM — you only counted prep and bake time",
            "2:00 PM — you added extra buffer time",
        ]
    },
    {
        "task": "You have 3 tasks: Task A takes 20 minutes, Task B takes 35 minutes, Task C takes 15 minutes. You can do them in any order, but Task B must be completed before Task C. What's the minimum total time?",
        "correct": "70 minutes — the order doesn't affect total time since tasks are sequential",
        "distractors": [
            "85 minutes — you're adding an unnecessary transition time",
            "50 minutes — you're incorrectly subtracting overlapping time",
            "60 minutes — you're miscounting one of the tasks",
        ]
    },
    {
        "task": "You're traveling from City A to City C. The trip requires a 2-hour train to City B, then a 30-minute layover, then a 1.5-hour bus to City C. If the train leaves at 8 AM, what time do you arrive in City C?",
        "correct": "11:50 AM — 8:00 + 2:00 + 0:30 + 1:30 = 11:50",
        "distractors": [
            "11:20 AM — you forgot the layover",
            "12:20 PM — you added extra time",
            "11:00 AM — you miscalculated one segment",
        ]
    },
]

# Working memory templates
MEMORY_TEMPLATES = [
    {
        "data": "Remember this sequence: 7, 3, 9, 2, 5, 8, 1, 4",
        "operation": "What is the sum of the 2nd and 6th numbers?",
        "correct": "11 — 2nd number is 3, 6th is 8, and 3 + 8 = 11",
        "distractors": [
            "9 — you used 1st and 5th numbers",
            "10 — you used 3rd and 7th numbers",
            "12 — you added incorrectly",
        ]
    },
    {
        "data": "Store these items in order: apple, chair, book, lamp, table",
        "operation": "Which item comes between 'book' and 'table' in the original sequence?",
        "correct": "lamp — the sequence is apple, chair, book, lamp, table",
        "distractors": [
            "chair — you're thinking of position",
            "book — there's no item between book and table",
            "apple — you're recalling from the wrong end",
        ]
    },
    {
        "data": "Remember these facts: Paris is in France, Tokyo is in Japan, Berlin is in Germany, Sydney is in Australia",
        "operation": "Which country was listed second alphabetically among the countries mentioned?",
        "correct": "France — countries are France, Japan, Germany, Australia, so France is second alphabetically (after Australia)",
        "distractors": [
            "Germany — you're listing in presentation order",
            "Japan — you're going by city order",
            "Australia — it's first alphabetically",
        ]
    },
]

# Stroop/inhibitory control templates
STROOP_TEMPLATES = [
    {
        "stimulus": "The word RED is printed in blue ink. The word BLUE is printed in red ink.",
        "instruction": "Name the INK COLOR of each word, not read the words.",
        "target": "What is the ink color of the word 'RED'?",
        "correct": "Blue — you must name the ink color, not read the word",
        "distractors": [
            "Red — you read the word instead of naming the ink color",
            "Both red and blue — you're confused about the task",
            "Neither — the colors are mixed",
        ]
    },
    {
        "stimulus": "You see the word 'UP' printed pointing downward, and the word 'DOWN' printed pointing upward.",
        "instruction": "Indicate the DIRECTION each word is pointing, ignore the word meaning.",
        "target": "Which direction is the word 'UP' pointing?",
        "correct": "Downward — you follow the visual direction, not the word meaning",
        "distractors": [
            "Upward — you're reading the word meaning",
            "Both directions — the word has both characteristics",
            "Sideways — you're misperceiving the orientation",
        ]
    },
    {
        "stimulus": "You hear the command 'STOP' but see a green light.",
        "instruction": "Follow the visual signal, not the auditory command.",
        "target": "What should you do?",
        "correct": "Go — green light means go, regardless of the word 'STOP'",
        "distractors": [
            "Stop — the command 'STOP' overrides the signal",
            "Wait for clarification — you're conflicted by the mismatch",
            "Do both — stop then go immediately",
        ]
    },
]

# Wisconsin Card Sorting/cognitive flexibility templates
WISCO_TEMPLATES = [
    {
        "setup": "You're sorting cards. The rule changes after you get 6 correct. Current rule: sort by COLOR (red cards go left, blue cards go right). You've sorted 4 correctly by color.",
        "new_card": "A red star card",
        "question": "Where do you put this card?",
        "correct": "Left — the rule is still color, and you've only had 4 correct (rule changes after 6)",
        "distractors": [
            "Right — you think the rule has changed prematurely",
            "It depends on the shape — you're trying a new rule",
            "You can't know without more information",
        ]
    },
    {
        "setup": "Sorting rule was COLOR. After 6 correct, you start getting feedback 'Wrong'. A card with RED SQUARE goes left (was correct) but now gets 'Wrong'. A BLUE CIRCLE goes right (was correct) and gets 'Correct'.",
        "new_card": "A red circle card",
        "question": "Where do you put this card and what's the likely new rule?",
        "correct": "Right — the rule likely changed to SHAPE (circles go right, squares go left, based on the feedback pattern)",
        "distractors": [
            "Left — stick with the old rule despite feedback",
            "Right — the rule changed to NUMBER, but there are no numbers",
            "You need more trials to determine the new rule",
        ]
    },
    {
        "setup": "Rule history: COLOR (6 correct) → SHAPE (6 correct) → NUMBER (6 correct) → now getting 'Wrong' again. Last correct was NUMBER (odd=left, even=right).",
        "new_card": "Card with the number 7",
        "question": "Where do you put it, and what are you testing?",
        "correct": "Left initially (7 is odd) — if wrong, test another rule like SHAPE or COLOR, maintaining cognitive flexibility",
        "distractors": [
            "Right — assume the rule inverted (even=left, odd=right)",
            "Left and stay there — the rule hasn't actually changed",
            "Randomly choose — all rules have been exhausted",
        ]
    },
]

# Conflict resolution templates
CONFLICT_TEMPLATES = [
    {
        "instructions": "Rule 1: If the shape is a circle, press LEFT. Rule 2: If the color is blue, press RIGHT. Both rules apply simultaneously.",
        "input": "A blue circle appears",
        "question": "What do you do?",
        "correct": "This is a conflict — both rules apply with opposite actions. Realistically, you'd need a precedence rule or clarification",
        "distractors": [
            "Press LEFT — Rule 1 takes precedence",
            "Press RIGHT — Rule 2 takes precedence",
            "Press both simultaneously",
        ]
    },
    {
        "instructions": "When you see an odd number, say 'odd'. When you see a number greater than 5, say 'high'. If both apply, say both words in order.",
        "input": "The number 7 appears",
        "question": "What do you say?",
        "correct": "'odd high' — 7 is both odd and greater than 5, so say both in the specified order",
        "distractors": [
            "'high odd' — you reversed the order",
            "'odd' or 'high' — you only said one",
            "'seven' — you said the number itself",
        ]
    },
    {
        "instructions": "Turn left at the second intersection unless there's a stop sign, in which case turn right. If there's a yield sign, go straight.",
        "input": "At the second intersection, there's both a stop sign and a yield sign",
        "question": "What do you do?",
        "correct": "This instruction is ambiguous — it doesn't specify what to do when multiple signs are present",
        "distractors": [
            "Turn right — stop sign instruction takes priority",
            "Go straight — yield sign instruction takes priority",
            "Turn left — neither sign affects the base instruction",
        ]
    },
]


def generate_plan_question(num: int) -> Dict[str, Any]:
    """Generate a multi-step planning question."""
    # Generate diverse planning scenarios
    scenario_types = ["time", "sequence", "resource", "route"]
    scenario_type = scenario_types[num % len(scenario_types)]

    if scenario_type == "time":
        # Time-based planning
        tasks = [
            (random.randint(15, 60), random.choice(["cooking", "cleaning", "exercising", "reading"])),
            (random.randint(20, 45), random.choice(["showering", "dressing", "eating", "commuting"])),
            (random.randint(10, 30), random.choice(["packing", "checking email", "feeding pets", "watering plants"])),
        ]
        total_minutes = sum(t[0] for t in tasks)
        start_hour = random.randint(6, 10)
        start_min = random.choice([0, 15, 30, 45])

        # Calculate end time
        end_total = start_hour * 60 + start_min + total_minutes
        end_hour = (end_total // 60) % 24
        end_min = end_total % 60

        question = f"""You need to complete these tasks in order:
1. {tasks[0][1]} ({tasks[0][0]} minutes)
2. {tasks[1][1]} ({tasks[1][0]} minutes)
3. {tasks[2][1]} ({tasks[2][0]} minutes)

If you start at {start_hour}:{start_min:02d} AM, what time will you finish?"""

        correct = f"{end_hour}:{end_min:02d} AM — total time is {total_minutes} minutes"

        # Distractors with plausible errors
        distractors = [
            f"{end_hour}:{(end_min + 15) % 60:02d} AM — added 15 minutes for transitions",
            f"{(end_hour - 1) % 12}:{end_min:02d} AM — miscalculated total time",
            f"{end_hour}:{(end_min - 10) % 60:02d} AM — subtracted time incorrectly",
        ]

    elif scenario_type == "sequence":
        # Sequence planning
        items = random.sample(OBJECTS, 4)
        constraints = [
            f"{items[0]} must come before {items[1]}",
            f"{items[2]} cannot be first",
        ]

        question = f"""You need to arrange these items in a specific order: {', '.join(items)}

Constraints:
- {constraints[0]}
- {constraints[1]}

Which of the following is a valid arrangement?"""

        # Generate a valid and invalid arrangements
        valid1 = f"{items[2]}, {items[0]}, {items[1]}, {items[3]}"
        valid2 = f"{items[3]}, {items[0]}, {items[2]}, {items[1]}"
        invalid1 = f"{items[1]}, {items[0]}, {items[2]}, {items[3]}"  # violates first constraint
        invalid2 = f"{items[2]}, {items[3]}, {items[1]}, {items[0]}"  # violates first constraint

        correct = valid1
        distractors = [invalid1, invalid2, valid2]

    elif scenario_type == "resource":
        # Resource allocation
        budget = random.randint(100, 500)
        costs = [
            (random.choice(["Item A", "Item B", "Item C"]), random.randint(20, 80)),
            (random.choice(["Item D", "Item E", "Item F"]), random.randint(30, 90)),
            (random.choice(["Item G", "Item H", "Item I"]), random.randint(25, 75)),
        ]
        total_cost = sum(c[1] for c in costs)

        question = f"""You have a budget of ${budget} and need to purchase:
- {costs[0][0]}: ${costs[0][1]}
- {costs[1][0]}: ${costs[1][1]}
- {costs[2][0]}: ${costs[2][1]}

Can you afford all items, and if not, what's the shortfall?"""

        if total_cost <= budget:
            correct = f"Yes — total cost is ${total_cost}, which is within your ${budget} budget"
            distractors = [
                f"No — you're ${budget - total_cost} short",
                f"No — you miscalculated and think it costs more",
                f"It depends on available discounts",
            ]
        else:
            shortfall = total_cost - budget
            correct = f"No — total cost is ${total_cost}, which is ${shortfall} over your ${budget} budget"
            distractors = [
                f"Yes — you have enough money",
                f"No — you're ${shortfall + 20} short (miscalculated)",
                f"Yes — if you negotiate prices down",
            ]

    else:  # route
        # Route planning
        segments = [
            (random.choice(["walk", "drive", "bus"]), random.randint(5, 25)),
            (random.choice(["walk", "drive", "bus"]), random.randint(10, 30)),
            (random.choice(["walk", "drive", "bus"]), random.randint(5, 20)),
        ]
        total_time = sum(s[1] for s in segments)

        question = f"""Your journey has these segments:
1. {segments[0][0]} for {segments[0][1]} minutes
2. {segments[1][0]} for {segments[1][1]} minutes
3. {segments[2][0]} for {segments[2][1]} minutes

What is the total travel time?"""

        correct = f"{total_time} minutes — {total_time // 60} hour{'s' if total_time // 60 != 1 else ''} and {total_time % 60} minutes"
        distractors = [
            f"{total_time + 10} minutes — added buffer time",
            f"{total_time - 5} minutes — missed one segment",
            f"{total_time + 15} minutes — double-counted transitions",
        ]

    qid = generate_qid("tefb", "plan", num, 4)
    return format_mc_question(qid, question, correct, distractors)


def generate_memory_question(num: int) -> Dict[str, Any]:
    """Generate a working memory question."""
    memory_types = ["sequence", "calculation", "manipulation"]
    memory_type = memory_types[num % len(memory_types)]

    if memory_type == "sequence":
        # Sequence memory
        items = random.sample(COLORS + ANIMALS + NUMBERS, 6)
        items_str = ", ".join(str(i) for i in items)

        # Ask about a specific position
        pos = random.randint(1, 6)
        answer = items[pos - 1]

        question = f"""Remember this sequence: {items_str}

What is the {pos}{'st' if pos == 1 else 'nd' if pos == 2 else 'rd' if pos == 3 else 'th'} item in the sequence?"""

        # Distractors: wrong positions, similar items
        wrong_pos1 = items[(pos) % 6]
        wrong_pos2 = items[(pos + 1) % 6]
        similar = random.choice([i for i in items if i != answer])

        correct = str(answer)
        distractors = [str(wrong_pos1), str(wrong_pos2), str(similar)]

    elif memory_type == "calculation":
        # Working memory calculation
        numbers = [random.randint(1, 20) for _ in range(5)]
        nums_str = ", ".join(str(n) for n in numbers)

        # Ask for calculation on specific positions
        pos1, pos2 = random.sample(range(5), 2)
        operations = ["sum", "difference", "product"]
        op = random.choice(operations)

        if op == "sum":
            answer = numbers[pos1] + numbers[pos2]
            question = f"""Remember these numbers: {nums_str}

What is the sum of the {pos1 + 1}{'st' if pos1 == 0 else 'nd' if pos1 == 1 else 'rd' if pos1 == 2 else 'th'} and {pos2 + 1}{'st' if pos2 == 0 else 'nd' if pos2 == 1 else 'rd' if pos2 == 2 else 'th'} numbers?"""
            distractors = [
                str(answer + random.randint(1, 5)),
                str(answer - random.randint(1, 5)),
                str(numbers[pos1] * numbers[pos2]),
            ]
        elif op == "difference":
            answer = abs(numbers[pos1] - numbers[pos2])
            question = f"""Remember these numbers: {nums_str}

What is the difference between the {pos1 + 1}{'st' if pos1 == 0 else 'nd' if pos1 == 1 else 'rd' if pos1 == 2 else 'th'} and {pos2 + 1}{'st' if pos2 == 0 else 'nd' if pos2 == 1 else 'rd' if pos2 == 2 else 'th'} numbers?"""
            distractors = [
                str(answer + random.randint(1, 5)),
                str(numbers[pos1] + numbers[pos2]),
                str(abs(answer - random.randint(1, 5))),
            ]
        else:  # product
            answer = numbers[pos1] * numbers[pos2]
            question = f"""Remember these numbers: {nums_str}

What is the product of the {pos1 + 1}{'st' if pos1 == 0 else 'nd' if pos1 == 1 else 'rd' if pos1 == 2 else 'th'} and {pos2 + 1}{'st' if pos2 == 0 else 'nd' if pos2 == 1 else 'rd' if pos2 == 2 else 'th'} numbers?"""
            distractors = [
                str(answer + numbers[pos1]),
                str(numbers[pos1] + numbers[pos2]),
                str(answer // 2 if answer > 10 else answer * 2),
            ]

        correct = str(answer)

    else:  # manipulation
        # Memory manipulation
        items = random.sample(OBJECTS, 4)
        locations = random.sample(LOCATIONS, 4)

        # Create associations
        pairs = list(zip(items, locations))
        pairs_str = "\n".join([f"- {item} is in the {location}" for item, location in pairs])

        # Ask about reverse association
        target_loc = random.choice(locations)
        target_item = next(item for item, loc in pairs if loc == target_loc)

        question = f"""Remember these item locations:
{pairs_str}

Which item is in the {target_loc}?"""

        correct = target_item
        wrong_items = [item for item, loc in pairs if loc != target_loc]
        distractors = wrong_items[:3]

    qid = generate_qid("tefb", "memory", num, 4)
    return format_mc_question(qid, question, correct, distractors)


def generate_stroop_question(num: int) -> Dict[str, Any]:
    """Generate an inhibitory control/Stroop question."""
    # Create word-color mismatches
    word_colors = list(zip(COLORS[:4], COLORS[4:8]))

    if num % 3 == 0:
        # Classic Stroop: word meaning vs ink color
        word, ink = random.choice(word_colors)
        question = f"""The word '{word.upper()}' is printed in {ink} ink.

Your task: Name the INK COLOR, do not read the word.

What is the correct response?"""

        correct = ink
        distractors = [word, random.choice([c for c in COLORS if c != ink and c != word]), "Both colors"]

    elif num % 3 == 1:
        # Directional Stroop
        directions = ["UP", "DOWN", "LEFT", "RIGHT"]
        actual_dirs = ["downward", "upward", "rightward", "leftward"]

        word_dir = random.choice(directions)
        actual_dir = actual_dirs[directions.index(word_dir)]

        question = f"""The word '{word_dir}' is printed pointing {actual_dir}.

Your task: Name the DIRECTION the word is pointing, ignore the word meaning.

What is the correct response?"""

        correct = actual_dir
        distractors = [word_dir.lower(), random.choice([d for d in actual_dirs if d != actual_dir]), "Both directions"]

    else:  # num % 3 == 2
        # Symbol-meaning conflict
        symbols = [
            ("✓ (checkmark)", "wrong / reject"),
            ("✗ (X mark)", "correct / accept"),
            ("→ (arrow right)", "go left"),
            ("← (arrow left)", "go right"),
        ]
        symbol, meaning = random.choice(symbols)

        question = f"""You see the symbol: {symbol}

Your task: Interpret it based on these special rules:
- A checkmark means 'wrong' or 'reject'
- An X mark means 'correct' or 'accept'
- Right arrow means 'go left'
- Left arrow means 'go right'

What does this symbol mean under these rules?"""

        correct = meaning
        distractors = [
            meaning.replace("wrong", "correct").replace("reject", "accept").replace("left", "right"),
            "The symbol has no meaning under these rules",
            "Both interpretations are valid",
        ]

    qid = generate_qid("tefb", "stroop", num, 4)
    return format_mc_question(qid, question, correct, distractors)


def generate_wisco_question(num: int) -> Dict[str, Any]:
    """Generate a Wisconsin Card Sorting/cognitive flexibility question."""
    # Define sorting dimensions
    dimensions = {
        "color": ["red", "blue", "green", "yellow"],
        "shape": ["circle", "square", "triangle", "star"],
        "number": ["one", "two", "three", "four"],
    }

    # Current rule state
    current_rule = random.choice(list(dimensions.keys()))
    correct_count = random.randint(1, 5)

    # Create a test card
    test_color = random.choice(dimensions["color"])
    test_shape = random.choice(dimensions["shape"])
    test_number = random.choice(dimensions["number"])

    # Target pile based on current rule
    if current_rule == "color":
        target = test_color
        rule_desc = f"sorry by COLOR ({test_color} cards go to the {test_color} pile)"
        correct_response = f"The {test_color} pile"
        wrong_response1 = f"The {test_shape} pile"
        wrong_response2 = f"The {test_number} pile"
    elif current_rule == "shape":
        target = test_shape
        rule_desc = f"sort by SHAPE ({test_shape} cards go to the {test_shape} pile)"
        correct_response = f"The {test_shape} pile"
        wrong_response1 = f"The {test_color} pile"
        wrong_response2 = f"The {test_number} pile"
    else:  # number
        target = test_number
        rule_desc = f"sort by NUMBER ({test_number} cards go to the {test_number} pile)"
        correct_response = f"The {test_number} pile"
        wrong_response1 = f"The {test_color} pile"
        wrong_response2 = f"The {test_shape} pile"

    if correct_count < 6:
        # Rule hasn't changed yet
        question = f"""Current rule: {rule_desc}
You've made {correct_count} correct sorts so far (rule changes after 6 correct).

Test card: {test_color} {test_shape} with {test_number} symbol(s)

Where do you sort this card?"""

        correct = correct_response
        distractors = [wrong_response1, wrong_response2,
                      f"The rule may have changed — need to test a different dimension"]

    else:
        # Rule just changed
        new_rule = random.choice([d for d in dimensions.keys() if d != current_rule])

        if new_rule == "color":
            new_target = test_color
            new_response = f"The {test_color} pile"
        elif new_rule == "shape":
            new_target = test_shape
            new_response = f"The {test_shape} pile"
        else:
            new_target = test_number
            new_response = f"The {test_number} pile"

        question = f"""You were sorting by {current_rule.upper()} and got 6 correct. Now you're getting 'Wrong' feedback.

Previous rule: {rule_desc}

Test card: {test_color} {test_shape} with {test_number} symbol(s)

After getting 'Wrong', you try sorting by a different dimension. Which dimension are you testing, and where does the card go based on that new rule?"""

        correct = f"Testing {new_rule.upper()} — {new_response}"
        distractors = [
            f"Continue with {current_rule.upper()} — {correct_response}",
            f"Testing a third dimension — {wrong_response1}",
            f"Give up — the pattern is unclear",
        ]

    qid = generate_qid("tefb", "wisco", num, 4)
    return format_mc_question(qid, question, correct, distractors)


def generate_conflict_question(num: int) -> Dict[str, Any]:
    """Generate a conflict resolution question."""
    conflict_types = ["rule_conflict", "instruction_ambiguity", "competing_goals"]
    conflict_type = conflict_types[num % len(conflict_types)]

    if conflict_type == "rule_conflict":
        # Conflicting rules
        rule1 = f"If the item is {random.choice(COLORS)}, select it"
        rule2 = f"If the item is a {random.choice(SHAPES)}, do NOT select it"

        item_color = random.choice(COLORS)
        item_shape = random.choice(SHAPES)

        question = f"""You have these two rules:
1. {rule1}
2. {rule2}

An item appears: It is {item_color} and it is a {item_shape}.

Both rules apply but give opposite instructions. What do you do?"""

        correct = "This is a rule conflict — you need a precedence rule or clarification to resolve it"
        distractors = [
            f"Select it — Rule 1 takes precedence",
            f"Do not select it — Rule 2 takes precedence",
            f"Select and deselect it — follow both in sequence",
        ]

    elif conflict_type == "instruction_ambiguity":
        # Ambiguous instructions
        question = f"""You receive these instructions:
'Go to the second door on the left, then turn right. If you see a red sign, go back to the start.'

You follow the instructions and see: a blue sign on your right, and a red sign on your left (visible but not in your path).

What do you do?"""

        correct = "The instruction is ambiguous — it's unclear if 'see a red sign' means in your path or anywhere visible"
        distractors = [
            "Go back to the start — you see a red sign",
            "Continue forward — the red sign isn't in your path",
            "Turn left toward the red sign — investigate further",
        ]

    else:  # competing_goals
        # Competing goals
        goal1 = "Finish your work assignment (due tomorrow)"
        goal2 = "Help your friend who just had an emergency"

        question = f"""You have two competing goals:
1. {goal1}
2. {goal2}

You only have time for one. How do you approach this conflict?"""

        correct = "Assess urgency and impact — emergencies typically take precedence, but evaluate the actual severity"
        distractors = [
            "Always prioritize the work assignment — deadlines are non-negotiable",
            "Always help your friend — relationships matter more than work",
            "Split your time between both — trying to do everything often accomplishes nothing",
        ]

    qid = generate_qid("tefb", "conflict", num, 4)
    return format_mc_question(qid, question, correct, distractors)


def generate_all_questions() -> Tuple[List[Dict[str, Any]], Dict[str, Any]]:
    """Generate all TEFB MC questions."""
    questions = []
    question_type = "tefb"

    generators = {
        "plan": generate_plan_question,
        "memory": generate_memory_question,
        "stroop": generate_stroop_question,
        "wisco": generate_wisco_question,
        "conflict": generate_conflict_question,
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
    """Generate TEFB MC dataset."""
    set_seed(SEED)

    print(f"{'='*60}")
    print("TEFB MC Generation")
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
    print_summary("TEFB MC Generation Summary", OUTPUT_CSV, stats)


if __name__ == "__main__":
    main()
