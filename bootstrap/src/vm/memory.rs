//! Memory Management
//!
//! Trinity VM memory layout and management.
//! Part of modular compiler architecture (Ring-019 - Parity Target 1).
//!
//! Implements canonical memory layout:
//! - Stack: 64KB, grows upward
//! - Heap: 256KB, growable
//! - Code: Read-only, loaded at base address
//! - Symbol table: Fixed at code section end
//!
//! Conforms to `specs/vm_core.t27` memory layout contract.

use super::super::super::types;

/// Memory layout constants (from vm_core.t27)
pub const STACK_SIZE: u32 = 65536;      // 64KB
pub const HEAP_SIZE: u32 = 262144;      // 256KB
pub const ALIGNMENT: u32 = 8;            // 8-byte alignment

/// Memory sections
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MemorySection {
  Stack,
  Heap,
  Code,
  SymbolTable,
}

/// Memory allocation result
#[derive(Debug)]
pub struct AllocResult {
  pub base: usize,
  pub size: usize,
}

/// Stack implementation (64KB, grows upward)
///
/// Stack is 64KB region that grows as needed.
/// Stack pointer (SP) points to top of stack.
pub struct Stack {
  pub base: usize,
  pub top: usize,    // Current stack pointer
  pub capacity: usize,
}

impl Stack {
  /// Create new stack
  pub fn new() -> Self {
    let memory = vec![0u8; STACK_SIZE];
    Stack {
      base: memory.as_ptr() as usize,
      top: memory.as_ptr() as usize,
      capacity: STACK_SIZE,
    }
  }

  /// Push value onto stack (grows downward)
  pub fn push(&mut self, value: u64) -> Result<usize, String> {
    if self.top < self.base + 8 {
      // Push 8 bytes (u64) onto stack
      self.top -= 8;
      unsafe {
        *(self.top as *mut u64) = value;
      }
      Ok(self.top)
    } else {
      Err("Stack overflow".to_string())
    }
  }

  /// Pop value from stack (grows upward)
  pub fn pop(&mut self) -> Result<u64, String> {
    if self.top < self.base + STACK_SIZE - 8 {
      let value = unsafe { *(self.top as *mut u64) };
      self.top += 8;
      Ok(value)
    } else {
      Err("Stack underflow".to_string())
    }
  }

  /// Get current stack depth
  pub fn depth(&self) -> usize {
    (self.base + STACK_SIZE - self.top) / 8
  }
}

/// Heap implementation (256KB, growable)
///
/// Heap is for dynamic allocations.
/// Managed via simple bump allocator.
pub struct Heap {
  pub base: usize,
  pub top: usize,
  pub capacity: usize,
}

impl Heap {
  /// Create new heap
  pub fn new() -> Self {
    let memory = vec![0u8; HEAP_SIZE];
    Heap {
      base: memory.as_ptr() as usize,
      top: memory.as_ptr() as usize,
      capacity: HEAP_SIZE,
    }
  }

  /// Allocate bytes from heap (grows upward)
  pub fn alloc(&mut self, size: usize) -> Result<usize, String> {
    let aligned = (size + (ALIGNMENT - 1)) / ALIGNMENT * ALIGNMENT;
    if self.top + aligned <= self.base + self.capacity {
      self.top += aligned;
      Ok(self.top - aligned)
    } else {
      Err("Heap overflow".to_string())
    }
  }

  /// Reset heap (for new execution)
  pub fn reset(&mut self) {
    self.top = self.base;
  }

  /// Get heap usage
  pub fn used(&self) -> usize {
    self.top - self.base
  }
}

/// Memory management
///
/// Manages all memory regions.
pub struct Memory {
  pub stack: Stack,
  pub heap: Heap,
  pub code_size: usize,
  pub symbol_table_size: usize,
}

impl Memory {
  /// Create new memory with canonical layout
  pub fn new() -> Self {
    Memory {
      stack: Stack::new(),
      heap: Heap::new(),
      code_size: 0,
      symbol_table_size: 0,
    }
  }

  /// Get memory usage stats
  pub fn stats(&self) -> MemoryStats {
    MemoryStats {
      stack_used: self.stack.depth() * 8,
      heap_used: self.heap.used(),
      stack_capacity: self.stack.capacity,
      heap_capacity: self.heap.capacity,
      code_size: self.code_size,
      symbol_table_size: self.symbol_table_size,
    }
  }

  /// Reset all memory regions (for new execution)
  pub fn reset(&mut self) {
    self.stack.top = self.stack.base + self.stack.capacity;
    self.heap.reset();
    self.code_size = 0;
    self.symbol_table_size = 0;
  }
}

/// Memory statistics
#[derive(Debug)]
pub struct MemoryStats {
  pub stack_used: usize,
  pub heap_used: usize,
  pub stack_capacity: usize,
  pub heap_capacity: usize,
  pub code_size: usize,
  pub symbol_table_size: usize,
}
