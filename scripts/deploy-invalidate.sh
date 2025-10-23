#!/usr/bin/env bash
# Invalidate CloudFront distribution cache
set -euo pipefail

# Load environment variables
: "${CLOUDFRONT_DISTRIBUTION_ID:?Error: CLOUDFRONT_DISTRIBUTION_ID not set in .env.deploy}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "🔄 Invalidating CloudFront distribution: ${CLOUDFRONT_DISTRIBUTION_ID}"

# Create invalidation
INVALIDATION_ID=$(aws cloudfront create-invalidation \
  --distribution-id "${CLOUDFRONT_DISTRIBUTION_ID}" \
  --paths "/*" \
  --query 'Invalidation.Id' \
  --output text)

echo -e "${BLUE}Invalidation ID: ${INVALIDATION_ID}${NC}"
echo "⏳ Waiting for invalidation to complete (this may take 1-2 minutes)..."

# Wait for invalidation to complete
aws cloudfront wait invalidation-completed \
  --distribution-id "${CLOUDFRONT_DISTRIBUTION_ID}" \
  --id "${INVALIDATION_ID}"

echo -e "${GREEN}✓ Cache invalidation complete${NC}"
