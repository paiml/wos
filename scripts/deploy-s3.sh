#!/usr/bin/env bash
# Deploy WOS to S3 bucket
set -euo pipefail

# Load environment variables
: "${S3_BUCKET:?Error: S3_BUCKET not set in .env.deploy}"
: "${CLOUDFRONT_DISTRIBUTION_ID:?Error: CLOUDFRONT_DISTRIBUTION_ID not set in .env.deploy}"

# Colors for output
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "📦 Deploying to S3 bucket: ${S3_BUCKET}"

# Sync dist/ directory to S3
echo -e "${BLUE}Uploading files...${NC}"
aws s3 sync dist/wos/ "s3://${S3_BUCKET}/" \
  --delete \
  --cache-control "public, max-age=31536000, immutable" \
  --exclude "index.html" \
  --exclude "*.wasm"

# Upload index.html with short cache (for updates)
aws s3 cp dist/wos/index.html "s3://${S3_BUCKET}/index.html" \
  --cache-control "public, max-age=300, must-revalidate" \
  --content-type "text/html"

# Upload WASM files with correct MIME type and long cache
for wasm_file in dist/wos/*.wasm; do
  if [ -f "$wasm_file" ]; then
    aws s3 cp "$wasm_file" "s3://${S3_BUCKET}/$(basename "$wasm_file")" \
      --cache-control "public, max-age=31536000, immutable" \
      --content-type "application/wasm"
  fi
done

echo -e "${GREEN}✓ Upload complete${NC}"
