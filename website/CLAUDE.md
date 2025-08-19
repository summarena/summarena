# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

This is an Astro-based website with Starlight documentation integration. Use pnpm as the package manager.

**Essential Commands:**

- `pnpm dev` - Start development server at localhost:4321
- `pnpm build` - Build production site (includes type checking)
- `pnpm lint` - Run prettier check and Astro type checking
- `pnpm lint:fix` - Auto-fix formatting issues with prettier
- `pnpm check` or `pnpm type-check` - Run Astro type checking only

**Testing:**

- `pnpm test` - Run unit tests with Vitest
- `pnpm test:run` - Run tests once (non-watch mode)
- `pnpm test:coverage` - Run tests with coverage report
- `pnpm test:e2e` - Run Playwright end-to-end tests
- `pnpm test:e2e:ui` - Run e2e tests with Playwright UI

## Architecture Overview

**Framework Stack:**

- Astro 5.x with TypeScript
- Starlight for documentation integration
- Tailwind CSS for styling
- Vue.js integration for interactive components
- MDX support for enhanced markdown

**Content Structure:**

- `src/content/blog/` - Blog posts (Markdown/MDX with frontmatter schema)
- `src/content/docs/` - Documentation pages managed by Starlight
- `src/pages/` - Astro page components and routes
- `src/components/` - Reusable Astro components
- `src/layouts/` - Page layout templates

**Testing Setup:**

- Vitest for unit tests with jsdom environment
- Playwright for e2e testing (chromium, webkit)
- Test files in `tests/` directory and `src/` with `.test` or `.spec` extensions
- E2E tests in `e2e/` directory

**Key Configuration:**

- Content collections defined in `src/content.config.ts` with Zod schemas
- Blog posts require title, description, pubDate fields
- Starlight sidebar and social links configured in `astro.config.mjs`
- Site URL: https://summarena.com

**Development Notes:**

- Uses pnpm workspace structure
- Prettier for code formatting
- TypeScript strict mode enabled
- Analytics script in `src/scripts/analytics.ts`
- **CRITICAL: After completing ANY set of changes, ALWAYS run `pnpm lint:fix` and `pnpm check` to ensure code quality and fix any formatting or type issues**
- ALWAYS add tests for new functionality. Add regression tests when fixing bugs. Add a mix of unit, ui, and integration tests as necessary
