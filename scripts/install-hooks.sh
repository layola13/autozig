#!/bin/bash

# Install Git hooks for AutoZig project

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
HOOKS_DIR="$PROJECT_ROOT/.githooks"
GIT_HOOKS_DIR="$PROJECT_ROOT/.git/hooks"

echo "🔧 Installing Git hooks for AutoZig..."

# Check if .git directory exists
if [ ! -d "$GIT_HOOKS_DIR" ]; then
    echo "❌ Error: .git directory not found. Are you in a git repository?"
    exit 1
fi

# Create symlinks for all hooks
for hook in "$HOOKS_DIR"/*; do
    if [ -f "$hook" ]; then
        hook_name=$(basename "$hook")
        target="$GIT_HOOKS_DIR/$hook_name"
        
        # Remove existing hook if it exists
        if [ -e "$target" ] || [ -L "$target" ]; then
            echo "  Removing existing $hook_name..."
            rm "$target"
        fi
        
        # Create symlink
        echo "  Installing $hook_name..."
        ln -s "$hook" "$target"
        chmod +x "$hook"
        chmod +x "$target"
    fi
done

echo "✅ Git hooks installed successfully!"
echo ""
echo "Installed hooks:"
ls -la "$GIT_HOOKS_DIR" | grep -v "sample" | grep "^l" || echo "  (none)"
echo ""
echo "💡 To skip example verification in pre-push hook, set:"
echo "   export SKIP_EXAMPLES=1"
echo ""
echo "To uninstall hooks, run:"
echo "   rm $GIT_HOOKS_DIR/pre-push"