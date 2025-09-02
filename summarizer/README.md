# Summarizer

This directory contains automation scripts to generate summaries.

## Overview

### Input
Possible input sources:
1. **local PDF/MD files**: Uploads from local storage
2. **Google Drive ID**: Only supports google docs
3. **URLs (e.g. arXiv hyperlinks)**: Adds directly to notebookLM

### Output

Output is currently dumped to the console. It can easily be integrated with summarena.

### Limitations

NotebookLM [limits](https://support.google.com/notebooklm/answer/16213268?hl=en):
* 500 notebooks
* 300 sources per notebook
* 500k words per source
* 500 daily chat queries

So we can support roughly 500 users with one Google account

### Example command

First, start a Chrome browser with remote debugging enabled on port 9222:

```bash
google-chrome --remote-debugging-port=9222 --user-data-dir=~/.config/google-chrome/summarizer
```

Then run the summarizer. First provide user_id, then separate potentially multiple inputs with a space:

```bash
poetry run python src/summarizer/module.py user1 https://arxiv.org/pdf/2504.11741 /home/user/file.pdf https://drive.google.com/drive/folders/1qCBf3n5...fij
```
