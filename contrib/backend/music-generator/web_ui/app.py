# contrib/backend/music-generator/web_ui/app.py
# Gradio web interface for music generation
# phi^2 + 1/phi^2 = 3 | TRINITY

"""Gradio web interface for T27 Music Generator.

Provides user-friendly web UI for generating music tracks
with real-time progress display and audio playback.
"""

from pathlib import Path
from typing import Optional, Tuple
import logging
import tempfile

try:
    import gradio as gr
    GRADIO_AVAILABLE = True
except ImportError:
    GRADIO_AVAILABLE = False

from ..pipeline import MusicPipeline
from ..config import config_from_env

logger = logging.getLogger(__name__)


class MusicGeneratorUI:
    """Gradio UI for music generation.

    Provides web interface with inputs for lyrics, style selection,
    effect controls, and real-time progress display.

    Attributes:
        pipeline: Music generation pipeline
        temp_dir: Temporary directory for outputs

    Example:
        >>> ui = MusicGeneratorUI()
        >>> app = ui.create_app()
        >>> app.launch()
    """

    def __init__(self, device: str = "cpu"):
        """Initialize UI.

        Args:
            device: Target device for generation

        Complexity: O(1)
        """
        self.pipeline = MusicPipeline(device=device)
        self.temp_dir = Path(tempfile.mkdtemp(prefix="t27_music_"))
        logger.info(f"UI initialized with device={device}, temp_dir={self.temp_dir}")

    def generate(
        self,
        lyrics: str,
        style: str,
        vocal_style: str,
        duration: int,
        vocal_level: float,
        intensity: float,
        include_vocals: bool,
        progress: gr.Progress,
    ) -> Tuple[str, str, str]:
        """Generate music track with progress updates.

        Args:
            lyrics: Lyrics text
            style: Musical style
            vocal_style: Vocal delivery style
            duration: Track duration in seconds
            vocal_level: Vocal level in dB
            intensity: Effect intensity
            include_vocals: Whether to include vocals
            progress: Gradio progress tracker

        Returns:
            Tuple of (output_audio_path, status_message, log_text)
        """
        log_lines = []

        def log(message: str) -> None:
            """Log message to output."""
            log_lines.append(message)
            logger.info(message)

        def progress_callback(message: str, percent: int) -> None:
            """Update progress bar."""
            progress(percent / 100, desc=message)
            log(f"[{percent}%] {message}")

        try:
            # Output path
            output_path = self.temp_dir / f"output_{style}_{id(lyrics)}.wav"

            # Generate
            result = self.pipeline.generate(
                lyrics=lyrics,
                style=style,
                output_path=output_path,
                duration=duration,
                vocal_style=vocal_style,
                instrumental_intensity=intensity,
                vocal_level_db=vocal_level,
                skip_vocals=not include_vocals,
                progress_callback=progress_callback,
            )

            status = "Generation complete!"
            log(status)

            return (
                str(output_path),
                status,
                "\n".join(log_lines),
            )

        except Exception as e:
            error_msg = f"Error: {str(e)}"
            log(error_msg)
            return (
                None,
                "Generation failed!",
                "\n".join(log_lines),
            )

    def create_app(self) -> gr.Blocks:
        """Create Gradio app.

        Returns:
            Gradio Blocks app

        Raises:
            ImportError: If Gradio is not installed

        Complexity: O(1)
        """
        if not GRADIO_AVAILABLE:
            raise ImportError(
                "Gradio is required for web UI. "
                "Install with: pip install gradio"
            )

        with gr.Blocks(
            title="T27 Music Generator",
            theme=gr.themes.Soft(),
            css="""
                .gradio-container {
                    max-width: 1200px !important;
                }
                #header {
                    text-align: center;
                    padding: 20px;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    border-radius: 10px;
                    margin-bottom: 20px;
                }
                #header h1 {
                    margin: 0;
                    font-size: 2em;
                }
                #header p {
                    margin: 5px 0 0 0;
                    opacity: 0.9;
                }
            """,
        ) as app:
            # Header
            gr.HTML("""
                <div id="header">
                    <h1>T27 Music Generator</h1>
                    <p>AI Music Generation with Voice Cloning | phi^2 + 1/phi^2 = 3 | TRINITY</p>
                </div>
            """)

            with gr.Row():
                # Left column: Inputs
                with gr.Column(scale=1):
                    gr.Markdown("### Input")

                    lyrics_input = gr.Textbox(
                        label="Lyrics",
                        placeholder="Enter your lyrics here...",
                        lines=6,
                    )

                    with gr.Row():
                        style_dropdown = gr.Dropdown(
                            label="Style",
                            choices=["phonk", "trap", "hip_hop", "drill", "lofi"],
                            value="phonk",
                        )
                        vocal_style_dropdown = gr.Dropdown(
                            label="Vocal Style",
                            choices=["aggressive", "eerie", "ethereal", "smooth", "choppy"],
                            value="aggressive",
                        )

                    duration_slider = gr.Slider(
                        label="Duration (seconds)",
                        minimum=15,
                        maximum=180,
                        value=60,
                        step=15,
                    )

                    with gr.Row():
                        vocal_level_slider = gr.Slider(
                            label="Vocal Level (dB)",
                            minimum=-12,
                            maximum=6,
                            value=-4,
                            step=1,
                        )
                        intensity_slider = gr.Slider(
                            label="Effect Intensity",
                            minimum=0.0,
                            maximum=2.0,
                            value=1.0,
                            step=0.1,
                        )

                    include_vocals_checkbox = gr.Checkbox(
                        label="Include Vocals",
                        value=True,
                    )

                    generate_btn = gr.Button(
                        "Generate Track",
                        variant="primary",
                        size="lg",
                    )

                # Right column: Outputs
                with gr.Column(scale=1):
                    gr.Markdown("### Output")

                    audio_output = gr.Audio(
                        label="Generated Track",
                        type="filepath",
                    )

                    status_output = gr.Textbox(
                        label="Status",
                        interactive=False,
                    )

                    log_output = gr.Textbox(
                        label="Generation Log",
                        lines=10,
                        interactive=False,
                    )

                    download_btn = gr.Button(
                        "Download Track",
                        visible=False,
                    )

            # Examples
            gr.Markdown("### Examples")
            gr.Examples(
                examples=[
                    ["Shadows creeping in the night, demons whispering my name", "phonk", "aggressive", 60, -4, 1.0, True],
                    ["Riding through the city lights, chasing dreams all night", "trap", "smooth", 60, -4, 1.0, True],
                    ["Chill vibes, rainy days, lost in thought", "lofi", "ethereal", 45, -6, 0.8, True],
                ],
                inputs=[
                    lyrics_input,
                    style_dropdown,
                    vocal_style_dropdown,
                    duration_slider,
                    vocal_level_slider,
                    intensity_slider,
                    include_vocals_checkbox,
                ],
            )

            # Event handlers
            generate_btn.click(
                fn=self.generate,
                inputs=[
                    lyrics_input,
                    style_dropdown,
                    vocal_style_dropdown,
                    duration_slider,
                    vocal_level_slider,
                    intensity_slider,
                    include_vocals_checkbox,
                ],
                outputs=[
                    audio_output,
                    status_output,
                    log_output,
                ],
            )

            # Style presets
            gr.Markdown("### Style Presets")
            with gr.Row():
                phonk_btn = gr.Button("Phonk", size="sm")
                trap_btn = gr.Button("Trap", size="sm")
                drill_btn = gr.Button("Drill", size="sm")
                lofi_btn = gr.Button("Lofi", size="sm")

            phonk_btn.click(lambda: "phonk", outputs=style_dropdown)
            trap_btn.click(lambda: "trap", outputs=style_dropdown)
            drill_btn.click(lambda: "drill", outputs=style_dropdown)
            lofi_btn.click(lambda: "lofi", outputs=style_dropdown)

            # Footer
            gr.HTML("""
                <div style="text-align: center; padding: 20px; color: #666;">
                    <p>T27 Music Generator v1.0.0 | CPU-Optimized | Open Source</p>
                </div>
            """)

        return app


def create_app(device: str = "cpu") -> gr.Blocks:
    """Create Gradio app instance.

    Args:
        device: Target device for generation

    Returns:
        Gradio Blocks app

    Complexity: O(1)

    Example:
        >>> app = create_app(device="cpu")
        >>> app.launch()
    """
    ui = MusicGeneratorUI(device=device)
    return ui.create_app()


def run_app(
    host: str = "127.0.0.1",
    port: int = 7860,
    device: str = "cpu",
    share: bool = False,
) -> None:
    """Run Gradio app.

    Args:
        host: Host to bind to
        port: Port to bind to
        device: Target device for generation
        share: Whether to create public link

    Complexity: O(1)

    Example:
        >>> run_app(host="0.0.0.0", port=7860)
    """
    if not GRADIO_AVAILABLE:
        print("Error: Gradio is not installed. Install with: pip install gradio")
        return

    app = create_app(device=device)
    app.launch(
        server_name=host,
        server_port=port,
        share=share,
    )


if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description="T27 Music Generator Web UI")
    parser.add_argument("--host", default="127.0.0.1", help="Host to bind to")
    parser.add_argument("--port", type=int, default=7860, help="Port to bind to")
    parser.add_argument("--device", choices=["cpu", "cuda"], default="cpu", help="Device to use")
    parser.add_argument("--share", action="store_true", help="Create public link")

    args = parser.parse_args()

    run_app(
        host=args.host,
        port=args.port,
        device=args.device,
        share=args.share,
    )
