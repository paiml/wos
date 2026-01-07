# WOS Deployment Guide

This guide explains how to deploy WOS to AWS S3 and CloudFront.

## Prerequisites

1. **AWS CLI**: Install from https://aws.amazon.com/cli/
2. **AWS Credentials**: Configure with `aws configure`
3. **S3 Bucket**: Create a bucket for hosting (e.g., `my-wos-app`)
4. **CloudFront Distribution**: (Optional) For CDN and HTTPS

## Quick Start

### 1. Create Deployment Configuration

```bash
# Generate configuration template
make deploy-config

# Copy and edit with your AWS details
cp .env.deploy.example .env.deploy
nano .env.deploy  # Edit with your S3 bucket and CloudFront ID
```

### 2. Deploy

```bash
# Full deployment (build + upload + invalidate cache)
make deploy
```

That's it! Your app is now live at your S3/CloudFront URL.

## Detailed Configuration

### .env.deploy

```bash
# S3 Bucket name
S3_BUCKET=my-wos-app-production

# CloudFront Distribution ID (optional, for cache invalidation)
CLOUDFRONT_DISTRIBUTION_ID=E1234567890ABC

# AWS Region
AWS_REGION=us-east-1

# AWS Profile (optional)
# AWS_PROFILE=my-profile
```

### S3 Bucket Setup

1. **Create bucket**:
   ```bash
   aws s3 mb s3://my-wos-app-production
   ```

2. **Enable static website hosting**:
   ```bash
   aws s3 website s3://my-wos-app-production/ \
     --index-document index.html \
     --error-document index.html
   ```

3. **Set bucket policy** (public read access):
   ```json
   {
     "Version": "2012-10-17",
     "Statement": [
       {
         "Sid": "PublicReadGetObject",
         "Effect": "Allow",
         "Principal": "*",
         "Action": "s3:GetObject",
         "Resource": "arn:aws:s3:::my-wos-app-production/*"
       }
     ]
   }
   ```

   Apply with:
   ```bash
   aws s3api put-bucket-policy \
     --bucket my-wos-app-production \
     --policy file://bucket-policy.json
   ```

### CloudFront Setup (Optional)

CloudFront provides:
- **HTTPS**: Secure connection
- **CDN**: Fast global delivery
- **Custom domain**: your-domain.com

1. **Create distribution**:
   - Origin: Your S3 bucket website endpoint
   - Default root object: `index.html`
   - Viewer protocol: Redirect HTTP to HTTPS
   - Price class: Use Only US, Canada and Europe (or your preference)

2. **Custom error responses**:
   - 404 → `/index.html` (200) - For client-side routing

3. **Note your Distribution ID** for `.env.deploy`

## Make Targets

```bash
make deploy               # Full deployment workflow
make deploy-build         # Build production WASM only
make deploy-upload        # Upload to S3 only
make deploy-invalidate    # Invalidate CloudFront cache only
make deploy-check         # Check prerequisites
make deploy-config        # Create .env.deploy.example
```

## Deployment Workflow

### Automatic (Recommended)

```bash
make deploy
```

This runs:
1. `deploy-check` - Verify AWS CLI and config
2. `deploy-build` - Build optimized WASM
3. `deploy-upload` - Sync to S3 bucket
4. `deploy-invalidate` - Clear CloudFront cache

### Manual Steps

```bash
# 1. Build production assets
make wasm

# 2. Upload to S3
source .env.deploy
bash scripts/deploy-s3.sh

# 3. Invalidate CloudFront (optional)
bash scripts/deploy-invalidate.sh
```

## Cache Strategy

### Long cache (1 year) for immutable assets:
- JavaScript files
- CSS files
- WASM files
- Images

### Short cache (5 minutes) for entry point:
- `index.html` - Allows quick updates

CloudFront invalidation clears all caches immediately.

## Troubleshooting

### "AWS CLI not found"
```bash
# Install AWS CLI
pip install awscli
# or
brew install awscli
```

### "Credentials not configured"
```bash
aws configure
# Enter: Access Key ID, Secret Access Key, Region, Output format
```

### "Permission denied"
Ensure your AWS user/role has:
- `s3:PutObject`, `s3:DeleteObject`, `s3:ListBucket` for S3
- `cloudfront:CreateInvalidation`, `cloudfront:GetInvalidation` for CloudFront

### "Bucket does not exist"
```bash
# Create the bucket
aws s3 mb s3://your-bucket-name
```

### "Distribution not found"
Check your CloudFront Distribution ID in AWS Console:
- CloudFront → Distributions → ID column

## CI/CD Integration

### GitHub Actions Example

```yaml
name: Deploy WOS

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          target: wasm32-unknown-unknown
      
      - name: Build WASM
        run: make wasm
      
      - name: Configure AWS
        uses: aws-actions/configure-aws-credentials@v2
        with:
          aws-access-key-id: ${{ secrets.AWS_ACCESS_KEY_ID }}
          aws-secret-access-key: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          aws-region: us-east-1
      
      - name: Deploy to S3
        env:
          S3_BUCKET: ${{ secrets.S3_BUCKET }}
          CLOUDFRONT_DISTRIBUTION_ID: ${{ secrets.CLOUDFRONT_DISTRIBUTION_ID }}
        run: |
          bash scripts/deploy-s3.sh
          bash scripts/deploy-invalidate.sh
```

## Rollback

To rollback to a previous version:

```bash
# List previous versions (if versioning enabled)
aws s3api list-object-versions \
  --bucket my-wos-app-production \
  --prefix index.html

# Restore a specific version
aws s3api copy-object \
  --bucket my-wos-app-production \
  --copy-source my-wos-app-production/index.html?versionId=VERSION_ID \
  --key index.html

# Invalidate cache
make deploy-invalidate
```

## Security Best Practices

1. **Enable S3 bucket versioning** (for rollback capability)
2. **Use IAM roles** (not root credentials)
3. **Enable CloudFront** (for HTTPS)
4. **Add .env.deploy to .gitignore** (never commit secrets)
5. **Use AWS Secrets Manager** (for production secrets)
6. **Enable CloudTrail** (audit deployments)

## Cost Optimization

- **S3**: ~$0.023/GB/month storage + ~$0.09/GB transfer
- **CloudFront**: Free tier includes 1TB transfer/month (first 12 months)
- **Total**: Expect <$5/month for typical usage

## Monitoring

### Check deployment status:
```bash
# S3 bucket contents
aws s3 ls s3://my-wos-app-production/

# CloudFront distribution status
aws cloudfront get-distribution \
  --id $CLOUDFRONT_DISTRIBUTION_ID \
  --query 'Distribution.Status'

# Recent invalidations
aws cloudfront list-invalidations \
  --distribution-id $CLOUDFRONT_DISTRIBUTION_ID
```

### Test deployment:
```bash
# Direct S3 (HTTP)
curl -I http://my-wos-app-production.s3-website-us-east-1.amazonaws.com/

# CloudFront (HTTPS)
curl -I https://d111111abcdef8.cloudfront.net/

# Custom domain (if configured)
curl -I https://wos.yourdomain.com/
```

## Support

- **AWS Documentation**: https://docs.aws.amazon.com/
- **WOS Issues**: https://github.com/paiml/wos/issues
- **AWS CLI Reference**: https://awscli.amazonaws.com/v2/documentation/api/latest/index.html

## Next Steps

After deployment:
1. Test your live site
2. Configure custom domain (Route 53)
3. Set up continuous deployment (GitHub Actions)
4. Monitor usage (CloudWatch)
5. Enable error tracking (Sentry/CloudWatch)
