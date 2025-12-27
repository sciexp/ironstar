# Secrets Management with SOPS

This directory contains encrypted secrets managed by [SOPS](https://github.com/getsops/sops) for CI/CD workflows.

## Required Secrets

Create a file `shared.yaml` in this directory with the following secrets:

```yaml
CLOUDFLARE_ACCOUNT_ID: your-cloudflare-account-id
CLOUDFLARE_API_TOKEN: your-cloudflare-api-token
GITGUARDIAN_API_KEY: your-gitguardian-api-key
CACHIX_AUTH_TOKEN: your-cachix-auth-token
CACHIX_CACHE_NAME: your-cachix-cache-name
CI_AGE_KEY: your-ci-age-private-key
FAST_FORWARD_PAT: your-github-pat-for-pr-merges
FLAKE_UPDATER_APP_ID: your-github-app-id
FLAKE_UPDATER_PRIVATE_KEY: your-github-app-private-key-pem
```

## Cloudflare Secrets

To deploy to Cloudflare Workers, you need:

1. **CLOUDFLARE_ACCOUNT_ID**: Your Cloudflare account ID
   - Find at: https://dash.cloudflare.com/ → (select account) → Workers & Pages → Overview (right sidebar)

2. **CLOUDFLARE_API_TOKEN**: API token with Workers deployment permissions
   - Create at: https://dash.cloudflare.com/profile/api-tokens
   - Required permissions: Account.Workers Scripts (Edit), Account.Workers Routes (Edit)

## CI/CD Secrets

3. **GITGUARDIAN_API_KEY**: API key for GitGuardian secret scanning
   - Get from: https://dashboard.gitguardian.com/api/personal-access-tokens

4. **CACHIX_AUTH_TOKEN**: Auth token for Nix binary cache
   - Get from: https://app.cachix.org/cache/YOUR_CACHE/settings

5. **CACHIX_CACHE_NAME**: Name of your Cachix cache

6. **CI_AGE_KEY**: Private age key for CI to decrypt secrets
   - This should be the private key corresponding to the CI public key in `.sops.yaml`
   - The public key in `.sops.yaml` is: `age1m9m8h5vqr7dqlmvnzcwshmm4uk8umcllazum6eaulkdp3qc88ugs22j3p8`

## GitHub Automation Secrets

7. **FAST_FORWARD_PAT**: Personal access token for PR fast-forward merges
   - Create at: https://github.com/settings/tokens
   - Required scopes: `repo` (full control of private repositories)

8. **FLAKE_UPDATER_APP_ID** and **FLAKE_UPDATER_PRIVATE_KEY**: GitHub App credentials for automated flake input updates
   - Create a GitHub App at: https://github.com/settings/apps/new
   - Required permissions: Contents (Read and write), Pull requests (Read and write)
   - After creation, generate and download a private key
   - The App ID is displayed on the app's settings page
   - Install the app on your repository

## Encrypting Secrets

After creating `shared.yaml` with unencrypted values:

```bash
# Encrypt the file in place
sops --encrypt --in-place vars/shared.yaml

# Or encrypt to a new file
sops --encrypt vars/shared.yaml > vars/shared.yaml.enc
mv vars/shared.yaml.enc vars/shared.yaml
```

## Editing Encrypted Secrets

```bash
# Edit encrypted file
sops vars/shared.yaml
```

SOPS will decrypt, open in your editor, and re-encrypt on save.

## Uploading to GitHub

After encrypting `shared.yaml`, upload the CI_AGE_KEY to GitHub:

```bash
# Set the CI_AGE_KEY as a GitHub secret
gh secret set SOPS_AGE_KEY < <(sops --decrypt --extract '["CI_AGE_KEY"]' vars/shared.yaml)
```

Or manually at: https://github.com/YOUR_USERNAME/ironstar/settings/secrets/actions

## Security Notes

- Never commit unencrypted secrets
- The `.gitignore` is configured to only allow encrypted files
- Encrypted files can be safely committed to the repository
- Only users with the age private keys in `.sops.yaml` can decrypt
