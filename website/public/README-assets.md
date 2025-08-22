# Public Assets Generation Guide

This directory contains templates and instructions for generating the required public assets for SummArena.

## Required Assets

### 1. Social Sharing Image (`og-1200x630.png`)

- **Dimensions:** 1200×630px
- **Format:** PNG
- **Template:** `og-template.html`
- **Usage:** Social media sharing (Twitter, LinkedIn, Facebook)

**To generate:**

```bash
# Use a tool like Puppeteer, Playwright, or screenshot service
npx playwright screenshot og-template.html og-1200x630.png --viewport-size=1200,630
```

### 2. Hero Images (Responsive)

- **brief-600.webp** (600w)
- **brief-900.webp** (900w)
- **brief-1200.webp** (1200w)
- **Template:** `brief-mockup.svg`
- **Usage:** Hero section of landing page

**To generate:**

```bash
# Convert SVG to WebP at different sizes
convert brief-mockup.svg -resize 600x400 brief-600.webp
convert brief-mockup.svg -resize 900x600 brief-900.webp
convert brief-mockup.svg -resize 1200x800 brief-1200.webp
```

### 3. Sample Brief PDF (`sample-brief.pdf`)

- **Template:** `sample-brief-template.html`
- **Usage:** Downloadable sample for users

**To generate:**

```bash
# Use Puppeteer or wkhtmltopdf
npx playwright pdf sample-brief-template.html sample-brief.pdf --format=A4
```

## Brand Colors Used

- **Primary:** #2b61d0 (brand-600)
- **Secondary:** #244ea8 (brand-700)
- **Light:** #eef6ff (brand-50)
- **Accent:** #d9ebff (brand-100)

## Content Guidelines

- **Branding:** Always use "SummArena" (not "Feed Summarizer")
- **Tagline:** "Your AI research brief with citations"
- **Style:** Professional, clean, tech-focused
- **Typography:** System fonts (-apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto)
