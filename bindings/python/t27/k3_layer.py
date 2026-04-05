# t27 K3 PyTorch Layers
# PyTorch neural network layers with ternary weights
# φ² + 1/φ² = 3 | TRINITY

"""
K3 (Kleene K3) PyTorch layers for ternary neural networks.

This module provides PyTorch-compatible layers that use ternary weights
{-1, 0, +1} instead of continuous weights. This enables:
- Sparse, efficient models
- Built-in regularization (via zero weights)
- Hardware-friendly implementation (no DSP blocks needed)
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
from typing import Optional
import numpy as np

from .ternary import Trit, k3_and, k3_or, k3_not


class K3Linear(nn.Module):
    """
    Linear layer with ternary weights.

    Weight values are quantized to {-1, 0, +1} during forward pass.
    Gradients are accumulated in continuous form and quantized during weight update.

    Args:
        in_features: Number of input features
        out_features: Number of output features
        bias: If True, adds a learnable bias (continuous)
        sparse_init: If True, initialize with 50% sparsity (half weights = 0)
    """

    def __init__(
        self,
        in_features: int,
        out_features: int,
        bias: bool = True,
        sparse_init: bool = True,
    ):
        super().__init__()
        self.in_features = in_features
        self.out_features = out_features
        self.sparse_init = sparse_init

        # Continuous weight for gradient accumulation
        self.weight_continuous = nn.Parameter(torch.Tensor(out_features, in_features))
        nn.init.xavier_uniform_(self.weight_continuous)

        # Ternary weight (used in forward pass)
        self.register_buffer('weight_ternary', torch.zeros_like(self.weight_continuous, dtype=torch.int8))

        # Quantize weights
        self._quantize_weights()

        if bias:
            self.bias = nn.Parameter(torch.zeros(out_features))
        else:
            self.register_parameter('bias', None)

    def _quantize_weights(self):
        """Quantize continuous weights to ternary values."""
        with torch.no_grad():
            # Sign-based quantization with threshold
            # > 0.1 → +1, < -0.1 → -1, else → 0
            self.weight_ternary = torch.sign(self.weight_continuous)
            self.weight_ternary[torch.abs(self.weight_continuous) < 0.1] = 0

            if self.sparse_init:
                # Initial sparsity: randomly zero 50% of weights
                mask = torch.rand_like(self.weight_continuous) > 0.5
                self.weight_ternary[mask] = 0

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass with ternary weights.

        Args:
            x: Input tensor of shape (*, in_features)

        Returns:
            Output tensor of shape (*, out_features)
        """
        # Convert ternary weights to float for computation
        weight_float = self.weight_ternary.float()

        # Linear operation
        output = F.linear(x, weight_float, self.bias)
        return output

    def update_ternary_weights(self):
        """
        Quantize accumulated continuous gradients back to ternary.

        Call this after optimizer.step() to apply weight updates.
        """
        self._quantize_weights()


class K3Conv2d(nn.Module):
    """
    2D convolution layer with ternary weights.

    Args:
        in_channels: Number of input channels
        out_channels: Number of output channels
        kernel_size: Size of the convolving kernel
        stride: Stride of the convolution
        padding: Padding added to all sides of input
        bias: If True, adds a learnable bias
        sparse_init: If True, initialize with 50% sparsity
    """

    def __init__(
        self,
        in_channels: int,
        out_channels: int,
        kernel_size: int = 3,
        stride: int = 1,
        padding: int = 1,
        bias: bool = True,
        sparse_init: bool = True,
    ):
        super().__init__()
        self.in_channels = in_channels
        self.out_channels = out_channels
        self.kernel_size = kernel_size
        self.stride = stride
        self.padding = padding

        # Continuous weight for gradient accumulation
        self.weight_continuous = nn.Parameter(
            torch.Tensor(out_channels, in_channels, kernel_size, kernel_size)
        )
        nn.init.kaiming_normal_(self.weight_continuous, mode='fan_out', nonlinearity='relu')

        # Ternary weight
        self.register_buffer('weight_ternary', torch.zeros_like(self.weight_continuous, dtype=torch.int8))

        # Quantize weights
        self._quantize_weights()

        if bias:
            self.bias = nn.Parameter(torch.zeros(out_channels))
        else:
            self.register_parameter('bias', None)

    def _quantize_weights(self):
        """Quantize continuous weights to ternary values."""
        with torch.no_grad():
            self.weight_ternary = torch.sign(self.weight_continuous)
            self.weight_ternary[torch.abs(self.weight_continuous) < 0.1] = 0

            if self.sparse_init:
                mask = torch.rand_like(self.weight_continuous) > 0.5
                self.weight_ternary[mask] = 0

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass with ternary weights.

        Args:
            x: Input tensor of shape (*, in_channels, H, W)

        Returns:
            Output tensor of shape (*, out_channels, H', W')
        """
        weight_float = self.weight_ternary.float()
        return F.conv2d(x, weight_float, self.bias, stride=self.stride, padding=self.padding)

    def update_ternary_weights(self):
        """Quantize accumulated continuous gradients back to ternary."""
        self._quantize_weights()


class K3Embedding(nn.Module):
    """
    Embedding layer with ternary embeddings.

    Args:
        num_embeddings: Size of the dictionary of embeddings
        embedding_dim: Size of each embedding vector
        sparse_init: If True, initialize with 50% sparsity
    """

    def __init__(
        self,
        num_embeddings: int,
        embedding_dim: int,
        sparse_init: bool = True,
    ):
        super().__init__()
        self.num_embeddings = num_embeddings
        self.embedding_dim = embedding_dim

        # Continuous embeddings for gradient accumulation
        self.weight_continuous = nn.Parameter(torch.Tensor(num_embeddings, embedding_dim))
        nn.init.normal_(self.weight_continuous, mean=0, std=0.02)

        # Ternary embedding (stored as int8)
        self.register_buffer('weight_ternary', torch.zeros_like(self.weight_continuous, dtype=torch.int8))

        # Quantize embeddings
        self._quantize_weights()

    def _quantize_weights(self):
        """Quantize continuous embeddings to ternary values."""
        with torch.no_grad():
            self.weight_ternary = torch.sign(self.weight_continuous)
            self.weight_ternary[torch.abs(self.weight_continuous) < 0.1] = 0

            if self.sparse_init:
                mask = torch.rand_like(self.weight_continuous) > 0.5
                self.weight_ternary[mask] = 0

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass with ternary embeddings.

        Args:
            x: Input tensor of shape (*, N) containing indices

        Returns:
            Output tensor of shape (*, N, embedding_dim)
        """
        weight_float = self.weight_ternary.float()
        return F.embedding(x, weight_float)

    def update_ternary_weights(self):
        """Quantize accumulated continuous gradients back to ternary."""
        self._quantize_weights()


class K3LayerNorm(nn.Module):
    """
    Layer normalization for ternary neural networks.

    Operates on continuous values (activations, not weights).
    Uses RMSNorm-style normalization without learnable scale (simpler).
    """

    def __init__(self, normalized_shape: int, eps: float = 1e-6):
        super().__init__()
        self.normalized_shape = normalized_shape
        self.eps = eps

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        RMSNorm-style layer normalization.

        Args:
            x: Input tensor

        Returns:
            Normalized tensor
        """
        variance = x.pow(2).mean(-1, keepdim=True)
        return x / torch.sqrt(variance + self.eps)


class K3Attention(nn.Module):
    """
    Attention mechanism with ternary keys, queries, values.

    Computes attention with ternary KV cache while keeping queries continuous.
    """

    def __init__(self, embed_dim: int, num_heads: int = 8):
        super().__init__()
        self.embed_dim = embed_dim
        self.num_heads = num_heads
        self.head_dim = embed_dim // num_heads

        # Ternary KV projections
        self.k_proj = K3Linear(embed_dim, embed_dim, bias=False)
        self.v_proj = K3Linear(embed_dim, embed_dim, bias=False)

        # Continuous Q projection (for compatibility)
        self.q_proj = nn.Linear(embed_dim, embed_dim, bias=False)

        # Output projection
        self.out_proj = nn.Linear(embed_dim, embed_dim, bias=False)

        self.scale = self.head_dim ** -0.5

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass with ternary KV.

        Args:
            x: Input tensor of shape (batch, seq_len, embed_dim)

        Returns:
            Output tensor of shape (batch, seq_len, embed_dim)
        """
        batch_size, seq_len, _ = x.shape

        # Project to Q, K, V
        q = self.q_proj(x).view(batch_size, seq_len, self.num_heads, self.head_dim).transpose(1, 2)
        k = self.k_proj(x).view(batch_size, seq_len, self.num_heads, self.head_dim).transpose(1, 2)
        v = self.v_proj(x).view(batch_size, seq_len, self.num_heads, self.head_dim).transpose(1, 2)

        # Compute attention scores
        scores = torch.matmul(q, k.transpose(-2, -1)) * self.scale

        # Softmax over sequence length
        attn = F.softmax(scores, dim=-1)

        # Apply attention to values
        output = torch.matmul(attn, v)
        output = output.transpose(1, 2).contiguous().view(batch_size, seq_len, self.embed_dim)

        # Output projection
        return self.out_proj(output)


class K3Block(nn.Module):
    """
    Transformer block with ternary attention.

    Contains:
    - Ternary multi-head attention
    - Feed-forward network with ternary weights
    - Layer normalization
    """

    def __init__(self, embed_dim: int, num_heads: int = 8, ff_dim: Optional[int] = None):
        super().__init__()
        self.embed_dim = embed_dim
        self.num_heads = num_heads
        self.ff_dim = ff_dim or embed_dim * 4

        self.norm1 = K3LayerNorm(embed_dim)
        self.attn = K3Attention(embed_dim, num_heads)

        self.norm2 = K3LayerNorm(embed_dim)
        self.ffn = nn.Sequential(
            K3Linear(embed_dim, self.ff_dim),
            nn.ReLU(),
            K3Linear(self.ff_dim, embed_dim),
        )

    def forward(self, x: torch.Tensor) -> torch.Tensor:
        """
        Forward pass with residual connections.

        Args:
            x: Input tensor

        Returns:
            Output tensor
        """
        # Attention + residual
        attn_out = self.attn(x)
        x = x + self.norm1(attn_out)

        # FFN + residual
        ffn_out = self.ffn(x)
        x = x + self.norm2(ffn_out)

        # Update ternary weights
        self.attn.k_proj.update_ternary_weights()
        self.attn.v_proj.update_ternary_weights()
        self.ffn[0].update_ternary_weights()
        self.ffn[2].update_ternary_weights()

        return x


class K3Model(nn.Module):
    """
    Complete K3 model (similar to HSLM architecture).

    Args:
        vocab_size: Vocabulary size
        embed_dim: Embedding dimension
        num_layers: Number of transformer blocks
        num_heads: Number of attention heads
        ff_dim: Feed-forward dimension
    """

    def __init__(
        self,
        vocab_size: int,
        embed_dim: int = 243,
        num_layers: int = 6,
        num_heads: int = 3,
        ff_dim: Optional[int] = None,
    ):
        super().__init__()
        self.vocab_size = vocab_size
        self.embed_dim = embed_dim
        self.num_layers = num_layers

        self.token_embedding = K3Embedding(vocab_size, embed_dim)
        self.pos_embedding = nn.Parameter(torch.zeros(1, 1024, embed_dim))  # Max seq len

        self.blocks = nn.ModuleList([
            K3Block(embed_dim, num_heads, ff_dim)
            for _ in range(num_layers)
        ])

        self.norm = K3LayerNorm(embed_dim)
        self.lm_head = nn.Linear(embed_dim, vocab_size, bias=False)

    def forward(self, input_ids: torch.Tensor) -> torch.Tensor:
        """
        Forward pass.

        Args:
            input_ids: Input token IDs of shape (batch, seq_len)

        Returns:
            Logits of shape (batch, seq_len, vocab_size)
        """
        batch_size, seq_len = input_ids.shape

        # Token + positional embeddings
        x = self.token_embedding(input_ids)
        pos_emb = self.pos_embedding[:, :seq_len, :]
        x = x + pos_emb

        # Transformer blocks
        for block in self.blocks:
            x = block(x)

        x = self.norm(x)
        logits = self.lm_head(x)

        return logits
