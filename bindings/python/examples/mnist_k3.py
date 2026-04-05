"""
MNIST Classification with Ternary Neural Network
Demonstrates training a digit classifier using K3 layers
"""

import torch
import torch.nn as nn
import torch.nn.functional as F
import torch.optim as optim
from torch.utils.data import DataLoader
from t27 import K3Linear, K3Conv2d, K3LayerNorm

print("=" * 50)
print("MNIST Training with Ternary Weights")
print("=" * 50)
print()

# Configuration
BATCH_SIZE = 128
LEARNING_RATE = 0.001
EPOCHS = 3  # Small for demo
EMBED_DIM = 243  # HSLM standard

print(f"Configuration:")
print(f"  Batch Size:    {BATCH_SIZE}")
print(f"  Learning Rate: {LEARNING_RATE}")
print(f"  Epochs:        {EPOCHS}")
print(f"  Embed Dim:     {EMBED_DIM}")
print()


class TernaryMNIST(nn.Module):
    """
    MNIST classifier with ternary weights.

    Architecture:
        - Conv2d(1, 32, 3) + ReLU
        - Conv2d(32, 64, 3) + ReLU
        - MaxPool2d(2)
        - Flatten
        - Linear(9216, 128) + ReLU
        - Linear(128, 10)
    """

    def __init__(self):
        super().__init__()

        # Convolutional layers with ternary weights
        self.conv1 = K3Conv2d(1, 32, kernel_size=3, padding=1, sparse_init=True)
        self.conv2 = K3Conv2d(32, 64, kernel_size=3, padding=1, sparse_init=True)

        # Linear layers with ternary weights
        self.fc1 = K3Linear(64 * 7 * 7, 128, sparse_init=True)
        self.fc2 = K3Linear(128, 10, sparse_init=False)  # Keep output layer dense

        self.pool = nn.MaxPool2d(2)
        self.norm1 = K3LayerNorm(32)
        self.norm2 = K3LayerNorm(64)
        self.norm3 = K3LayerNorm(128)

    def forward(self, x):
        # Conv block 1
        x = self.conv1(x)
        x = self.norm1(x)
        x = F.relu(x)
        x = self.pool(x)

        # Conv block 2
        x = self.conv2(x)
        x = self.norm2(x)
        x = F.relu(x)
        x = self.pool(x)

        # Flatten
        x = x.view(x.size(0), -1)

        # FC layers
        x = self.fc1(x)
        x = self.norm3(x)
        x = F.relu(x)
        x = self.fc2(x)

        return x

    def update_ternary_weights(self):
        """Quantize all continuous weights back to ternary."""
        self.conv1.update_ternary_weights()
        self.conv2.update_ternary_weights()
        self.fc1.update_ternary_weights()
        self.fc2.update_ternary_weights()


def create_model():
    """Create and initialize the model."""
    model = TernaryMNIST()
    return model


def print_model_stats(model):
    """Print model statistics."""
    total_params = sum(p.numel() for p in model.parameters())

    ternary_count = 0
    zero_count = 0

    for name, module in model.named_modules():
        if hasattr(module, 'weight_ternary'):
            ternary_count += module.weight_ternary.numel()
            zero_count += (module.weight_ternary == 0).sum().item()
            sparsity = (module.weight_ternary == 0).sum().item() / module.weight_ternary.numel() * 100
            print(f"  {name:15s}: {module.weight_ternary.numel():6,} trits ({sparsity:5.1f}% sparse)")

    print(f"\nTotal parameters:     {total_params:,}")
    print(f"Ternary weights:      {ternary_count:,}")
    print(f"Zero weights (sparse): {zero_count:,} ({zero_count/ternary_count*100:.1f}%)")
    print(f"Active weights:       {ternary_count - zero_count:,} ({(1-zero_count/ternary_count)*100:.1f}%)")
    print()

    return ternary_count


def synthetic_data():
    """
    Generate synthetic MNIST-like data for demonstration.

    In a real scenario, use torchvision.datasets.MNIST:

    from torchvision import transforms
    transform = transforms.Compose([transforms.ToTensor()])
    train_dataset = datasets.MNIST('./data', train=True, download=True, transform=transform)
    train_loader = DataLoader(train_dataset, batch_size=BATCH_SIZE, shuffle=True)
    """
    # Generate synthetic images (28x28)
    images = torch.randn(BATCH_SIZE, 1, 28, 28)
    labels = torch.randint(0, 10, (BATCH_SIZE,))
    return images, labels


def train_epoch(model, device, optimizer, criterion, epoch):
    """Train for one epoch."""
    model.train()
    total_loss = 0
    correct = 0
    total = 0

    # Simulate batches (in real training, loop over dataloader)
    num_batches = 10  # Small for demo

    for batch_idx in range(num_batches):
        # Get batch data
        data, target = synthetic_data()

        optimizer.zero_grad()

        # Forward pass
        output = model(data)
        loss = criterion(output, target)

        # Backward pass
        loss.backward()
        optimizer.step()

        # Quantize ternary weights
        model.update_ternary_weights()

        # Statistics
        total_loss += loss.item()
        pred = output.argmax(dim=1, keepdim=True)
        correct += pred.eq(target.view_as(pred)).sum().item()
        total += target.size(0)

    avg_loss = total_loss / num_batches
    accuracy = 100.0 * correct / total

    return avg_loss, accuracy


def main():
    """Main training loop."""

    # Create model
    print("Creating model...")
    model = create_model()
    device = torch.device("cpu")  # Use CPU for demo
    model = model.to(device)

    print("\nModel Architecture:")
    print("-" * 40)
    ternary_count = print_model_stats(model)

    # Optimizer (only train continuous weights)
    optimizer = optim.Adam(
        [p for p in model.parameters() if p.requires_grad and p.size() > 0],
        lr=LEARNING_RATE
    )

    # Loss function
    criterion = nn.CrossEntropyLoss()

    # Training loop
    print("=" * 50)
    print("Training")
    print("=" * 50)
    print()

    for epoch in range(1, EPOCHS + 1):
        loss, acc = train_epoch(model, device, optimizer, criterion, epoch)
        print(f"Epoch {epoch}/{EPOCHS}: Loss={loss:.4f}, Accuracy={acc:.2f}%")

    print()
    print("=" * 50)
    print("Training Complete!")
    print("=" * 50)
    print()

    # Final statistics
    print("Final Ternary Weight Statistics:")
    print("-" * 40)

    for name, module in model.named_modules():
        if hasattr(module, 'weight_ternary'):
            pos = (module.weight_ternary == 1).sum().item()
            neg = (module.weight_ternary == -1).sum().item()
            zero = (module.weight_ternary == 0).sum().item()
            total = module.weight_ternary.numel()

            print(f"\n{name}:")
            print(f"  POS  (+1): {pos:6,} ({pos/total*100:5.2f}%)")
            print(f"  ZERO ( 0): {zero:6,} ({zero/total*100:5.2f}%)")
            print(f"  NEG  (-1): {neg:6,} ({neg/total*100:5.2f}%)")

    print()
    print("=" * 50)
    print("Performance Notes")
    print("=" * 50)
    print()
    print("1. Memory Efficiency:")
    print("   - Ternary weights use ~2 bits per weight (vs 32 bits for fp32)")
    print("   - Sparse representation reduces storage further")
    print()
    print("2. Hardware Efficiency:")
    print("   - No DSP blocks needed on FPGAs")
    print("   - Simple integer operations")
    print()
    print("3. Expected Accuracy (Real MNIST):")
    print("   - Full precision baseline: ~98%")
    print("   - Ternary weights (this model): ~95-97%")
    print("   - Gap due to quantization and sparsity")
    print()
    print("4. Training Tips:")
    print("   - Use larger models to compensate for quantization")
    print("   - Consider gradual sparsity increase")
    print("   - Lower learning rate may help with ternary constraints")
    print()


if __name__ == "__main__":
    main()
