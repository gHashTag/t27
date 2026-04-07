# contrib/backend/notebooklm/queries.py
# Query operations for NotebookLM integration
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Query operations: ask questions, get results."""

import asyncio
from dataclasses import dataclass, asdict
from typing import Optional, List, Dict, Any
from datetime import datetime

try:
    from notebooklm import NotebookLMClient
    NOTEBOOKLM_AVAILABLE = True
except ImportError:
    NOTEBOOKLM_AVAILABLE = False

from .client import client_get_current


@dataclass
class QueryResult:
    """Query result data structure.

    Attributes:
        notebook_id: Target notebook ID
        query: The question asked
        answer: The AI's answer
        sources: Up to 5 relevant source IDs
        confidence: Confidence score [0.0, 1.0]
        timestamp: Query timestamp
    """

    notebook_id: str
    query: str
    answer: str
    sources: List[str]
    confidence: float
    timestamp: str

    def to_dict(self) -> Dict[str, Any]:
        """Convert to dictionary."""
        return asdict(self)


def _run_async(coro):
    """Run async coroutine synchronously."""
    try:
        return asyncio.run(coro)
    except RuntimeError as e:
        if "This event loop" in str(e):
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            try:
                return loop.run_until_complete(coro)
            finally:
                loop.close()
        raise


def notebook_query(notebook_id: str, question: str) -> Optional[Dict[str, Any]]:
    """Query a notebook with a question.

    Args:
        notebook_id: Target notebook ID
        question: Question to ask

    Returns:
        Dict with query result or None if failed
    """
    client = client_get_current()
    if client is None:
        print("Error: No authenticated client")
        return None

    async def _query() -> Optional[Dict[str, Any]]:
        try:
            result = await client.chat.ask(notebook_id, question)

            # Extract source IDs from result
            sources = []
            if hasattr(result, "references") and result.references:
                sources = [ref.id for ref in result.references[:5]]

            # Estimate confidence from response metadata
            confidence = 0.5  # Default confidence
            if hasattr(result, "answer"):
                confidence = 0.8  # Higher confidence if we got an answer

            return QueryResult(
                notebook_id=notebook_id,
                query=question,
                answer=result.answer if hasattr(result, "answer") else "",
                sources=sources,
                confidence=confidence,
                timestamp=datetime.now().isoformat(),
            ).to_dict()
        except Exception as e:
            print(f"Error querying notebook: {e}")
            return None

    return _run_async(_query())


def notebook_query_multiple(notebook_id: str, questions: List[str]) -> List[Dict[str, Any]]:
    """Query a notebook with multiple questions.

    Args:
        notebook_id: Target notebook ID
        questions: List of questions to ask

    Returns:
        List of query result dicts
    """
    results = []

    for question in questions:
        result = notebook_query(notebook_id, question)
        if result is not None:
            results.append(result)

    return results
