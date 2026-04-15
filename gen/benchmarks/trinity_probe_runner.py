#!/usr/bin/env python3
"""
Trinity Cognitive Probe Runner — Generated from spec.
Source: specs/benchmarks/trinity_cognitive_probe_runner.t27
spec_hash: sha256:e5432742efdee2df977d5856c4feddc25cd99e10d1f345f031c52f7f23563c3c

Benchmarks 5 cognitive tracks on live LLM models.
phi^2 + 1/phi^2 = 3 = TRINITY
"""

import csv
import json
import os
import random
import re
import sys
import time
from dataclasses import dataclass, field, asdict
from pathlib import Path
from typing import Optional

# --- Constants (from spec) ---

PHI = 1.618033988749895
PHI_INV = 0.618033988749895
TRINITY = 3.0
TRINITY_TOL = 1e-10

# Verify TRINITY invariant at import
_trinity_check = PHI * PHI + PHI_INV * PHI_INV
assert abs(_trinity_check - TRINITY) < TRINITY_TOL, (
    f"TRINITY invariant violated: phi^2 + phi^-2 = {_trinity_check}"
)

# --- Track definitions ---

TRACK_REGISTRY = {
    "thlp": {
        "slug": "trinity-cognitive-probes-thlp-mc",
        "csv_name": "thlp_mc_new.csv",
        "total_questions": 19681,
        "brain_zones": ["Hippocampus", "Entorhinal Cortex"],
        "domain": "Pattern Learning, Belief Update, Rule Induction",
    },
    "ttm": {
        "slug": "trinity-cognitive-probes-tmp-mc",
        "csv_name": "ttm_mc_v5.csv",
        "total_questions": 4931,
        "brain_zones": ["PCC", "dlPFC"],
        "domain": "Confidence Calibration, Error Detection, Meta-Learning",
    },
    "tagp": {
        "slug": "trinity-cognitive-probes-tagp-mc",
        "csv_name": "tagp_mc.csv",
        "total_questions": 17601,
        "brain_zones": ["Parietal Cortex", "FEF"],
        "domain": "Selective Filtering, Sustained Attention, Attention Shifting",
    },
    "tefb": {
        "slug": "trinity-cognitive-probes-tefb-mc",
        "csv_name": "tefb_mc_new.csv",
        "total_questions": 21081,
        "brain_zones": ["dlPFC", "ACC", "OFC"],
        "domain": "Multi-step Planning, Working Memory, Cognitive Flexibility",
    },
    "tscp": {
        "slug": "trinity-cognitive-probes-tscp-mc",
        "csv_name": "tscp_mc_v5.csv",
        "total_questions": 2839,
        "brain_zones": ["TPJ", "mPFC"],
        "domain": "Theory of Mind, Pragmatic Inference, Social Norms",
    },
}

# --- Data structures ---


@dataclass
class MCQuestion:
    id: str
    question_type: str
    question: str
    choices: str
    answer: str


@dataclass
class ModelResponse:
    model_id: str
    question_id: str
    predicted: str
    latency_ms: float
    raw_response: str
    format_valid: bool


@dataclass
class TrackResult:
    track: str
    model_id: str
    total: int
    correct: int
    accuracy: float
    format_valid_count: int
    format_validity: float
    latency_p50_ms: float
    latency_p95_ms: float
    latency_p99_ms: float


@dataclass
class BenchmarkSummary:
    sample_size: int
    models: list
    tracks: list
    results: list
    total_calls: int
    total_duration_ms: float
    timestamp: str


# --- CSV Loading ---


def load_track_csv(data_dir: str, track_key: str, sample_size: int) -> list[MCQuestion]:
    """Load CSV from local directory, sample random subset."""
    meta = TRACK_REGISTRY[track_key]
    csv_path = Path(data_dir) / meta["csv_name"]

    if not csv_path.exists():
        print(f"  WARNING: {csv_path} not found, skipping")
        return []

    questions = []
    with open(csv_path, "r", encoding="utf-8") as f:
        reader = csv.DictReader(f)
        for row in reader:
            # Normalize column order (some CSVs have different order)
            q = MCQuestion(
                id=row.get("id", ""),
                question_type=row.get("question_type", ""),
                question=row.get("question", ""),
                choices=row.get("choices", ""),
                answer=row.get("answer", ""),
            )
            # Only include MC questions with valid answers
            if q.answer.strip().upper() in ("A", "B", "C", "D"):
                questions.append(q)

    if not questions:
        print(f"  WARNING: No valid MC questions in {csv_path}")
        return []

    # Random sample
    if len(questions) > sample_size:
        random.seed(42)  # Reproducible
        questions = random.sample(questions, sample_size)

    print(f"  Loaded {len(questions)} questions from {track_key.upper()}")
    return questions


# --- Model API Calls ---

# System prompt for MC evaluation
MC_SYSTEM_PROMPT = """You are evaluating a multiple-choice cognitive assessment question.
Read the question and choices carefully.
Respond with ONLY the letter of the correct answer: A, B, C, or D.
Do not include any explanation, just the single letter."""


def parse_mc_answer(raw_response: str) -> tuple[str, bool]:
    """Extract A/B/C/D from model response."""
    text = raw_response.strip().upper()

    # Direct single letter
    if text in ("A", "B", "C", "D"):
        return text, True

    # "The answer is X" pattern
    match = re.search(r"(?:answer|correct)\s*(?:is|:)?\s*([ABCD])", text, re.IGNORECASE)
    if match:
        return match.group(1).upper(), True

    # First letter if it's A-D
    if text and text[0] in "ABCD":
        return text[0], True

    # Look for standalone A/B/C/D
    match = re.search(r"\b([ABCD])\b", text)
    if match:
        return match.group(1), True

    return "", False


def call_anthropic(question: str, choices: str, model: str, timeout_ms: int) -> ModelResponse:
    """Call Anthropic Claude API."""
    try:
        import anthropic

        client = anthropic.Anthropic()
        prompt = f"Question: {question}\n\nChoices:\n{choices}\n\nAnswer (single letter A/B/C/D):"

        start = time.time()
        response = client.messages.create(
            model=model,
            max_tokens=10,
            system=MC_SYSTEM_PROMPT,
            messages=[{"role": "user", "content": prompt}],
        )
        latency = (time.time() - start) * 1000
        raw = response.content[0].text.strip()
        predicted, valid = parse_mc_answer(raw)

        return ModelResponse(
            model_id=model,
            question_id="",
            predicted=predicted,
            latency_ms=latency,
            raw_response=raw,
            format_valid=valid,
        )
    except Exception as e:
        return ModelResponse(
            model_id=model,
            question_id="",
            predicted="",
            latency_ms=0,
            raw_response=f"ERROR: {e}",
            format_valid=False,
        )


def call_openai(question: str, choices: str, model: str, timeout_ms: int) -> ModelResponse:
    """Call OpenAI API."""
    try:
        import openai

        client = openai.OpenAI()
        prompt = f"Question: {question}\n\nChoices:\n{choices}\n\nAnswer (single letter A/B/C/D):"

        start = time.time()
        response = client.chat.completions.create(
            model=model,
            max_tokens=10,
            messages=[
                {"role": "system", "content": MC_SYSTEM_PROMPT},
                {"role": "user", "content": prompt},
            ],
        )
        latency = (time.time() - start) * 1000
        raw = response.choices[0].message.content.strip()
        predicted, valid = parse_mc_answer(raw)

        return ModelResponse(
            model_id=model,
            question_id="",
            predicted=predicted,
            latency_ms=latency,
            raw_response=raw,
            format_valid=valid,
        )
    except Exception as e:
        return ModelResponse(
            model_id=model,
            question_id="",
            predicted="",
            latency_ms=0,
            raw_response=f"ERROR: {e}",
            format_valid=False,
        )


def call_together(question: str, choices: str, model: str, timeout_ms: int) -> ModelResponse:
    """Call Together AI API (for Llama models)."""
    try:
        import openai

        client = openai.OpenAI(
            base_url="https://api.together.xyz/v1",
            api_key=os.environ.get("TOGETHER_API_KEY", ""),
        )
        prompt = f"Question: {question}\n\nChoices:\n{choices}\n\nAnswer (single letter A/B/C/D):"

        start = time.time()
        response = client.chat.completions.create(
            model=f"meta-llama/{model}",
            max_tokens=10,
            messages=[
                {"role": "system", "content": MC_SYSTEM_PROMPT},
                {"role": "user", "content": prompt},
            ],
        )
        latency = (time.time() - start) * 1000
        raw = response.choices[0].message.content.strip()
        predicted, valid = parse_mc_answer(raw)

        return ModelResponse(
            model_id=model,
            question_id="",
            predicted=predicted,
            latency_ms=latency,
            raw_response=raw,
            format_valid=valid,
        )
    except Exception as e:
        return ModelResponse(
            model_id=model,
            question_id="",
            predicted="",
            latency_ms=0,
            raw_response=f"ERROR: {e}",
            format_valid=False,
        )


def run_probe(question: MCQuestion, model_id: str, timeout_ms: int = 30000) -> ModelResponse:
    """Route to correct API based on model_id."""
    if "claude" in model_id:
        resp = call_anthropic(question.question, question.choices, model_id, timeout_ms)
    elif "gpt" in model_id:
        resp = call_openai(question.question, question.choices, model_id, timeout_ms)
    elif "llama" in model_id:
        resp = call_together(question.question, question.choices, model_id, timeout_ms)
    else:
        resp = ModelResponse(
            model_id=model_id,
            question_id=question.id,
            predicted="",
            latency_ms=0,
            raw_response=f"Unknown model: {model_id}",
            format_valid=False,
        )

    resp.question_id = question.id
    return resp


# --- Metrics ---


def compute_percentile(values: list[float], p: float) -> float:
    """Compute p-th percentile (0-100)."""
    if not values:
        return 0.0
    sorted_v = sorted(values)
    k = (len(sorted_v) - 1) * (p / 100.0)
    f_k = int(k)
    c_k = f_k + 1
    if c_k >= len(sorted_v):
        return sorted_v[-1]
    d = k - f_k
    return sorted_v[f_k] + d * (sorted_v[c_k] - sorted_v[f_k])


def run_track_benchmark(
    data_dir: str, track_key: str, model_id: str, sample_size: int, timeout_ms: int = 30000
) -> Optional[TrackResult]:
    """Run benchmark for one track + one model."""
    questions = load_track_csv(data_dir, track_key, sample_size)
    if not questions:
        return None

    responses = []
    for i, q in enumerate(questions):
        resp = run_probe(q, model_id, timeout_ms)
        responses.append((q, resp))
        if (i + 1) % 25 == 0:
            print(f"    [{track_key.upper()}:{model_id}] {i+1}/{len(questions)}")

    # Compute metrics
    correct = sum(1 for q, r in responses if r.predicted == q.answer.strip().upper())
    format_valid = sum(1 for _, r in responses if r.format_valid)
    latencies = [r.latency_ms for _, r in responses if r.latency_ms > 0]

    total = len(responses)
    return TrackResult(
        track=track_key,
        model_id=model_id,
        total=total,
        correct=correct,
        accuracy=correct / total if total > 0 else 0.0,
        format_valid_count=format_valid,
        format_validity=format_valid / total if total > 0 else 0.0,
        latency_p50_ms=compute_percentile(latencies, 50),
        latency_p95_ms=compute_percentile(latencies, 95),
        latency_p99_ms=compute_percentile(latencies, 99),
    )


# --- Main benchmark ---


def run_full_benchmark(
    data_dir: str,
    models: list[str],
    tracks: list[str],
    sample_size: int = 100,
    timeout_ms: int = 30000,
) -> BenchmarkSummary:
    """Run full benchmark across all tracks and models."""
    start_time = time.time()
    results = []

    for track_key in tracks:
        print(f"\n=== Track: {track_key.upper()} ===")
        for model_id in models:
            print(f"  Model: {model_id}")
            result = run_track_benchmark(data_dir, track_key, model_id, sample_size, timeout_ms)
            if result:
                results.append(result)
                print(
                    f"  -> Accuracy: {result.accuracy:.3f}, "
                    f"Format: {result.format_validity:.3f}, "
                    f"Latency p50: {result.latency_p50_ms:.0f}ms"
                )

    duration = (time.time() - start_time) * 1000

    summary = BenchmarkSummary(
        sample_size=sample_size,
        models=models,
        tracks=tracks,
        results=[asdict(r) for r in results],
        total_calls=sum(r.total for r in results),
        total_duration_ms=duration,
        timestamp=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
    )

    return summary


def print_results_table(summary: BenchmarkSummary):
    """Print formatted results table."""
    print("\n" + "=" * 80)
    print("TRINITY COGNITIVE PROBES — BENCHMARK RESULTS")
    print(f"phi^2 + 1/phi^2 = 3 = TRINITY")
    print("=" * 80)
    print(f"\nSample size: {summary.sample_size} per track")
    print(f"Total API calls: {summary.total_calls}")
    print(f"Total duration: {summary.total_duration_ms/1000:.1f}s")
    print(f"Timestamp: {summary.timestamp}")

    # Group by track
    print(f"\n{'Track':<8} {'Model':<35} {'Acc':>6} {'Fmt':>6} {'p50ms':>7} {'p95ms':>7}")
    print("-" * 75)

    for r in summary.results:
        print(
            f"{r['track'].upper():<8} "
            f"{r['model_id']:<35} "
            f"{r['accuracy']:>6.3f} "
            f"{r['format_validity']:>6.3f} "
            f"{r['latency_p50_ms']:>7.0f} "
            f"{r['latency_p95_ms']:>7.0f}"
        )

    print("-" * 75)
    print()


def save_results(summary: BenchmarkSummary, output_dir: str):
    """Save results as JSON."""
    os.makedirs(output_dir, exist_ok=True)
    path = Path(output_dir) / "benchmark_results.json"
    with open(path, "w") as f:
        json.dump(asdict(summary), f, indent=2)
    print(f"Results saved to {path}")


# --- CLI ---


def main():
    import argparse

    parser = argparse.ArgumentParser(
        description="Trinity Cognitive Probe Runner — Kaggle MC Benchmark"
    )
    parser.add_argument(
        "--data-dir", required=True, help="Directory containing track CSV files"
    )
    parser.add_argument(
        "--models",
        nargs="+",
        default=[
            "claude-3-5-sonnet-20241022",
            "gpt-4o-mini-2024-07-18",
            "llama-3.1-8b-instruct",
        ],
    )
    parser.add_argument(
        "--tracks",
        nargs="+",
        default=["thlp", "ttm", "tagp", "tefb", "tscp"],
    )
    parser.add_argument("--sample-size", type=int, default=100)
    parser.add_argument("--timeout-ms", type=int, default=30000)
    parser.add_argument("--output-dir", default="outputs/benchmark")
    parser.add_argument("--dry-run", action="store_true", help="Skip API calls, use mock data")

    args = parser.parse_args()

    print("=" * 70)
    print("TRINITY COGNITIVE PROBE RUNNER")
    print(f"phi^2 + 1/phi^2 = {PHI**2 + PHI_INV**2:.10f} = TRINITY")
    print("=" * 70)

    if args.dry_run:
        print("\n[DRY RUN MODE — using mock responses]\n")
        # Generate mock results for testing
        results = []
        for track in args.tracks:
            for model in args.models:
                results.append(
                    asdict(
                        TrackResult(
                            track=track,
                            model_id=model,
                            total=args.sample_size,
                            correct=int(args.sample_size * random.uniform(0.3, 0.85)),
                            accuracy=random.uniform(0.3, 0.85),
                            format_valid_count=args.sample_size,
                            format_validity=1.0,
                            latency_p50_ms=random.uniform(200, 1500),
                            latency_p95_ms=random.uniform(1500, 4000),
                            latency_p99_ms=random.uniform(4000, 8000),
                        )
                    )
                )

        summary = BenchmarkSummary(
            sample_size=args.sample_size,
            models=args.models,
            tracks=args.tracks,
            results=results,
            total_calls=len(args.tracks) * len(args.models) * args.sample_size,
            total_duration_ms=0,
            timestamp=time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        )
    else:
        summary = run_full_benchmark(
            data_dir=args.data_dir,
            models=args.models,
            tracks=args.tracks,
            sample_size=args.sample_size,
            timeout_ms=args.timeout_ms,
        )

    print_results_table(summary)
    save_results(summary, args.output_dir)

    return 0


if __name__ == "__main__":
    sys.exit(main())
