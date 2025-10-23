#!/usr/bin/env bash
# Link WebOS dist to interactive.paiml.com for rapid development iteration
# This script creates a symlink so changes in wos/dist/wos appear immediately in the paiml.com dist

set -euo pipefail

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

WOS_DIST="/home/noah/src/wos/dist/wos"
PAIML_DIST="/home/noah/src/interactive.paiml.com/dist/wos"

echo -e "${BLUE}🔗 WebOS Symlink Deployment${NC}"
echo ""

# Verify source exists
if [[ ! -d "$WOS_DIST" ]]; then
    echo -e "${YELLOW}⚠️  WebOS dist not found. Building...${NC}"
    cd /home/noah/src/wos
    make build
fi

# Remove existing destination (file, dir, or symlink)
if [[ -e "$PAIML_DIST" ]] || [[ -L "$PAIML_DIST" ]]; then
    echo -e "${BLUE}📦 Removing existing: $PAIML_DIST${NC}"
    rm -rf "$PAIML_DIST"
fi

# Create symlink
echo -e "${BLUE}🔗 Creating symlink: $PAIML_DIST -> $WOS_DIST${NC}"
ln -s "$WOS_DIST" "$PAIML_DIST"

# Verify
if [[ -L "$PAIML_DIST" ]]; then
    LINK_TARGET=$(readlink -f "$PAIML_DIST")
    echo ""
    echo -e "${GREEN}✅ Symlink created successfully!${NC}"
    echo -e "${GREEN}   Source: $WOS_DIST${NC}"
    echo -e "${GREEN}   Link:   $PAIML_DIST${NC}"
    echo -e "${GREEN}   Target: $LINK_TARGET${NC}"
    echo ""
    echo -e "${BLUE}💡 Now you can:${NC}"
    echo -e "   1. Edit WebOS code in /home/noah/src/wos/"
    echo -e "   2. Run 'make build' in wos/"
    echo -e "   3. Changes appear immediately in paiml.com dist/"
    echo -e "   4. Run 'make deploy' in interactive.paiml.com/ when ready"
else
    echo -e "${YELLOW}⚠️  Symlink creation failed${NC}"
    exit 1
fi
