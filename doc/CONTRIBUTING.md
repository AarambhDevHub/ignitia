# Contributing to Ignitia

🔥 **First off, thanks for taking the time to contribute to Ignitia!** 🔥

All types of contributions are encouraged and valued. See the [Table of Contents](#table-of-contents) for different ways to help and details about how this project handles them. Please make sure to read the relevant section before making your contribution.

> **Love the project but don't have time to contribute?** That's fine! There are other easy ways to support Ignitia:
> - ⭐ Star the project on GitHub
> - 🐦 Tweet about it using #IgnitiaFramework
> - 📝 Reference Ignitia in your project's README
> - 🗣️ Mention the project at Rust meetups and conferences
> - 📺 Create content (blog posts, videos, tutorials) about Ignitia
> - ☕ Support development on [Buy Me A Coffee](https://buymeacoffee.com/aarambhdevhub)

## Table of Contents

- [Code of Conduct](#code-of-conduct)
- [I Have a Question](#i-have-a-question)
- [Getting Started](#getting-started)
- [Development Environment](#development-environment)
- [Ways to Contribute](#ways-to-contribute)
- [Reporting Bugs](#reporting-bugs)
- [Suggesting Features](#suggesting-features)
- [Code Contributions](#code-contributions)
- [Documentation](#documentation)
- [Testing Guidelines](#testing-guidelines)
- [Style Guidelines](#style-guidelines)
- [Commit Guidelines](#commit-guidelines)
- [Pull Request Process](#pull-request-process)
- [Security](#security)
- [Community](#community)

## Code of Conduct

This project and everyone participating in it is governed by our **Code of Conduct**. By participating, you are expected to uphold this code. Please report unacceptable behavior to the project maintainers.

**Our Pledge**: We are committed to making participation in this project a harassment-free experience for everyone, regardless of age, body size, disability, ethnicity, gender identity and expression, level of experience, nationality, personal appearance, race, religion, or sexual identity and orientation.

## I Have a Question

> If you want to ask a question, we assume that you have read the available [Documentation](../README.md).

Before asking a question:

1. **Search existing issues** - Your question might already be answered
2. **Check the documentation** - Look through README, guides, and examples
3. **Search discussions** - Check GitHub Discussions for similar topics

If you still need help:
- 💬 **GitHub Discussions** - For general questions and community discussion
- 🐛 **GitHub Issues** - For bug reports and feature requests
- 🚀 **Discord** - For real-time community chat: [https://discord.gg/HDth6PfCnp](https://discord.gg/HDth6PfCnp)

## Getting Started

### Prerequisites

- **Rust 1.70+** - Install via [rustup](https://rustup.rs/)
- **Git** - For version control
- **IDE/Editor** - We recommend VS Code with rust-analyzer

### Quick Setup

```
# Clone the repository
git clone https://github.com/AarambhDevHub/ignitia.git
cd ignitia

# Check that everything works
cargo check

# Run tests
cargo test

# Build examples
cargo build --examples
```

## Development Environment

### Building

```
# Standard build
cargo build

# Release build
cargo build --release

# Build with all features
cargo build --all-features

# Build specific features
cargo build --features "websocket,tls,self-signed"
```

### Testing

```
# Run all tests
cargo test

# Run with all features
cargo test --all-features

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture
```

### Documentation

```
# Build documentation
cargo doc --no-deps --all-features

# Open documentation in browser
cargo doc --no-deps --all-features --open
```

## Ways to Contribute

### 🐛 Bug Reports
Found a bug? Help us squash it! See [Reporting Bugs](#reporting-bugs).

### 💡 Feature Suggestions
Have an idea? We'd love to hear it! See [Suggesting Features](#suggesting-features).

### 🔧 Code Contributions
Ready to code? See [Code Contributions](#code-contributions).

### 📚 Documentation
Help make Ignitia easier to use! See [Documentation](#documentation).

### 🧪 Testing
Help us improve test coverage and quality.

### 🌟 Examples
Create examples showing how to use Ignitia in different scenarios.

### 📦 Middleware & Extensions
Build reusable middleware components for the community.

## Reporting Bugs

### Before Submitting

- **Use the latest version** - Update to the latest version of Ignitia
- **Search existing issues** - Check if the bug is already reported
- **Minimal reproduction** - Can you reproduce it with minimal code?
- **Check dependencies** - Ensure compatible versions of dependencies

### Bug Report Template

**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create a new project with '...'
2. Add this code '...'
3. Run with '...'
4. See error

**Expected behavior**
What you expected to happen.

**Actual behavior**
What actually happened.

**Environment:**
- OS: [e.g. Ubuntu 22.04, macOS 13.0, Windows 11]
- Rust version: [e.g. 1.75.0]
- Ignitia version: [e.g. 0.2.2]
- Features enabled: [e.g. websocket, tls]

**Additional context**
- Stack trace or error messages
- Minimal reproduction code
- Any relevant logs


## Suggesting Features

### Before Submitting

- **Check the roadmap** - See if it's already planned
- **Search existing issues** - Avoid duplicates
- **Consider scope** - Does it fit Ignitia's goals?
- **Think about API design** - How should it work?

### Feature Request Template


**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Other solutions or features you've considered.

**API Design (if applicable)**
```
// Show how the API might look
```

**Use cases**
Who would use this and why?

**Additional context**
Any other context, screenshots, or examples.


## Code Contributions

### Architecture Overview

Ignitia is structured as follows:

```
src/
├── lib.rs              # Main library entry point
├── server/             # HTTP server implementation
├── router/             # Request routing
├── middleware/         # Middleware components
├── handler/            # Request handlers and extractors
├── request/            # Request types and utilities
├── response/           # Response types and utilities
├── websocket/          # WebSocket support (feature-gated)
├── multipart/          # File upload support
├── cookie/             # Cookie handling
├── extension/          # Type-safe extensions
├── error/              # Error types and handling
└── utils/              # Utility functions
```

### Key Principles

1. **Performance First** - Ignitia prioritizes speed and efficiency
2. **Type Safety** - Leverage Rust's type system for correctness
3. **Ergonomics** - Easy to use APIs with sensible defaults
4. **Modularity** - Features should be optional via feature flags
5. **Standards Compliance** - Follow HTTP and web standards
6. **Documentation** - All public APIs must be documented

### Feature Development

1. **Create an issue** - Discuss the feature before implementation
2. **Feature flag** - New features should be feature-gated when appropriate
3. **Tests** - Include comprehensive tests
4. **Documentation** - Update docs and examples
5. **Performance** - Consider performance impact

### Feature Flags

Ignitia uses feature flags for optional functionality:

```
[features]
default = []
websocket = ["dep:tokio-tungstenite", "dep:tungstenite", "dep:sha1", "dep:base64"]
tls = ["dep:tokio-rustls", "dep:rustls", "dep:rustls-pemfile"]
self-signed = ["tls", "dep:rcgen"]
```

### Adding New Features

1. **Plan the API** - Design the public interface first
2. **Feature gate** - Add appropriate feature flags
3. **Implement** - Write the implementation
4. **Test** - Add comprehensive tests
5. **Document** - Add documentation and examples
6. **Integration** - Ensure it works with existing features

## Documentation

### Types of Documentation

1. **API Documentation** - Rustdoc comments on public APIs
2. **Guides** - Step-by-step tutorials in `/doc`
3. **Examples** - Working code examples in `/examples`
4. **README** - Project overview and quick start

### Documentation Standards

```rust
/// Brief description of what this function does.
///
/// Longer description explaining the behavior, edge cases,
/// and any important details.
///
/// # Arguments
///
/// * `param1` - Description of the first parameter
/// * `param2` - Description of the second parameter
///
/// # Returns
///
/// Description of what the function returns.
///
/// # Errors
///
/// Description of when and why this function might error.
///
/// # Examples
///
/// ```
/// use ignitia::Router;
///
/// let router = Router::new()
///     .get("/", || async { Ok(Response::text("Hello!")) });
/// ```
pub fn example_function(param1: Type1, param2: Type2) -> Result<ReturnType> {
    // Implementation
}
```

### Writing Guides

1. **Start with the problem** - What does this solve?
2. **Step-by-step** - Break it down into clear steps
3. **Complete examples** - Show working, runnable code
4. **Explain concepts** - Don't assume knowledge
5. **Link related topics** - Help users discover more

## Testing Guidelines

### Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_feature_works() {
        // Arrange
        let router = Router::new();

        // Act
        let response = router.handle(request).await;

        // Assert
        assert!(response.is_ok());
    }
}
```

### Test Categories

1. **Unit Tests** - Test individual functions and methods
2. **Integration Tests** - Test feature interactions
3. **Example Tests** - Ensure examples compile and run
4. **Benchmark Tests** - Performance regression testing

### Testing Best Practices

- **Test edge cases** - Empty inputs, boundary conditions
- **Test error paths** - Ensure errors are handled correctly
- **Use descriptive names** - Test names should explain what they test
- **Keep tests simple** - One concept per test
- **Mock external dependencies** - Keep tests fast and reliable

## Style Guidelines

### Rust Code Style

We follow the standard Rust style guidelines:

```
# Format code
cargo fmt

# Check style and common mistakes
cargo clippy

# Check with all features
cargo clippy --all-features
```

### Code Organization

```
// 1. Standard library imports
use std::collections::HashMap;
use std::sync::Arc;

// 2. External crate imports
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

// 3. Internal imports
use crate::error::{Error, Result};
use crate::middleware::Middleware;

// 4. Type definitions
type HandlerResult = Result<Response>;

// 5. Constants
const DEFAULT_TIMEOUT: u64 = 30;

// 6. Implementation
```

### Naming Conventions

- **Types**: `PascalCase` (e.g., `RouterConfig`, `HttpMethod`)
- **Functions**: `snake_case` (e.g., `handle_request`, `parse_headers`)
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `DEFAULT_PORT`, `MAX_BODY_SIZE`)
- **Modules**: `snake_case` (e.g., `middleware`, `websocket`)

### Error Handling

```
// Prefer Result types
pub fn parse_config(input: &str) -> Result<Config> {
    // Implementation
}

// Use custom error types
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("Invalid format: {0}")]
    InvalidFormat(String),
}

// Provide context
.map_err(|e| Error::Internal(format!("Failed to parse config: {}", e)))
```

## Commit Guidelines

### Commit Message Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- **feat**: New feature
- **fix**: Bug fix
- **docs**: Documentation changes
- **style**: Code style changes (formatting, etc.)
- **refactor**: Code refactoring
- **perf**: Performance improvements
- **test**: Adding or updating tests
- **chore**: Maintenance tasks

### Examples

```
feat(websocket): add batch message handling

Add support for processing WebSocket messages in batches to improve
performance for high-throughput applications.

Closes #123
```

```
fix(router): handle empty path parameters correctly

Previously, empty path parameters would cause a panic. Now they are
handled gracefully and return a BadRequest error.

Fixes #456
```

## Pull Request Process

### Before Opening a PR

1. **Fork the repository** and create a feature branch
2. **Write tests** for your changes
3. **Update documentation** if needed
4. **Run the full test suite** locally
5. **Check formatting** with `cargo fmt`
6. **Check for issues** with `cargo clippy`

### PR Template

## Description
Brief description of changes.

## Type of Change
- [ ] Bug fix (non-breaking change fixing an issue)
- [ ] New feature (non-breaking change adding functionality)
- [ ] Breaking change (fix or feature causing existing functionality to break)
- [ ] Documentation update

## Testing
- [ ] Tests pass locally
- [ ] Added tests for new functionality
- [ ] Updated existing tests if needed

## Checklist
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No new compiler warnings


### Review Process

1. **Automated checks** must pass (CI/CD)
2. **Code review** by maintainers
3. **Discussion** and feedback incorporation
4. **Final approval** and merge

### Branch Naming

- `feature/description` - For new features
- `fix/description` - For bug fixes
- `docs/description` - For documentation
- `refactor/description` - For refactoring

## Security

### Reporting Security Issues

**DO NOT** report security vulnerabilities as public GitHub issues.

<!--Instead, please email security issues to: **[security@aarambhdev.com]**-->

### Security Considerations

When contributing:

1. **Input validation** - Always validate user input
2. **Memory safety** - Leverage Rust's safety guarantees
3. **Dependency security** - Keep dependencies updated
4. **Crypto practices** - Use established crypto libraries
5. **Rate limiting** - Consider DoS attack vectors

## Community

### Getting Help

- 💬 **GitHub Discussions** - Questions and community discussion
- 🐦 **Twitter** - [@AarambhDevHub](https://twitter.com/AarambhDevHub)
- 📺 **YouTube** - [AarambhDevHub Channel](https://youtube.com/@aarambhdevhub)
- 🚀 **Discord** - Real-time community chat: [https://discord.gg/HDth6PfCnp](https://discord.gg/HDth6PfCnp)

### Support the Project

- ☕ **Buy Me A Coffee** - Support development: [buymeacoffee.com/aarambhdevhub](https://buymeacoffee.com/aarambhdevhub)

### Recognition

Contributors are recognized in:
- **CHANGELOG.md** - Notable contributions
- **README.md** - Core contributors
- **Release notes** - Feature contributors
- **Social media** - Shout-outs for great contributions

### Becoming a Maintainer

Active contributors may be invited to become maintainers. Maintainers:
- Review and merge pull requests
- Triage issues and discussions
- Help shape the project roadmap
- Mentor new contributors

***

## Attribution

This guide is inspired by various open source contribution guides and adapted for Ignitia's needs.

**Thank you for contributing to Ignitia! 🔥**
