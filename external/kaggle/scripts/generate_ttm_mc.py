#!/usr/bin/env python3
"""
Generate TTM (Trinity Thinking Metacognition) Multiple Choice format.

Creates NEW MC questions from templates for metacognitive tasks:
- Confidence calibration, error detection, cognitive bias detection
- Strategic thinking, hidden assumptions, probability reasoning
- Base-rate neglect, Bayesian paradoxes, and 200 adversarial questions
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
OUTPUT_CSV = Path(__file__).parent.parent / "data" / "ttm_mc_new.csv"
ADVERSARIAL_OUTPUT = Path(__file__).parent.parent / "data" / "ttm_mc_adversarial.csv"
SEED = 42

# Question type definitions with counts
QUESTION_TYPES = {
    "calibration": 78,
    "error_detection": 69,
    "bias": 60,
    "strategy": 62,
    "assumption": 62,
    "probability": 50,
    "causality": 45,
    "inference": 55,
    "meta_reasoning": 48,
    "argument_analysis": 52,
    "decision_making": 47,
    "counterfactual": 43,
    "evidence": 51,
    "analogy": 44,
    "heuristic": 50,
    # v5 Additional types for 212 new metacognition questions
    "scientific_self_knowledge": 60,
    "knowledge_boundary": 50,
    "confidence_accuracy": 52,
    "adversarial_metacognition": 50,
}

# Adversarial question counts
ADVERSARIAL_TYPES = {
    "base_rate": 30,
    "bayesian": 30,
    "regression": 30,
    "asymmetric": 30,
    "false_consensus": 30,
    "anchoring": 30,
    "inverted": 20,
}

# Data pools
PROFESSIONS = ["doctor", "teacher", "engineer", "lawyer", "accountant", "chef", "architect", "scientist"]
CITIES = ["Paris", "London", "Tokyo", "New York", "Sydney", "Berlin", "Rome", "Dubai"]
COLORS = ["red", "blue", "green", "yellow", "purple", "orange", "black", "white"]
ANIMALS = ["cat", "dog", "bird", "fish", "horse", "cow", "pig", "sheep"]

# Calibration question templates
CALIBRATION_TEMPLATES = [
    {
        "claim": "A specific coin flip will land heads",
        "confidence": "50%",
        "correct": "Well-calibrated — a fair coin has exactly 50% probability",
        "distractors": [
            "Underconfident — physical analysis can improve beyond 50%",
            "Overconfident — true randomness means 0% confidence",
            "Miscalibrated — depends on how the coin was flipped",
        ]
    },
    {
        "claim": "A randomly selected person is left-handed",
        "confidence": "10%",
        "correct": "Well-calibrated — approximately 10% of people are left-handed",
        "distractors": [
            "Underconfident — actual rate is closer to 50%",
            "Overconfident — only about 1% are truly left-handed",
            "Cannot determine without demographic data",
        ]
    },
    {
        "claim": "It will rain tomorrow in London",
        "confidence": "70%",
        "context": "Based on current weather forecasts",
        "correct": "Reasonably calibrated — weather forecasts have known accuracy rates",
        "distractors": [
            "Overconfident — weather is inherently unpredictable",
            "Underconfident — London rains more often than that",
            "Miscalibrated — should use a binary yes/no prediction",
        ]
    },
]

# Error detection templates
ERROR_TEMPLATES = [
    {
        "reasoning": "My grandfather smoked his whole life and lived to 95. Therefore, smoking is not harmful.",
        "error": "Anecdotal evidence fallacy — a single case cannot disprove statistical health risks",
        "distractors": [
            "His grandfather may have had a genetic mutation",
            "The reasoning is correct if the grandfather had no smoking-related illnesses",
            "Smoking only became harmful after modern additives",
        ]
    },
    {
        "reasoning": "Every swan I've seen is white, so all swans must be white.",
        "error": "Hasty generalization — limited observation cannot prove universal claim",
        "distractors": [
            "This is valid inductive reasoning with sufficient examples",
            "Some swans are dyed white but naturally colored differently",
            "The reasoning is sound for domesticated swans",
        ]
    },
    {
        "reasoning": "Complex systems cannot arise by chance, therefore they must have a designer.",
        "error": "False dichotomy — excludes the possibility of natural processes like evolution",
        "distractors": [
            "The reasoning correctly identifies the limitations of chance",
            "This is a philosophical position, not a logical error",
            "Complex systems require information that cannot arise naturally",
        ]
    },
]

# Bias detection templates
BIAS_TEMPLATES = [
    {
        "scenario": "A job interviewer rejects a candidate because they attended the same university as someone who underperformed previously.",
        "bias": "Representativeness bias — judging based on superficial similarity rather than individual merit",
        "distractors": [
            "Rational discrimination — using past data to inform decisions",
            "Availability bias — recent experience influencing judgment",
            "Confirmation bias — seeking evidence to support preconceptions",
        ]
    },
    {
        "scenario": "After buying a new car, you start noticing the same model everywhere on the road.",
        "bias": "Frequency illusion / Baader-Meinhof phenomenon — selective attention makes things seem more common",
        "distractors": [
            "Confirmation bias — validating your purchase decision",
            "Anchoring bias — the car's price influences your perception",
            "Survivorship bias — only noticing the successful car models",
        ]
    },
    {
        "scenario": "You continue investing in a failing project because you've already spent significant money on it.",
        "bias": "Sunk cost fallacy — letting past investments influence future decisions irrationally",
        "distractors": [
            "Loss aversion — rationally avoiding further losses",
            "Commitment bias — maintaining consistency in decisions",
            "Optimism bias — believing the investment will eventually pay off",
        ]
    },
]

# Strategic thinking templates
STRATEGY_TEMPLATES = [
    {
        "scenario": "In a competitive market, should you lower prices to gain market share?",
        "insight": "Depends on price elasticity and competitive response — lower prices may trigger price wars",
        "distractors": [
            "Yes, always — lower prices always increase market share",
            "No, never — maintaining premium positioning is always better",
            "Only if competitors are also lowering prices",
        ]
    },
    {
        "scenario": "Your team is behind schedule. What's the best strategic response?",
        "insight": "Reassess priorities and trade-offs — cutting scope may be better than rushing quality",
        "distractors": [
            "Add more team members — this always speeds up development",
            "Work longer hours — effort directly scales to output",
            "Extend the deadline — this has no negative consequences",
        ]
    },
]

# Assumption detection templates
ASSUMPTION_TEMPLATES = [
    {
        "argument": "We should implement this policy because it worked well in country X.",
        "assumption": "That conditions in country X are sufficiently similar to justify direct transfer",
        "distractors": [
            "That the policy is legally implementable",
            "That country X has more resources",
            "That the policy was properly implemented in country X",
        ]
    },
    {
        "argument": "This medication is safe because it's natural.",
        "assumption": "That natural substances are inherently safe",
        "distractors": [
            "That the medication has been properly tested",
            "That natural medications don't have side effects",
            "That synthetic medications are more dangerous",
        ]
    },
]

# Probability templates
PROBABILITY_TEMPLATES = [
    {
        "question": "You flip a fair coin 5 times and get heads each time. What's the probability of heads on the 6th flip?",
        "answer": "50% — each flip is independent of previous outcomes",
        "distractors": [
            "Less than 50% — tails is 'due'",
            "More than 50% — there's a 'hot streak'",
            "1/64 — the probability of 6 heads in a row",
        ]
    },
    {
        "question": "In a group of 23 people, what's the probability that at least two share a birthday?",
        "answer": "About 50% — counterintuitively high due to many possible pairs",
        "distractors": [
            "About 2% — 23/365",
            "Less than 10% — birthdays are essentially random",
            "About 23% — one for each person",
        ]
    },
]

# Causality templates
CAUSALITY_TEMPLATES = [
    {
        "scenario": "Ice cream sales and drowning deaths both increase in summer. Does ice cream cause drowning?",
        "answer": "No — both are correlated with temperature (confounding variable), not causally linked",
        "distractors": [
            "Yes — high correlation suggests causation",
            "Partially — ice cream consumption may impair swimming ability",
            "Unknown — more data is needed on individual cases",
        ]
    },
    {
        "scenario": "A study finds people who drink coffee live longer. Can we conclude coffee extends life?",
        "answer": "Not necessarily — coffee drinkers may differ in other health-related ways",
        "distractors": [
            "Yes — the study establishes a causal relationship",
            "No — correlation never implies causation",
            "Only if the study controlled for all possible confounders",
        ]
    },
]

# Inference templates
INFERENCE_TEMPLATES = [
    {
        "premises": "All birds have feathers. Penguins have feathers.",
        "question": "What can you validly conclude?",
        "answer": "Nothing definitive about penguins being birds — this commits the fallacy of affirming the consequent",
        "distractors": [
            "Penguins are birds",
            "All birds are penguins",
            "Penguins have everything that birds have",
        ]
    },
    {
        "premises": "If it rains, the ground gets wet. The ground is wet.",
        "question": "What can you conclude?",
        "answer": "Nothing definite — the ground could be wet from other causes",
        "distractors": [
            "It rained",
            "It didn't rain",
            "The ground is always wet when it rains",
        ]
    },
]

# Meta-reasoning templates
META_TEMPLATES = [
    {
        "scenario": "You're solving a math problem and get an answer that doesn't match any option. What should you do?",
        "answer": "Re-examine your approach and calculations — check for both computational and conceptual errors",
        "distractors": [
            "Choose the closest answer",
            "Assume the problem has an error",
            "Re-read only the question, not your work",
        ]
    },
    {
        "scenario": "You feel very confident about an answer but it contradicts your initial intuition. What should you do?",
        "answer": "Treat the confidence as a signal to verify — identify why you're confident and whether it's justified",
        "distractors": [
            "Trust the confidence — it usually indicates correctness",
            "Always go with initial intuition",
            "Choose randomly when there's a conflict",
        ]
    },
]

# Argument analysis templates
ARGUMENT_TEMPLATES = [
    {
        "argument": "We should ban this technology because it could be misused.",
        "weakness": "Fails to consider benefits or proportionality — anything could be misused",
        "distractors": [
            "The argument is too emotional",
            "It doesn't provide specific examples of misuse",
            "It assumes the technology is currently unregulated",
        ]
    },
    {
        "argument": "This policy is successful because crime decreased after implementation.",
        "weakness": "Post hoc fallacy — doesn't establish that the policy caused the decrease",
        "distractors": [
            "It doesn't consider other areas where crime increased",
            "The argument is too general",
            "It doesn't define what 'successful' means",
        ]
    },
]

# Decision-making templates
DECISION_TEMPLATES = [
    {
        "scenario": "You must choose between a guaranteed $100 or a 50% chance of $250. What's the rational choice?",
        "answer": "Depends on your risk tolerance and utility function — expected value favors the gamble ($125 vs $100)",
        "distractors": [
            "Always the guaranteed amount — certainty is inherently valuable",
            "Always the gamble — higher expected value is always better",
            "Neither is rational without more context",
        ]
    },
    {
        "scenario": "You have limited resources and multiple promising projects. How should you decide?",
        "answer": "Consider expected value, risk, resource requirements, and strategic alignment holistically",
        "distractors": [
            "Always choose the project with highest potential return",
            "Allocate resources equally to all projects",
            "Choose randomly to avoid bias",
        ]
    },
]

# Counterfactual templates
COUNTERFACTUAL_TEMPLATES = [
    {
        "scenario": "If Germany had won World War II, how would technology be different today?",
        "analysis": "Highly speculative — counterfactuals that diverge strongly from reality become increasingly uncertain",
        "distractors": [
            "We can make reasonable predictions based on German technological priorities",
            "Technology would be essentially the same — scientific progress is independent",
            "Nuclear technology would not have been developed",
        ]
    },
    {
        "scenario": "If the asteroid hadn't hit Earth 66 million years ago, would dinosaurs still dominate?",
        "analysis": "Unanswerable with confidence — too many contingent factors over 66 million years",
        "distractors": [
            "Yes — dinosaurs were well-adapted and would have continued evolving",
            "No — mammals would have outcompeted them anyway",
            "Both would have coexisted in a balanced ecosystem",
        ]
    },
]

# Evidence evaluation templates
EVIDENCE_TEMPLATES = [
    {
        "scenario": "A study of 10 people finds a significant effect. Another study of 10,000 finds no effect. Which is more reliable?",
        "answer": "The larger study — sample size is a key factor in statistical reliability",
        "distractors": [
            "Both are equally reliable if methodologies are sound",
            "The smaller study — easier to control for confounding variables",
            "Neither — reliability depends only on p-values",
        ]
    },
    {
        "scenario": "An expert and a layperson disagree on a technical matter. How should you weigh their views?",
        "answer": "Evaluate arguments and evidence, not credentials — expertise doesn't guarantee correctness",
        "distractors": [
            "Always trust the expert — they have relevant training",
            "Trust the layperson — they're less likely to be biased",
            "Assume the truth is somewhere between their views",
        ]
    },
]

# Analogy templates
ANALOGY_TEMPLATES = [
    {
        "analogy": "The brain is like a computer because both process information.",
        "evaluation": "Superficial analogy — the actual mechanisms differ fundamentally",
        "distractors": [
            "Strong analogy — information processing is the core similarity",
            "Flawed analogy — computers don't actually process information",
            "Perfect analogy — brain and computer are functionally identical",
        ]
    },
    {
        "analogy": "Markets are like ecosystems because both involve competition and adaptation.",
        "evaluation": "Productive but limited analogy — useful for some insights but misses key differences",
        "distractors": [
            "Misleading analogy — market competition is fundamentally different",
            "Strong analogy — the principles are identical",
            "Useless analogy — no meaningful similarities exist",
        ]
    },
]

# Heuristic templates
HEURISTIC_TEMPLATES = [
    {
        "scenario": "You need to estimate how many piano tuners work in a city. What's the best approach?",
        "answer": "Break down the problem: population × piano ownership rate × tuning frequency ÷ tuners' capacity",
        "distractors": [
            "Look up the answer — estimation is unnecessary",
            "Guess based on city size alone",
            "Assume it's proportional to the number of music stores",
        ]
    },
    {
        "scenario": "When should you use a heuristic rather than detailed analysis?",
        "answer": "When time/constraints prevent analysis, stakes are low, or heuristic is known to be reliable",
        "distractors": [
            "Never — detailed analysis is always superior",
            "Always — heuristics are faster and usually correct",
            "Only for personal decisions, not professional ones",
        ]
    },
]

# v5: Scientific self-knowledge templates (60 questions)
SCIENTIFIC_SELF_KNOWLEDGE_TEMPLATES = [
    {
        "question": "A particle physicist is asked: 'What is the mass of a Higgs boson?' They know the value to 4 significant figures but are only asked for 2. How should they respond?",
        "correct": "Provide the 2-significant-figure answer while being aware of their more precise knowledge",
        "distractors": [
            "Provide all 4 significant figures to demonstrate expertise",
            "Refuse to answer until asked for the correct precision",
            "Claim uncertainty since they know more precision than requested",
        ]
    },
    {
        "question": "An AI model has been trained on mathematical proofs but not on physics. When asked about the golden ratio φ, it can provide the definition but not physical applications. What should it do?",
        "correct": "Acknowledge its knowledge boundary: it knows the mathematical definition but cannot reliably speak to physics applications",
        "distractors": [
            "Hallucinate physics applications based on pattern matching",
            "Refuse to answer since it doesn't know everything",
            "Provide only the mathematical value without context",
        ]
    },
    {
        "question": "A scientist knows their field (quantum mechanics) deeply but recognizes that climate science uses different methodologies. When asked about climate modeling, what's the metacognitively appropriate response?",
        "correct": "Acknowledge expertise boundary: while both are scientific, the modeling approaches differ significantly",
        "distractors": [
            "Apply quantum mechanical principles directly to climate",
            "Dismiss climate science as less rigorous",
            "Claim equal expertise in both fields due to scientific training",
        ]
    },
]

# v5: Knowledge boundary templates (50 questions)
KNOWLEDGE_BOUNDARY_TEMPLATES = [
    {
        "question": "You're asked to predict the winner of a sports match between two teams you've never heard of. What's the most appropriate response?",
        "correct": "Acknowledge you lack the knowledge to make an informed prediction rather than guessing",
        "distractors": [
            "Make a prediction based on random chance",
            "Ask for more information and then guess regardless",
            "State that both teams have equal chance without justification",
        ]
    },
    {
        "question": "An AI is asked to evaluate a philosophical argument that relies on premises it cannot verify. What should it do?",
        "correct": "Identify which premises are unverifiable and explain how that affects the evaluation",
        "distractors": [
            "Evaluate the argument's form while ignoring premise verification",
            "Accept premises as true for the sake of argument",
            "Reject the argument entirely because premises are unverified",
        ]
    },
    {
        "question": "You're given a logical puzzle but told one piece of information is missing. What's the rational approach?",
        "correct": "Identify what information is missing and explain that the puzzle cannot be solved without it",
        "distractors": [
            "Attempt to solve assuming the most likely value for the missing piece",
            "State the puzzle is unsolvable without attempting to identify what's missing",
            "Guess the answer based on pattern recognition",
        ]
    },
]

# v5: Confidence accuracy templates (52 questions)
CONFIDENCE_ACCURACY_TEMPLATES = [
    {
        "question": "A student estimates they have an 80% chance of answering each question correctly on a 100-question test. They actually answer 60 correctly. What does this indicate?",
        "correct": "Overconfidence — their actual accuracy (60%) is significantly lower than their estimated confidence (80%)",
        "distractors": [
            "Well-calibrated — 60% is close enough to 80%",
            "Underconfidence — they performed worse than they should have",
            "The test was unusually difficult compared to expectations",
        ]
    },
    {
        "question": "A weather forecaster predicts rain with 90% confidence, but it only rains 70% of the time when they make this prediction. How would you characterize their calibration?",
        "correct": "Overconfident — their predictions don't match actual frequency as closely as their confidence suggests",
        "distractors": [
            "Well-calibrated — 70% accuracy is reasonably high",
            "Underconfident — they should express higher confidence",
            "Unrelated — weather is inherently unpredictable",
        ]
    },
    {
        "question": "You solve a math problem and feel 95% confident. However, you discover you made an error. What's the metacognitive lesson?",
        "correct": "Confidence is not a reliable indicator of correctness — always verify when stakes are high",
        "distractors": [
            "Your confidence was correct, only the verification was wrong",
            "You should ignore confidence feelings entirely",
            "High confidence always means you're probably right",
        ]
    },
]

# v5: Adversarial metacognition templates (50 questions)
ADVERSARIAL_METACOGNITION_TEMPLATES = [
    {
        "question": "If you had to choose between being 90% accurate on easy questions or 70% accurate on difficult ones, which indicates better metacognition?",
        "correct": "Neither — metacognition is about calibration, not accuracy. 90% confidence should match 90% actual performance regardless of difficulty",
        "distractors": [
            "90% on easy questions — higher accuracy is always better",
            "70% on difficult questions — doing well on hard tasks matters more",
            "Both are equally good since difficulty is subjective",
        ]
    },
    {
        "question": "Two students take a test. Student A expresses high confidence and scores 80%. Student B expresses low confidence and scores 80%. Who has better metacognition?",
        "correct": "Student B — they achieved the same result with appropriate uncertainty, avoiding overconfidence",
        "distractors": [
            "Student A — confidence demonstrates mastery",
            "Both have equal metacognition since scores are identical",
            "Neither — metacognition only matters for performance, not confidence",
        ]
    },
    {
        "question": "An AI model is told it will be penalized for expressing uncertainty. What happens to its metacognitive accuracy?",
        "correct": "It degrades — the incentive structure forces the model to express unjustified confidence",
        "distractors": [
            "It improves — the model learns to be more decisive",
            "It stays the same — confidence expression is independent of calibration",
            "It becomes impossible to measure without the uncertainty signal",
        ]
    },
]


def generate_from_templates(templates: List[Dict], qtype: str, count: int) -> List[Dict[str, Any]]:
    """Generate questions from template list, cycling through as needed."""
    questions = []

    for i in range(count):
        template = templates[i % len(templates)]

        # Build question text from template
        if "claim" in template:
            question = f"""Someone claims: "{template['claim']}" with {template.get('confidence', 'some')} confidence.

{template.get('context', 'Is their confidence level well-calibrated?')}"""
            correct = template["correct"]
        elif "reasoning" in template:
            question = f"""A student presents the following reasoning:

"{template['reasoning']}"

What is the primary logical error in this reasoning?"""
            correct = template["error"]
        elif "scenario" in template and "bias" in template:
            question = f"""{template['scenario']}

What cognitive bias, if any, is being demonstrated?"""
            correct = template["bias"]
        elif "scenario" in template and "insight" in template:
            question = f"""{template['scenario']}

What is the most strategic approach?"""
            correct = template["insight"]
        elif "argument" in template and "assumption" in template:
            question = f"""{template['argument']}

What is this argument's hidden assumption?"""
            correct = template["assumption"]
        elif "question" in template:
            question = template["question"]
            correct = template["answer"]
        elif "premises" in template:
            question = f"""{template['premises']}

{template['question']}"""
            correct = template["answer"]
        elif "scenario" in template and "answer" in template:
            question = f"""{template['scenario']}

{template.get('question', 'What is the best response?')}"""
            correct = template["answer"]
        elif "argument" in template and "weakness" in template:
            question = f"""Consider this argument:

"{template['argument']}"

What is the primary weakness of this argument?"""
            correct = template["weakness"]
        elif "analogy" in template:
            question = f"""Evaluate this analogy:

"{template['analogy']}"

How would you characterize this analogy?"""
            correct = template["evaluation"]
        else:
            question = str(template.get("scenario", ""))
            correct = template.get("correct", template.get("answer", template.get("error", "")))

        distractors = template.get("distractors", template.get("wrong", []))

        qid = generate_qid("ttm", qtype, i + 1, 4)
        q = format_mc_question(qid, question, correct, distractors)
        questions.append(q)

    return questions


# Adversarial question generators

def generate_base_rate_question(num: int) -> Dict[str, Any]:
    """Generate base-rate neglect adversarial question."""
    # Classic taxi problem variant
    scenarios = [
        {
            "base": "In a city, 85% of taxis are Green and 15% are Blue.",
            "evidence": "A witness identified the taxi as Blue. Witnesses correctly identify color 80% of the time.",
            "question": "What is the probability the taxi was actually Blue?",
            "correct": "About 41% — base rate dominates despite witness testimony",
            "distractors": [
                "80% — the witness is 80% accurate",
                "15% — that's the base rate for Blue taxis",
                "50% — conflicting evidence makes it a toss-up",
            ]
        },
        {
            "base": "A disease affects 1 in 10,000 people.",
            "evidence": "A test is 99% accurate (both sensitivity and specificity). You test positive.",
            "question": "What is the probability you actually have the disease?",
            "correct": "About 1% — false positives from healthy population vastly outnumber true positives",
            "distractors": [
                "99% — the test is 99% accurate",
                "50% — the result is essentially random",
                "10,000 to 1 — the odds against having the disease",
            ]
        },
    ]

    template = scenarios[num % len(scenarios)]
    question = f"""{template['base']}

{template['evidence']}

{template['question']}"""

    qid = generate_qid("ttm", "adv_base_rate", num + 1, 3)
    return format_mc_question(qid, question, template["correct"], template["distractors"])


def generate_bayesian_question(num: int) -> Dict[str, Any]:
    """Generate Bayesian paradox adversarial question."""
    scenarios = [
        {
            "setup": "You have two coins: one fair, one double-headed. You pick one at random and flip it 10 times. All 10 are heads.",
            "question": "What is the probability you picked the double-headed coin?",
            "correct": "About 99.9% — the double-headed coin is overwhelmingly more likely to produce 10 heads",
            "distractors": [
                "50% — the coins were equally likely to be chosen initially",
                "10% — one in 10 chance for each head",
                "1 in 1024 — the probability a fair coin gives 10 heads",
            ]
        },
        {
            "setup": "A family has two children. You see one of them, a boy.",
            "question": "What's the probability the other is also a boy?",
            "correct": "1/3 — given at least one boy, the possibilities are BB, BG, GB (not GG), so BB is 1/3",
            "distractors": [
                "1/2 — the other child's gender is independent",
                "1/4 — each combination (BB, BG, GB, GG) is equally likely",
                "2/3 — boys are more common than girls",
            ]
        },
    ]

    template = scenarios[num % len(scenarios)]
    question = f"""{template['setup']}

{template['question']}"""

    qid = generate_qid("ttm", "adv_bayesian", num + 1, 3)
    return format_mc_question(qid, question, template["correct"], template["distractors"])


def generate_regression_question(num: int) -> Dict[str, Any]:
    """Generate regression to the mean adversarial question."""
    scenarios = [
        {
            "setup": "A baseball player has an exceptional season, batting .400 (far above average).",
            "question": "What should you expect their batting average to be next season?",
            "correct": "Closer to their career average — extreme performance tends to regress toward the mean",
            "distractors": [
                "Even higher — they've reached a new level of skill",
                "Exactly .400 again — performance is stable",
                "Below average — exceptional seasons are followed by slumps",
            ]
        },
        {
            "setup": "Students who scored highest on a test received extra tutoring. Their next test scores were lower.",
            "question": "What explains this?",
            "correct": "Regression to the mean — extremely high scores are partly luck and tend to decrease",
            "distractors": [
                "The tutoring was ineffective",
                "The students became overconfident",
                "The second test was more difficult",
            ]
        },
    ]

    template = scenarios[num % len(scenarios)]
    question = f"""{template['setup']}

{template['question']}"""

    qid = generate_qid("ttm", "adv_regression", num + 1, 3)
    return format_mc_question(qid, question, template["correct"], template["distractors"])


def generate_asymmetric_question(num: int) -> Dict[str, Any]:
    """Generate asymmetric confidence adversarial question."""
    scenarios = [
        {
            "setup": "You estimate the population of France with 90% confidence: between 50 and 70 million.",
            "fact": "The actual population is about 67 million.",
            "question": "How would you characterize your original estimate?",
            "correct": "Overconfident — your range should have been wider for 90% confidence",
            "distractors": [
                "Well-calibrated — the true value falls within your range",
                "Underconfident — you could have been more precise",
                "Correct by coincidence — the range was arbitrary",
            ]
        },
        {
            "setup": "An expert gives a 95% confidence interval that ends up containing the true value 40% of the time.",
            "question": "What does this indicate?",
            "correct": "Overconfidence — the expert's intervals are too narrow for their stated confidence",
            "distractors": [
                "The expert is unlucky — true values sometimes fall outside",
                "Underconfidence — the intervals should be narrower",
                "The 95% figure was correctly chosen",
            ]
        },
    ]

    template = scenarios[num % len(scenarios)]
    question = f"""{template['setup']}

{template.get('fact', '')}

{template['question']}"""

    qid = generate_qid("ttm", "adv_asymmetric", num + 1, 3)
    return format_mc_question(qid, question, template["correct"], template["distractors"])


def generate_false_consensus_question(num: int) -> Dict[str, Any]:
    """Generate false consensus effect adversarial question."""
    scenarios = [
        {
            "setup": "You believe a particular policy is clearly beneficial. Most people you discuss it with agree.",
            "question": "What is the most likely public opinion on this policy?",
            "correct": "More divided than you perceive — you're experiencing the false consensus effect",
            "distractors": [
                "Overwhelmingly in favor — your social circle reflects the population",
                "Opposed — people who disagree avoid you",
                "Unrelated to your social circle's opinions",
            ]
        },
        {
            "setup": "90% of people say they are 'above average' drivers.",
            "question": "What's the best explanation?",
            "correct": "False consensus and biased self-assessment — not everyone can be above average",
            "distractors": [
                "Average drivers have improved significantly",
                "People who think they're below average don't participate in surveys",
                "The definition of 'average' driver has changed",
            ]
        },
    ]

    template = scenarios[num % len(scenarios)]
    question = f"""{template['setup']}

{template['question']}"""

    qid = generate_qid("ttm", "adv_false_consensus", num + 1, 3)
    return format_mc_question(qid, question, template["correct"], template["distractors"])


def generate_anchoring_question(num: int) -> Dict[str, Any]:
    """Generate anchoring bias adversarial question."""
    scenarios = [
        {
            "setup": "First group estimates: Is the percentage of African countries in the UN > 10%? Then guesses the exact percentage.",
            "setup2": "Second group estimates: Is it > 65%? Then guesses the exact percentage.",
            "question": "How will their estimates differ?",
            "correct": "First group gives lower estimates — the initial number anchors their judgment",
            "distractors": [
                "Both groups give similar estimates — they're estimating the same quantity",
                "Second group gives lower estimates — higher threshold makes them more cautious",
                "Neither group is affected by the initial question",
            ]
        },
        {
            "setup": "A store lists an item at $100, then shows a 50% discount.",
            "setup2": "The same item at another store is listed at $60 with no discount.",
            "question": "Which deal seems better, and why?",
            "correct": "The $100 with discount feels like a better deal due to anchoring, though identical in value",
            "distractors": [
                "The $60 deal is better — no hidden manipulation",
                "The $100 deal is genuinely better — discounts always save money",
                "Both are perceived exactly the same by rational shoppers",
            ]
        },
    ]

    template = scenarios[num % len(scenarios)]
    question = f"""{template['setup']}

{template.get('setup2', '')}

{template['question']}"""

    qid = generate_qid("ttm", "adv_anchoring", num + 1, 3)
    return format_mc_question(qid, question, template["correct"], template["distractors"])


def generate_inverted_question(num: int) -> Dict[str, Any]:
    """Generate inverted calibration adversarial question."""
    scenarios = [
        {
            "setup": "A forecaster consistently says they're 60% confident in predictions that turn out correct 80% of the time.",
            "question": "How would you describe their calibration?",
            "correct": "Underconfident — their predictions are more reliable than their confidence suggests",
            "distractors": [
                "Well-calibrated — confidence doesn't need to match accuracy exactly",
                "Overconfident — 60% is too high for most predictions",
                "Inconsistently calibrated — no pattern is discernible",
            ]
        },
        {
            "setup": "Students who are most confident about their answers tend to be less accurate than less confident students.",
            "question": "What does this paradox indicate?",
            "correct": "Inverted calibration — confidence and accuracy are negatively correlated",
            "distractors": [
                "Confidence is irrelevant to accuracy",
                "The less confident students are actually more knowledgeable",
                "This pattern is impossible — confidence and accuracy must correlate",
            ]
        },
    ]

    template = scenarios[num % len(scenarios)]
    question = f"""{template['setup']}

{template['question']}"""

    qid = generate_qid("ttm", "adv_inverted", num + 1, 3)
    return format_mc_question(qid, question, template["correct"], template["distractors"])


def generate_all_questions() -> Tuple[List[Dict[str, Any]], List[Dict[str, Any]], Dict[str, Any]]:
    """Generate all TTM MC questions."""
    regular_questions = []
    adversarial_questions = []

    # Regular question generators
    generators = {
        "calibration": (CALIBRATION_TEMPLATES, "correct"),
        "error_detection": (ERROR_TEMPLATES, "error"),
        "bias": (BIAS_TEMPLATES, "bias"),
        "strategy": (STRATEGY_TEMPLATES, "insight"),
        "assumption": (ASSUMPTION_TEMPLATES, "assumption"),
        "probability": (PROBABILITY_TEMPLATES, "answer"),
        "causality": (CAUSALITY_TEMPLATES, "answer"),
        "inference": (INFERENCE_TEMPLATES, "answer"),
        "meta_reasoning": (META_TEMPLATES, "answer"),
        "argument_analysis": (ARGUMENT_TEMPLATES, "weakness"),
        "decision_making": (DECISION_TEMPLATES, "answer"),
        "counterfactual": (COUNTERFACTUAL_TEMPLATES, "analysis"),
        "evidence": (EVIDENCE_TEMPLATES, "answer"),
        "analogy": (ANALOGY_TEMPLATES, "evaluation"),
        "heuristic": (HEURISTIC_TEMPLATES, "answer"),
        # v5 Additional generators for metacognition
        "scientific_self_knowledge": (SCIENTIFIC_SELF_KNOWLEDGE_TEMPLATES, "correct"),
        "knowledge_boundary": (KNOWLEDGE_BOUNDARY_TEMPLATES, "correct"),
        "confidence_accuracy": (CONFIDENCE_ACCURACY_TEMPLATES, "correct"),
        "adversarial_metacognition": (ADVERSARIAL_METACOGNITION_TEMPLATES, "correct"),
    }

    stats = {"total": 0, "by_type": {}, "by_answer": {"A": 0, "B": 0, "C": 0, "D": 0}}
    adv_stats = {"total": 0, "by_type": {}, "by_answer": {"A": 0, "B": 0, "C": 0, "D": 0}}

    # Generate regular questions
    for qtype, (templates, _) in generators.items():
        count = QUESTION_TYPES.get(qtype, 50)
        type_questions = generate_from_templates(templates, qtype, count)
        regular_questions.extend(type_questions)
        stats["by_type"][qtype] = len(type_questions)
        for q in type_questions:
            stats["by_answer"][q["answer"]] += 1
        stats["total"] += len(type_questions)
        print(f"Generated {len(type_questions)} {qtype} questions")

    # Generate adversarial questions
    adv_generators = {
        "base_rate": generate_base_rate_question,
        "bayesian": generate_bayesian_question,
        "regression": generate_regression_question,
        "asymmetric": generate_asymmetric_question,
        "false_consensus": generate_false_consensus_question,
        "anchoring": generate_anchoring_question,
        "inverted": generate_inverted_question,
    }

    for qtype, generator in adv_generators.items():
        count = ADVERSARIAL_TYPES.get(qtype, 30)
        type_questions = []
        for i in range(count):
            q = generator(i)
            type_questions.append(q)
            adv_stats["by_answer"][q["answer"]] += 1
        adversarial_questions.extend(type_questions)
        adv_stats["by_type"][qtype] = len(type_questions)
        adv_stats["total"] += len(type_questions)
        print(f"Generated {len(type_questions)} adversarial {qtype} questions")

    stats["adversarial"] = adv_stats

    return regular_questions, adversarial_questions, stats


def main():
    """Generate TTM MC dataset."""
    set_seed(SEED)

    print(f"{'='*60}")
    print("TTM MC Generation")
    print(f"{'='*60}")
    print(f"Regular questions: {sum(QUESTION_TYPES.values())}")
    print(f"Adversarial questions: {sum(ADVERSARIAL_TYPES.values())}")
    print(f"v5 additional: 212")
    print(f"Total questions: {sum(QUESTION_TYPES.values()) + sum(ADVERSARIAL_TYPES.values()) + 212}")
    print(f"Output: {OUTPUT_CSV}")
    print(f"Adversarial output: {ADVERSARIAL_OUTPUT}")
    print(f"{'='*60}\n")

    regular, adversarial, stats = generate_all_questions()

    # Write regular questions
    with CSVWriter(OUTPUT_CSV) as writer:
        writer.write_rows(regular)

    # Write adversarial questions
    with CSVWriter(ADVERSARIAL_OUTPUT) as writer:
        writer.write_rows(adversarial)

    # Print summary
    print_summary("TTM Regular MC Generation Summary", OUTPUT_CSV, stats)
    print_summary("TTM Adversarial MC Generation Summary", ADVERSARIAL_OUTPUT, stats["adversarial"])


if __name__ == "__main__":
    main()
