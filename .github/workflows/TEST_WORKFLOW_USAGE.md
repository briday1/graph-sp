# Testing the Publish Workflow

This document explains how to use the `test-publish.yml` workflow to verify that the publish pipeline works correctly.

## Overview

The `test-publish.yml` workflow allows you to:
1. **Test version generation** - Verify that versions are correctly generated from PyPI
2. **Test builds** - Run Rust tests and examples, build Python wheels
3. **Test wheel building** - Build wheels for multiple platforms and Python versions
4. **Test publishing** - Optionally publish to TestPyPI to verify the full pipeline

## Quick Start

### Basic Test (No Publishing)

1. Go to the **Actions** tab in GitHub
2. Select **"Test Publish Workflow"** from the left sidebar
3. Click **"Run workflow"**
4. Keep default settings:
   - `publish_to_testpypi`: **false** (unchecked)
   - `skip_tests`: **false** (unchecked)
5. Click **"Run workflow"** button

This will:
- Generate a new version number from PyPI
- Run all Rust tests and examples
- Run all Python examples
- Build wheels for 4 platform/Python combinations
- Display a summary of results

**Duration:** ~10-15 minutes

### Full Test (With TestPyPI Publishing)

To test the complete publishing flow including uploading to TestPyPI:

1. **First, get a TestPyPI token:**
   - Go to https://test.pypi.org
   - Create an account (if you don't have one)
   - Go to Account Settings → API tokens
   - Click "Add API token"
   - Set scope to "Entire account" or specific to "pygraph-sp"
   - Copy the token (starts with `pypi-`)

2. **Add the token to GitHub secrets:**
   - Go to repository **Settings** → **Secrets and variables** → **Actions**
   - Click **"New repository secret"**
   - Name: `TEST_PYPI_API_TOKEN`
   - Value: Paste your TestPyPI token
   - Click **"Add secret"**

3. **Run the workflow:**
   - Go to **Actions** tab
   - Select **"Test Publish Workflow"**
   - Click **"Run workflow"**
   - Set options:
     - `publish_to_testpypi`: **true** (checked) ✓
     - `skip_tests`: **false** (unchecked)
   - Click **"Run workflow"**

This will do everything from the basic test PLUS:
- Build a source distribution
- Verify all distributions with `twine check`
- Upload to TestPyPI
- Provide a link to view the published package

**Duration:** ~15-20 minutes

## Workflow Options

### `publish_to_testpypi`
- **Default:** `false`
- **Purpose:** Enable/disable publishing to TestPyPI
- **When to use:** Set to `true` when you want to verify the complete publish pipeline

### `skip_tests`
- **Default:** `false`
- **Purpose:** Skip running Rust tests and examples (faster iteration)
- **When to use:** Set to `true` if you only want to test wheel building without running all tests

## What Gets Tested

### Version Generation
- Queries PyPI for existing versions of `pygraph-sp`
- Generates next version in `YYYY.N` format
- Updates `pyproject.toml` and `Cargo.toml`

### Rust Testing (unless skipped)
- Runs `cargo test --verbose`
- Executes 6 Rust examples:
  - comprehensive_demo
  - output_access_demo
  - parallel_execution_demo
  - per_node_output_access
  - tuple_api_demo
  - variant_demo_full

### Python Testing (unless skipped)
- Builds wheel with maturin 1.2.0
- Installs the wheel
- Runs 3 Python examples:
  - python_comprehensive_demo.py
  - python_demo.py
  - python_parallel_demo.py

### Wheel Building
- Builds wheels for:
  - **OS:** ubuntu-latest, windows-latest
  - **Python:** 3.10, 3.11
  - **Total:** 4 wheel builds (subset for faster testing)
- Uploads artifacts for inspection

### Publishing (if enabled)
- Downloads all wheel artifacts
- Builds source distribution
- Verifies all distributions with `twine check`
- Publishes to TestPyPI

## Expected Results

### Success Indicators
After the workflow completes, you should see:

```
✅ WORKFLOW VALIDATION SUCCESSFUL!

The publish workflow is working correctly.
```

If you enabled TestPyPI publishing:
```
✅ Successfully published to TestPyPI!
View at: https://test.pypi.org/project/pygraph-sp/[VERSION]/
```

### Failure Scenarios

**Version generation fails:**
- Check if PyPI is accessible
- Verify package name is correct (`pygraph-sp`)

**Tests/examples fail:**
- Check the specific job logs for error details
- Rust tests use `continue-on-error: true`, so they won't block the workflow

**Wheel building fails:**
- Check Rust/Python setup logs
- Verify maturin version compatibility

**TestPyPI publish fails:**
- Verify `TEST_PYPI_API_TOKEN` secret is set correctly
- Check if the version already exists on TestPyPI
- Ensure token has proper permissions

## Differences from Production Workflow

The test workflow differs from `publish.yml` in these ways:

1. **Reduced matrix:** Tests 2 OS × 2 Python versions (4 builds) instead of 3 × 4 (12 builds) for faster feedback
2. **TestPyPI target:** Publishes to test.pypi.org instead of pypi.org
3. **Manual trigger only:** No automatic tag-based triggering
4. **Optional components:** Can skip tests or publishing for faster iteration
5. **Enhanced logging:** Provides detailed summary of all steps

## Viewing Results

### Artifacts
After the workflow runs, you can download the built wheels:
1. Go to the workflow run page
2. Scroll to **"Artifacts"** section at the bottom
3. Download `test-wheels-*` artifacts to inspect

### TestPyPI Package
If you published to TestPyPI, view your package at:
```
https://test.pypi.org/project/pygraph-sp/
```

To install from TestPyPI for testing:
```bash
pip install -i https://test.pypi.org/simple/ pygraph-sp
```

## Troubleshooting

### "TEST_PYPI_API_TOKEN secret is not set"
- Add the secret as described in "Full Test" section above

### "Version already exists on TestPyPI"
- TestPyPI doesn't allow overwriting versions
- Wait for the workflow to generate a new version number
- Or manually delete old versions from TestPyPI

### Workflow is slow
- Set `skip_tests: true` to skip running examples
- The wheel building matrix is already reduced for speed

### Can't find the workflow
- Check that you're on the correct branch
- Refresh the Actions page

## Production Publishing

Once the test workflow validates successfully, you can use the production `publish.yml` workflow:

**For a release:**
```bash
git tag v1.0.0
git push origin v1.0.0
```

This will trigger the full production workflow that:
- Builds wheels for all 12 platform/Python combinations
- Publishes to production PyPI (not TestPyPI)
- Publishes to crates.io

**For manual publishing:**
1. Go to Actions → "Publish to crates.io and PyPI"
2. Click "Run workflow"
3. Select branch and run

## Summary

The test workflow provides a safe way to verify the publish pipeline without affecting production PyPI. Use it to:
- ✅ Validate version generation logic
- ✅ Ensure all tests and examples pass
- ✅ Verify wheel building works across platforms
- ✅ Test the complete publish flow end-to-end

Run it before making any changes to the production `publish.yml` workflow!
