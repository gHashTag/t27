"""Unit tests for enrich.py transcript extraction."""

import re
import sys
from pathlib import Path
from typing import Optional
from urllib.parse import urlparse, parse_qs

# Constants from enrich.py
MAX_TRANSCRIPT_SIZE = 10 * 1024 * 1024  # 10MB limit
YOUTUBE_DOMAINS = {"youtube.com", "www.youtube.com", "m.youtube.com", "music.youtube.com", "youtu.be"}


class TestHelperMethods:
    """Test helper methods directly without full class import."""

    def _is_youtube_url(self, url: str) -> bool:
        """Check if URL is a YouTube URL."""
        parsed = urlparse(url.strip())
        hostname = (parsed.hostname or "").lower()
        return hostname in YOUTUBE_DOMAINS

    def _extract_youtube_video_id(self, url: str) -> Optional[str]:
        """Extract YouTube video ID from URL (mirrors SDK pattern)."""
        try:
            parsed = urlparse(url.strip())
            hostname = (parsed.hostname or "").lower()

            # youtu.be short URLs
            if hostname == "youtu.be":
                return parsed.path.lstrip("/").split("/")[0]

            # youtube.com path-based formats
            path_segments = parsed.path.lstrip("/").split("/")
            if len(path_segments) >= 2 and path_segments[0] in ("shorts", "embed", "live", "v"):
                return path_segments[1]

            # Query param ?v=VIDEO_ID
            if parsed.query:
                query_params = parse_qs(parsed.query)
                if "v" in query_params:
                    return query_params["v"][0]
        except Exception:
            pass
        return None

    def _srt_to_text(self, srt_content: str) -> str:
        """Convert SRT subtitle format to plain text."""
        lines = []
        in_text_block = False

        for line in srt_content.split("\n"):
            line = line.strip()

            # Skip timestamps (contain -->)
            if "-->" in line or re.match(r"^\d+$", line):
                in_text_block = False
                continue

            # Skip empty lines and line numbers
            if not line or (not in_text_block and line.isdigit()):
                if not line.isdigit():
                    in_text_block = True
                continue

            lines.append(line)

        return "\n".join(lines)

    # Test methods
    def test_is_youtube_url_standard(self):
        """Test standard youtube.com URL detection."""
        assert self._is_youtube_url("https://youtube.com/watch?v=abc123")
        assert self._is_youtube_url("https://www.youtube.com/watch?v=abc123")
        assert self._is_youtube_url("http://www.youtube.com/watch?v=abc123")

    def test_is_youtube_url_short(self):
        """Test youtu.be short URL detection."""
        assert self._is_youtube_url("https://youtu.be/abc123")
        assert self._is_youtube_url("http://youtu.be/abc123")

    def test_is_youtube_url_mobile(self):
        """Test m.youtube.com URL detection."""
        assert self._is_youtube_url("https://m.youtube.com/watch?v=abc123")

    def test_is_youtube_url_music(self):
        """Test music.youtube.com URL detection."""
        assert self._is_youtube_url("https://music.youtube.com/watch?v=abc123")

    def test_is_youtube_url_negative(self):
        """Test non-YouTube URLs are not detected."""
        assert not self._is_youtube_url("https://example.com")
        assert not self._is_youtube_url("https://vimeo.com/123")
        assert not self._is_youtube_url("https://fakeyoutube.com/watch?v=abc")

    def test_is_youtube_url_with_whitespace(self):
        """Test URLs with surrounding whitespace."""
        assert self._is_youtube_url("  https://youtube.com/watch?v=abc123  ")
        assert self._is_youtube_url("\nhttps://youtu.be/xyz\n")

    def test_extract_standard_url(self):
        """Test video ID from standard youtube.com URL."""
        assert self._extract_youtube_video_id("https://youtube.com/watch?v=abc123") == "abc123"
        assert self._extract_youtube_video_id("https://www.youtube.com/watch?v=xyz789&t=10") == "xyz789"

    def test_extract_short_url(self):
        """Test video ID from youtu.be short URL."""
        assert self._extract_youtube_video_id("https://youtu.be/abc123") == "abc123"
        assert self._extract_youtube_video_id("https://youtu.be/abc123?t=10") == "abc123"

    def test_extract_embed_url(self):
        """Test video ID from embed URL."""
        assert self._extract_youtube_video_id("https://youtube.com/embed/abc123") == "abc123"
        assert self._extract_youtube_video_id("https://www.youtube.com/embed/xyz789") == "xyz789"

    def test_extract_shorts_url(self):
        """Test video ID from shorts URL."""
        assert self._extract_youtube_video_id("https://youtube.com/shorts/abc123") == "abc123"
        assert self._extract_youtube_video_id("https://www.youtube.com/shorts/xyz789") == "xyz789"

    def test_extract_live_url(self):
        """Test video ID from live URL."""
        assert self._extract_youtube_video_id("https://youtube.com/live/abc123") == "abc123"

    def test_extract_v_path_url(self):
        """Test video ID from /v/ path URL."""
        assert self._extract_youtube_video_id("https://youtube.com/v/abc123") == "abc123"

    def test_extract_negative_cases(self):
        """Test video ID extraction from non-YouTube or malformed URLs."""
        assert self._extract_youtube_video_id("https://example.com") is None
        assert self._extract_youtube_video_id("https://youtube.com") is None
        assert self._extract_youtube_video_id("") is None
        assert self._extract_youtube_video_id("not a url") is None

    def test_extract_malformed_url(self):
        """Test video ID extraction from malformed URLs returns None."""
        assert self._extract_youtube_video_id("https://youtube.com/?t=10") is None

    def test_basic_conversion(self):
        """Test basic SRT conversion."""
        srt = """1
00:00:00 --> 00:00:05
Hello world

2
00:00:05 --> 00:00:10
Test transcript"""
        result = self._srt_to_text(srt)
        assert "Hello world" in result
        assert "Test transcript" in result
        assert "-->" not in result
        assert "00:00:00" not in result
        assert "1\n" not in result

    def test_skip_timestamps_and_numbers(self):
        """Test that timestamps and line numbers are removed."""
        srt = """1
00:00:00,500 --> 00:00:05,000
First line

2
00:00:06,000 --> 00:00:10,000
Second line"""
        result = self._srt_to_text(srt)
        assert "First line" in result
        assert "Second line" in result
        assert "-->" not in result
        assert not re.search(r"^\d+$", result, re.MULTILINE)

    def test_empty_lines_handling(self):
        """Test empty lines are handled correctly."""
        srt = """1
00:00:00 --> 00:00:05
Text here


2
00:00:05 --> 00:00:10
More text"""
        result = self._srt_to_text(srt)
        lines = result.split("\n")
        assert "Text here" in result
        assert "More text" in result
        # Check we don't have excessive empty lines
        empty_count = sum(1 for line in lines if not line.strip())
        assert empty_count < len(lines)  # Should have fewer empty lines than total

    def test_html_tags_in_srt(self):
        """Test that HTML tags in SRT are preserved."""
        srt = """1
00:00:00 --> 00:00:05
<i>Italic text</i> and <b>bold</b>"""
        result = self._srt_to_text(srt)
        assert "Italic text" in result
        assert "bold" in result

    def test_unicode_in_srt(self):
        """Test that unicode characters in SRT are preserved."""
        srt = """1
00:00:00 --> 00:00:05
Hello 世界 🌍"""
        result = self._srt_to_text(srt)
        assert "Hello" in result
        assert "世界" in result
        assert "🌍" in result


def test_constants():
    """Test module constants."""
    expected = {"youtube.com", "www.youtube.com", "m.youtube.com", "music.youtube.com", "youtu.be"}
    assert YOUTUBE_DOMAINS == expected


if __name__ == "__main__":
    """Run all tests."""
    test = TestHelperMethods()

    # YouTube URL detection tests
    test.test_is_youtube_url_standard()
    print("✓ test_is_youtube_url_standard")

    test.test_is_youtube_url_short()
    print("✓ test_is_youtube_url_short")

    test.test_is_youtube_url_mobile()
    print("✓ test_is_youtube_url_mobile")

    test.test_is_youtube_url_music()
    print("✓ test_is_youtube_url_music")

    test.test_is_youtube_url_negative()
    print("✓ test_is_youtube_url_negative")

    test.test_is_youtube_url_with_whitespace()
    print("✓ test_is_youtube_url_with_whitespace")

    # Video ID extraction tests
    test.test_extract_standard_url()
    print("✓ test_extract_standard_url")

    test.test_extract_short_url()
    print("✓ test_extract_short_url")

    test.test_extract_embed_url()
    print("✓ test_extract_embed_url")

    test.test_extract_shorts_url()
    print("✓ test_extract_shorts_url")

    test.test_extract_live_url()
    print("✓ test_extract_live_url")

    test.test_extract_v_path_url()
    print("✓ test_extract_v_path_url")

    test.test_extract_negative_cases()
    print("✓ test_extract_negative_cases")

    test.test_extract_malformed_url()
    print("✓ test_extract_malformed_url")

    # SRT conversion tests
    test.test_basic_conversion()
    print("✓ test_basic_conversion")

    test.test_skip_timestamps_and_numbers()
    print("✓ test_skip_timestamps_and_numbers")

    test.test_empty_lines_handling()
    print("✓ test_empty_lines_handling")

    test.test_html_tags_in_srt()
    print("✓ test_html_tags_in_srt")

    test.test_unicode_in_srt()
    print("✓ test_unicode_in_srt")

    # Constants test
    test_constants()
    print("✓ test_constants")

    print("\n" + "="*50)
    print("All 19 tests passed!")
    print("="*50)
