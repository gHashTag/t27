#!/usr/bin/env python3
"""
Shared utilities for MC question generation scripts.

Base classes and functions used by all Trinity Cognitive Probes MC generators.
"""

import csv
import random
import re
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import List, Tuple, Optional, Dict, Any, Iterator, ContextManager
from contextlib import contextmanager


@dataclass
class QuestionTemplate:
    """Template for a multiple choice question."""
    track: str              # e.g., "thlp", "ttm", "tscp", "tefb"
    qtype: str              # e.g., "belief", "calibration", "tom"
    question: str           # The question text
    correct_answer: str     # The correct answer
    distractors: List[str]  # 3 incorrect but plausible options
    metadata: Optional[Dict[str, Any]] = field(default_factory=dict)

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary for CSV writing."""
        return asdict(self)


class DistractorGenerator:
    """Generate and manage multiple choice distractors."""

    @staticmethod
    def shuffle_options(options: List[str], correct_index: int = 0) -> Tuple[List[str], str]:
        """
        Shuffle options and track correct answer position.

        Args:
            options: List of 4 options (correct + 3 distractors)
            correct_index: Index of correct answer in input list (default 0)

        Returns:
            Tuple of (shuffled_options, answer_letter)
        """
        if len(options) != 4:
            raise ValueError("Exactly 4 options required")

        # Keep track of correct answer
        correct_answer = options[correct_index]

        # Shuffle all options
        shuffled = options.copy()
        random.shuffle(shuffled)

        # Find new position of correct answer
        new_index = shuffled.index(correct_answer)
        answer_letter = chr(ord('A') + new_index)

        return shuffled, answer_letter

    @staticmethod
    def format_choices(options: List[str]) -> str:
        """
        Format options as A) X\nB) Y\nC) Z\nD) W.

        Args:
            options: List of 4 options

        Returns:
            Formatted choices string
        """
        if len(options) != 4:
            raise ValueError("Exactly 4 options required")

        letters = ["A", "B", "C", "D"]
        return "\n".join([f"{letter}) {opt}" for letter, opt in zip(letters, options)])

    @staticmethod
    def check_similarity(option1: str, option2: str, threshold: float = 0.7) -> float:
        """
        Check similarity between two options using simple character overlap.

        Args:
            option1: First option text
            option2: Second option text
            threshold: Similarity threshold to flag

        Returns:
            Similarity score (0-1)
        """
        # Simple character Jaccard-like similarity
        set1 = set(option1.lower().replace(" ", ""))
        set2 = set(option2.lower().replace(" ", ""))

        if not set1 or not set2:
            return 0.0

        intersection = len(set1 & set2)
        union = len(set1 | set2)

        return intersection / union if union > 0 else 0.0


class CSVWriter:
    """Context manager for writing MC CSV files with validation."""

    def __init__(self, output_path: Path, fieldnames: List[str] = None):
        """
        Initialize CSV writer.

        Args:
            output_path: Path to output CSV file
            fieldnames: Column names (defaults to MC format)
        """
        self.output_path = Path(output_path)
        self.fieldnames = fieldnames or ["id", "question_type", "question", "choices", "answer"]
        self._file = None
        self._writer = None
        self._count = 0

    def __enter__(self) -> 'CSVWriter':
        """Open file and create writer."""
        self.output_path.parent.mkdir(parents=True, exist_ok=True)
        self._file = open(self.output_path, 'w', encoding='utf-8', newline='')
        self._writer = csv.DictWriter(self._file, fieldnames=self.fieldnames)
        self._writer.writeheader()
        return self

    def write_row(self, row: Dict[str, Any]) -> None:
        """Write a single row with validation."""
        if not all(field in row for field in self.fieldnames):
            missing = [f for f in self.fieldnames if f not in row]
            raise ValueError(f"Missing fields: {missing}")

        # Validate answer is A, B, C, or D
        if row["answer"] not in ["A", "B", "C", "D"]:
            raise ValueError(f"Answer must be A-D, got: {row['answer']}")

        # Validate choices format
        choices = row["choices"]
        if not all(f"{letter})" in choices for letter in ["A", "B", "C", "D"]):
            raise ValueError(f"Choices must contain A), B), C), D)")

        self._writer.writerow(row)
        self._count += 1

    def write_rows(self, rows: List[Dict[str, Any]]) -> None:
        """Write multiple rows."""
        for row in rows:
            self.write_row(row)

    @property
    def count(self) -> int:
        """Number of rows written."""
        return self._count

    def __exit__(self, exc_type, exc_val, exc_tb):
        """Close file."""
        if self._file:
            self._file.close()


class QuestionValidator:
    """Validate question templates before generation."""

    @staticmethod
    def validate_template(template: QuestionTemplate) -> List[str]:
        """
        Validate a question template.

        Args:
            template: QuestionTemplate to validate

        Returns:
            List of validation errors (empty if valid)
        """
        errors = []

        # Check track name
        if not re.match(r'^(thlp|ttm|tscp|tefb|tagp)$', template.track):
            errors.append(f"Invalid track: {template.track}")

        # Check question type
        if not template.qtype or len(template.qtype) < 2:
            errors.append(f"Invalid qtype: {template.qtype}")

        # Check question text
        if not template.question or len(template.question.strip()) < 5:
            errors.append("Question text too short or empty")

        # Check correct answer
        if not template.correct_answer or len(template.correct_answer.strip()) < 1:
            errors.append("Correct answer missing")

        # Check distractors
        if len(template.distractors) != 3:
            errors.append(f"Expected 3 distractors, got {len(template.distractors)}")

        # Check for duplicate options
        all_options = [template.correct_answer] + template.distractors
        if len(set([opt.lower().strip() for opt in all_options])) != 4:
            errors.append("Duplicate options detected")

        # Check option similarity
        for i, opt1 in enumerate(all_options):
            for j, opt2 in enumerate(all_options[i+1:], i+1):
                similarity = DistractorGenerator.check_similarity(opt1, opt2)
                if similarity > 0.85:
                    errors.append(f"Options {i} and {j} too similar ({similarity:.2f})")

        return errors

    @staticmethod
    def validate_dataset(csv_path: Path) -> Dict[str, Any]:
        """
        Validate an existing MC dataset CSV.

        Args:
            csv_path: Path to CSV file

        Returns:
            Dictionary with validation results
        """
        results = {
            "valid": True,
            "errors": [],
            "stats": {
                "total": 0,
                "by_answer": {"A": 0, "B": 0, "C": 0, "D": 0},
                "by_type": {},
                "avg_question_length": 0,
            }
        }

        try:
            with open(csv_path, 'r', encoding='utf-8') as f:
                reader = csv.DictReader(f)

                for row in reader:
                    results["stats"]["total"] += 1

                    # Count by answer
                    answer = row.get("answer", "")
                    if answer in results["stats"]["by_answer"]:
                        results["stats"]["by_answer"][answer] += 1

                    # Count by question type
                    qid = row.get("id", "")
                    if "_" in qid:
                        qtype = "_".join(qid.split("_")[1:-1])
                        results["stats"]["by_type"][qtype] = results["stats"]["by_type"].get(qtype, 0) + 1

                    # Track question length
                    question = row.get("question", "")
                    results["stats"]["avg_question_length"] += len(question)

                    # Validate choices
                    choices = row.get("choices", "")
                    if not all(f"{letter})" in choices for letter in ["A", "B", "C", "D"]):
                        results["errors"].append(f"Row {qid}: Missing choice letter")

            # Calculate average
            if results["stats"]["total"] > 0:
                results["stats"]["avg_question_length"] /= results["stats"]["total"]

            # Check answer distribution
            answers = results["stats"]["by_answer"]
            total = results["stats"]["total"]
            expected = total / 4
            for letter, count in answers.items():
                deviation = abs(count - expected) / expected
                if deviation > 0.2:  # 20% deviation
                    results["errors"].append(
                        f"Answer distribution skewed: {letter} has {count}/{total} ({count/total:.1%})"
                    )

            if results["errors"]:
                results["valid"] = False

        except Exception as e:
            results["valid"] = False
            results["errors"].append(f"Failed to read CSV: {e}")

        return results


def generate_qid(track: str, qtype: str, num: int, total_digits: int = 4) -> str:
    """
    Generate a question ID.

    Args:
        track: Track name (e.g., "thlp")
        qtype: Question type (e.g., "belief")
        num: Question number
        total_digits: Number of digits for padding

    Returns:
        Formatted question ID like "thlp_belief_0123"
    """
    return f"{track}_{qtype}_{num:0{total_digits}d}"


def format_mc_question(
    qid: str,
    question: str,
    correct_answer: str,
    distractors: List[str],
    shuffle: bool = True
) -> Dict[str, str]:
    """
    Format a question as MC dictionary.

    Args:
        qid: Question ID
        question: Question text
        correct_answer: Correct answer
        distractors: List of 3 distractors
        shuffle: Whether to shuffle answer position

    Returns:
        Dictionary with MC format keys
    """
    options = [correct_answer] + distractors

    if shuffle:
        shuffled, answer_letter = DistractorGenerator.shuffle_options(options)
    else:
        shuffled, answer_letter = options, "A"

    choices = DistractorGenerator.format_choices(shuffled)

    return {
        "id": qid,
        "question_type": "mc",
        "question": question,
        "choices": choices,
        "answer": answer_letter
    }


def load_word_lists(base_path: Path) -> Dict[str, List[str]]:
    """
    Load word lists from a base path if they exist.

    Args:
        base_path: Path to look for word list files

    Returns:
        Dictionary of word lists by category
    """
    word_lists = {
        "nouns": [],
        "verbs": [],
        "adjectives": [],
        "colors": [],
        "animals": [],
        "professions": [],
        "objects": [],
    }

    # Default word lists if no files found
    word_lists["nouns"] = ["cat", "dog", "bird", "fish", "tree", "house", "car", "book", "table", "chair"]
    word_lists["verbs"] = ["run", "jump", "eat", "sleep", "read", "write", "speak", "listen", "watch", "think"]
    word_lists["adjectives"] = ["big", "small", "fast", "slow", "happy", "sad", "hot", "cold", "new", "old"]
    word_lists["colors"] = ["red", "blue", "green", "yellow", "purple", "orange", "black", "white"]
    word_lists["animals"] = ["cat", "dog", "bird", "fish", "horse", "cow", "pig", "sheep"]
    word_lists["professions"] = ["doctor", "teacher", "engineer", "artist", "chef", "lawyer", "pilot", "nurse"]
    word_lists["objects"] = ["key", "book", "pen", "phone", "wallet", "bag", "cup", "plate"]

    return word_lists


def get_random_item(items: List[str], exclude: List[str] = None) -> str:
    """
    Get a random item from a list, excluding certain values.

    Args:
        items: List to choose from
        exclude: Items to exclude from selection

    Returns:
        Random item not in exclude list
    """
    exclude = exclude or []
    available = [item for item in items if item not in exclude]
    return random.choice(available) if available else random.choice(items)


def print_summary(title: str, output_path: Path, stats: Dict[str, Any]) -> None:
    """
    Print a formatted summary of generation results.

    Args:
        title: Section title
        output_path: Path to output file
        stats: Statistics dictionary
    """
    print(f"\n{'='*60}")
    print(f"{title}")
    print(f"{'='*60}")
    print(f"Output: {output_path}")
    print(f"Total questions: {stats.get('total', 0)}")

    if "by_type" in stats and stats["by_type"]:
        print(f"\nBy question type:")
        for qtype, count in sorted(stats["by_type"].items()):
            print(f"  {qtype}: {count}")

    if "by_answer" in stats and stats["by_answer"]:
        print(f"\nAnswer distribution:")
        for letter, count in sorted(stats["by_answer"].items()):
            pct = count / stats["total"] * 100 if stats["total"] > 0 else 0
            print(f"  {letter}: {count} ({pct:.1f}%)")

    print(f"{'='*60}\n")


# Seed for reproducibility (can be overridden by callers)
DEFAULT_SEED = 42


def set_seed(seed: int = DEFAULT_SEED) -> None:
    """Set random seed for reproducibility."""
    random.seed(seed)
