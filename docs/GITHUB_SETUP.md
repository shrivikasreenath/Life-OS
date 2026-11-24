# LIFE OS GitHub Setup & Remote Deployment Guide
**Owner:** Shrivika  
**Project:** Life OS  
**Security Policy:** Strict isolation from Vedav's `JainOS` repository  

---

## ⚠️ Important Safety Notice

The host machine contains a separate operating system project belonging to Vedav (`JainOS` located at `/home/vedav/rust-micro-os`).
- **NEVER** push Life OS commits to Vedav's GitHub account or repositories.
- **NEVER** configure Vedav's remote URLs inside the Life OS directory.
- Life OS maintains its own dedicated Git repository with author identity strictly set to **Shrivika**.

---

## 1. Verify Current Local Git Configuration

Inside the Life OS directory (`/home/vedav/life-os`):
```bash
# Check current Git author identity
git config user.name
# Output must be: Shrivika

git config user.email
# Output: shrivika@users.noreply.github.com (or Shrivika's college email)

# Verify no remotes are currently configured
git remote -v
```

---

## 2. Step-by-Step GitHub Setup for Shrivika

When Shrivika is ready to push Life OS to her personal GitHub account:

### Step 1: Create a New GitHub Repository
1. Log into your GitHub account at [https://github.com](https://github.com).
2. Click **New Repository** (or visit [https://github.com/new](https://github.com/new)).
3. Repository settings:
   - **Repository Name:** `life-os`
   - **Description:** `Autonomous Bare-Metal Rust Operating System (x86_64)`
   - **Visibility:** Public (or Private as preferred)
   - **Do NOT** check "Add a README", ".gitignore", or "license" (we already have our complete codebase and README locally).
4. Click **Create repository**.

---

### Step 2: Add Remote in Life OS Repository
In your WSL2 terminal inside `/home/vedav/life-os`:

```bash
# If using HTTPS:
git remote add origin https://github.com/<YOUR_GITHUB_USERNAME>/life-os.git

# Or if using SSH:
git remote add origin git@github.com:<YOUR_GITHUB_USERNAME>/life-os.git
```

Replace `<YOUR_GITHUB_USERNAME>` with Shrivika's actual GitHub username.

---

### Step 3: Push Code to GitHub
```bash
# Ensure branch is named main
git branch -M main

# Push all commits and tags to Shrivika's GitHub repository
git push -u origin main
```

---

## 3. Uploading the Bootable ISO to GitHub Releases

Because the final bootable ISO (`build/LifeOS.iso`) is only **3.41 MB**, it can easily be hosted on GitHub Releases for examiners and evaluators to download and test immediately:

1. Go to your repository on GitHub: `https://github.com/<YOUR_GITHUB_USERNAME>/life-os`.
2. On the right side, click **Releases -> Create a new release**.
3. Tag version: `v1.0.0`
4. Release title: `Life OS v1.0.0 — Bootable Bare-Metal Rust Operating System`
5. Drag and drop `build/LifeOS.iso` into the binary attachment box.
6. Click **Publish release**.

Anyone can now download `LifeOS.iso`, attach it to Oracle VirtualBox, and boot it in under 2 seconds!
