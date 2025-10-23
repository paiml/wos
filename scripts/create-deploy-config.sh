#!/usr/bin/env bash
# Create deployment configuration template
set -euo pipefail

CONFIG_FILE=".env.deploy.example"

cat > "$CONFIG_FILE" << 'ENVEOF'
# WOS Deployment Configuration
# Copy this file to .env.deploy and fill in your AWS credentials

# S3 Bucket name (e.g., my-wos-app-production)
S3_BUCKET=your-bucket-name-here

# CloudFront Distribution ID (e.g., E1234567890ABC)
CLOUDFRONT_DISTRIBUTION_ID=your-distribution-id-here

# AWS Region (e.g., us-east-1)
AWS_REGION=us-east-1

# AWS Profile (optional, uses default if not set)
# AWS_PROFILE=your-profile-name
ENVEOF

echo "✓ Created ${CONFIG_FILE}"
echo ""
echo "Next steps:"
echo "  1. Copy: cp .env.deploy.example .env.deploy"
echo "  2. Edit .env.deploy with your AWS credentials"
echo "  3. Run: make deploy"
