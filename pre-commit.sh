#!/bin/bash
# Pre-commit hook for code quality

echo "🔍 Running pre-commit checks..."

# Check if Rust files were modified
if git diff --cached --name-only | grep -q "\.rs$"; then
    echo "📝 Checking Rust formatting..."
    if ! npm run cargo:fmt:check; then
        echo "❌ Rust code is not formatted. Run 'npm run cargo:fmt' to fix."
        exit 1
    fi
    
    echo "🔍 Running Clippy linter..."
    if ! npm run cargo:lint; then
        echo "❌ Clippy found issues. Please fix them before committing."
        exit 1
    fi
    
    echo "🧪 Running tests..."
    if ! npm run cargo:test; then
        echo "❌ Tests failed. Please fix them before committing."
        exit 1
    fi
fi

# Check if CSS files were modified
if git diff --cached --name-only | grep -q "\.css$"; then
    echo "🎨 Building CSS..."
    npm run build-css
fi

echo "✅ All checks passed!"