# t27 Python Bindings

**Status:** WIP

## Installation

```bash
pip install maturin
```

## Usage

```python
import t27
```

## Module: t27_core

Rust bindings for t27 types.

### Exports

```rust
use pyo3::prelude::*;

/// t27 Python wrapper types
#[pymodule]
pub fn add(_python: Python, py: Python, _m: &PyModule) -> PyResult<()> {
    let core = _m.getattr("core")?.downcast_ref::<Py<t27_core>>()?;
    core.add(py)?;
    Ok(())
}

/// Document type
#[pyclass]
pub struct Document {
    #[pyo3(get)]
    pub uri: String,
    #[pyo3(get)]
    pub text: String,
    #[pyo3(get)]
    pub version: i32,
}

/// Symbol type
#[pyclass]
pub struct Symbol {
    #[pyo3(get)]
    pub name: String,
    #[pyo3(get)]
    pub kind: String,
    #[pyo3(get)]
    pub detail: Option<String>,
}

/// Diagnostic type
#[pyclass]
pub struct Diagnostic {
    #[pyo3(get)]
    pub range: Option<Range>,
    #[pyo3(get)]
    pub severity: String,
    #[pyo3(get)]
    pub message: String,
}

/// Range type
#[pyclass]
pub struct Range {
    #[pyo3(get)]
    pub start: Position,
    #[pyo3(get)]
    pub end: Position,
}

/// Position type
#[pyclass]
pub struct Position {
    #[pyo3(get)]
    pub line: u32,
    #[pyo3(get)]
    pub character: u32,
}

/// t27 core Python wrapper
#[pyclass]
pub struct T27Core {
    #[pyo3(get)]
    pub fn version(&self) -> String {
        "0.1.0".to_string()
    }

    #[pyo3(get)]
    pub fn create_document(&self, uri: String, text: String) -> Document {
        Document {
            uri,
            text,
            version: 1,
        }
    }

    #[pyo3(get)]
    pub fn create_symbol(&self, name: String, kind: String) -> Symbol {
        Symbol {
            name,
            kind,
            detail: None,
        }
    }

    #[pyo3(get)]
    pub fn create_diagnostic(&self, message: String, severity: String) -> Diagnostic {
        Diagnostic {
            range: None,
            severity,
            message,
        }
    }

    #[pyo3(get)]
    pub fn create_range(&self, start: Position, end: Position) -> Range {
        Range { start, end }
    }
}

/// t27_core module
#[pymodule]
pub struct T27Core {}

impl T27Core {
    #[new]
    pub fn version(&self) -> String {
        "0.1.0".to_string()
    }
}
