# Kaggle Datasets - Setup Guide for Real Model Benchmarking

## Overview

All 5 Trinity Cognitive Probes datasets are uploaded and fixed on Kaggle:
- [THLP](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-thlp-mc) - 19,680 questions
- [TTM](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tmp-mc) - 2,482 questions
- [TAGP](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tagp-mc) - 17,600 questions
- [TEFB](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tefb-mc) - 21,080 questions
- [TSCP](https://www.kaggle.com/datasets/playra/trinity-cognitive-probes-tscp-mc) - 2,838 questions

**Total: 65,133 MC questions**

## Prerequisites

```bash
pip install anthropic>=0.25.0
# Or for OpenAI: pip install openai>=1.12.0
```

## Setup API Keys

### Anthropic Claude

```bash
# Set API key
export ANTHROPIC_API_KEY="sk-ant-api03-..."
```

### OpenAI GPT-4o / GPT-4 Turbo

```bash
# Set API key
export OPENAI_API_KEY="sk-proj-..."
```

### Verify Setup

```bash
# Test connection
python3 -c "import anthropic; client = anthropic.Anthropic(); print(client.models.list)"
```

## Kaggle Setup

```bash
# Login to Kaggle
kaggle competitions list
```

## Running Benchmarks

### Option 1: Kaggle Notebooks (Online)

1. Open Kaggle notebook for desired track:
   - [THLP](https://www.kaggle.com/code/playra/trinity-thlp-hippocampal-learning-mc-benchmark)
   - [TTM](https://www.kaggle.com/code/playra/trinity-ttm-metacognition-mc-benchmark)
   - [TAGP](https://www.kaggle.com/code/playra/trinity-tagp-attentional-gateway-mc-benchmark)
   - [TEFB](https://www.kaggle.com/code/playra/trinity-tefb-executive-function-mc-benchmark)
   - [TSCP](https://www.kaggle.com/code/playra/trinity-tscp-social-cognition-mc-benchmark)

2. In Settings panel, select model:
   - **For Claude**: claude-3-5-sonnet-20250214
   - **For OpenAI**: gpt-4o-2024-04-09
   - **For GPT-4o**: gpt-4-turbo-2024-04-09

3. Run all cells

### Option 2: Local Testing

Create a simple evaluation script for testing one question:

```python3
import anthropic

client = anthropic.Anthropic(api_key=os.environ['ANTHROPIC_API_KEY'])

response = client.messages.create(
    model="claude-3-5-sonnet-20250214",
    max_tokens=1024,
    temperature=0.3,
    messages=[{
        {"role": "user", "content": "Solve this: 0°C water boils at 100°C?"},
        {"role": "assistant", "content": "No, water boils at 100°C at sea level."}
    }
)

answer = response.content[0].content[0].strip()  # Extract first character
print(f"Model answered: {answer}")
```

## Dataset Details

Each dataset has MC questions formatted as:
```
id,question_type,question,choices,answer
```

Example row:
```
thlp_learning_0001,mc,"Which best describes: 0°C?","A) 0°C | B) Both statements could be true | C) The first statement is correct | D) The first statement is true
```

## Running Full Evaluation

To evaluate all tracks on all models, you would need to:
1. Run through all 5 notebooks on Kaggle with each model
2. Or create a batch evaluation script using API

## Expected Results

- **Accuracy**: Percentage of correct answers (A-D)
- **Performance per cognitive domain**: Learning, Metacognition, Attention, Executive Function, Social Cognition
- **Leaderboard position**: Compared to other hackathon participants

## Troubleshooting

### API Errors

If you get errors:
```python
# Check credentials
python3 -c "import anthropic; client = anthropic.Anthropic(api_key=os.environ['ANTHROPIC_API_KEY']); print(client.models.list())"
```

### Model Selection

- **Claude Sonnet** (claude-3-5-sonnet-20250214): Best balance of speed and quality
- **Claude Opus** (claude-3-opus-20250214): Most capable for complex reasoning
- **GPT-4o** (gpt-4o): Best for coding and technical tasks
- **GPT-4 Turbo** (gpt-4-turbo): Faster, good for quick tests

## Sources

- [Datasets](https://github.com/gHashTag/t27/tree/main/external/kaggle)
- [Repository](https://github.com/gHashTag/t27/tree/main/external/kaggle)
- [Kaggle Notebooks](https://github.com/gHashTag/t27/tree/main/external/kaggle/notebooks)

## License

All datasets released under **CC0-1.0** (Public Domain)
